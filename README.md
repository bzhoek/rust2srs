# Pest based Substation Alpha parser

* https://pest.rs/book/
* https://blog.logrocket.com/building-rust-parser-pest-peg/
* https://gist.github.com/cetra3/eafaf107252d7b3845d9fd0363f08cf7
* https://blog.logrocket.com/decoding-encoding-images-rust-using-image-crate/

## fmpeg
`-y` overwrite.

```sh
ffmpeg -ss 0 -t 30 -i file.mp3 file.wav
ffmpeg -i ichigo-01.mkv -ss 0:01:39.62 -frames:v 1 -y output.jpeg
ffmpeg -i ichigo-01.mkv -ss 0:01:39.62 -to 0:01:41.62 -y output.mp3


Frame 34815
target/debug/rust2srs -v ichigo-01.mkv -s tests/ichigo-01.ass -t  -p ichigo-1  158.03s user 1.00s system 99% cpu 2:39.92 total

Frame 351
target/debug/rust2srs -v ichigo-01.mkv -s tests/ichigo-01.ass -t  -p ichigo-1  6.77s user 0.07s system 96% cpu 7.096 total
```