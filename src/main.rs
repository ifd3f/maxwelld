use std::{
    fs::File,
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use clap::Parser;
use walkdir::WalkDir;

#[derive(clap::Parser)]
struct Args {
    /// File or directory to operate on.
    path: PathBuf,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    for entry in WalkDir::new(args.path) {
        let entry = entry?;
        let path = entry.path();
        eprintln!("deentropizing {}", path.display());
        if let Err(e) = deentropize(path) {
            eprintln!("could not deentropize: {e}");
        }
    }
    Ok(())
}

fn deentropize(path: impl AsRef<Path>) -> std::io::Result<()> {
    let mut f = File::options().read(true).write(true).open(path)?;

    f.seek(SeekFrom::Start(0))?;
    let bitcount = bitcount_file(&mut f, 2 << 16)?;

    f.seek(SeekFrom::Start(0))?;
    let params = calculate_file(bitcount);

    let mut f = BufWriter::new(f);
    write_file(params, &mut f)?;
    f.flush()?;

    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct BitCount {
    one_bits: usize,
    total_bytes: usize,
}

fn bitcount_file(mut f: impl Read, blocksize: usize) -> Result<BitCount, std::io::Error> {
    let mut one_bits = 0;
    let mut total_bytes = 0;
    let mut buf = vec![0u8; blocksize];
    loop {
        match f.read(&mut buf)? {
            0 => break,
            count => {
                total_bytes += count;
                for i in &buf[..count] {
                    one_bits += i.count_ones() as usize;
                }
            }
        }
    }

    Ok(BitCount {
        one_bits,
        total_bytes,
    })
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct LowEntropyFile {
    zero_bytes: usize,
    middle_byte: u8,
    one_bytes: usize,
}

fn calculate_file(bitcount: BitCount) -> LowEntropyFile {
    let one_bytes = bitcount.one_bits / 8;
    let middle_one_bits = bitcount.one_bits % 8;
    let zero_bytes = bitcount.total_bytes - one_bytes - 1;

    // fill lower part of the byte with ones
    let mut middle_byte = 0;
    for _ in 0..middle_one_bits {
        middle_byte <<= 1;
        middle_byte |= 1;
    }

    LowEntropyFile {
        zero_bytes,
        middle_byte,
        one_bytes,
    }
}

fn write_file(params: LowEntropyFile, mut f: impl Write) -> std::io::Result<()> {
    for _ in 0..params.zero_bytes {
        f.write(&[0x00])?;
    }
    f.write(&[params.middle_byte])?;
    for _ in 0..params.one_bytes {
        f.write(&[0xff])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use rstest::*;

    #[test]
    fn test_bitcount() {
        let data = vec![0b0010011, 0b01101011, 0b0111000];
        let result = bitcount_file(Cursor::new(data), 100).unwrap();
        assert_eq!(
            result,
            BitCount {
                one_bits: 11,
                total_bytes: 3
            }
        );
    }

    #[rstest]
    #[case(
        BitCount { one_bits: 11, total_bytes: 3 },
        LowEntropyFile { zero_bytes: 1, middle_byte: 0b0000111, one_bytes: 1 }
    )]
    #[case(
        BitCount { one_bits: 11, total_bytes: 5 },
        LowEntropyFile { zero_bytes: 3, middle_byte: 0b0000111, one_bytes: 1 }
    )]
    #[case(
        BitCount { one_bits: 8, total_bytes: 10 },
        LowEntropyFile { zero_bytes: 8, middle_byte: 0b0000000, one_bytes: 1 }
    )]
    #[case(
        BitCount { one_bits: 0, total_bytes: 4 },
        LowEntropyFile { zero_bytes: 3, middle_byte: 0b0000000, one_bytes: 0 }
    )]
    fn test_calculate_file(#[case] input: BitCount, #[case] expected: LowEntropyFile) {
        let result = calculate_file(input);
        assert_eq!(result, expected)
    }
}
