# maxwelld

The Maxwell Daemon. It conserves bit mass while reducing the information entropy of your file.

Let's say you have an extremely high-entropy 16-byte test file. This is very high
and will contribute to the heat death of the universe.

```
$ ./make_testfile.sh 16
+ dd bs=16 if=/dev/urandom of=testfile count=1
1+0 records in
1+0 records out
16 bytes copied, 5.0155e-05 s, 319 kB/s
+ cp testfile testfile.orig
$ xxd testfile
00000000: 2b31 4543 7821 8708 4d5a 4cd2 674b 1edc  +1ECx!..MZL.gK..
```

`maxwelld` will clean up this file by moving all the ones to one end and the zeros
to the other end.

```
$ maxwelld testfile
deentropizing testfile
$ xxd testfile
00000000: 0000 0000 0000 0000 01ff ffff ffff ffff  ................
```
