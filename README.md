# Pest based Substation Alpha parser

Rust based implementation of [subs2srs](https://subs2srs.sourceforge.net/) to generate [mass immersion]
(https://refold.la/mia/) content.

```sh
brew install ffmpeg
```

## Usage

```sh
rust2srs -s ichigo-02-jp.ass -p ichigo-02 -o test anki -t ichigo-02-en.ass
```

## Netflix subtitles

Violentmonkey plug-in that adds a `Netflix subtitle downloader` menu to the top of the screen on netflix.com.

* https://greasyfork.org/en/scripts/26654-netflix-subtitle-downloader

## Reencode

Change video resolution and audio encoding to something appropriate for a mobile device with:
```sh
ffpmeg -i input.mkv -vf scale=-1:720 -c:v libx264 -crf 18 -preset veryslow -acodec libmp3lame output.mkv
```

## Subtitles

Substation Alpha (SSA) supports formatting, animation and karaoke. V4+ (ASS) is the advanced newer version.

Add subtitles to a new stream with:
```sh
ffmpeg -i in.mkv -i subs.srt -map 0 -map 1 -c copy out.mkv
```

### Audio

Extract [audio](https://www.baeldung.com/linux/ffmpeg-audio-from-video) with:
```sh
ffmpeg -i totoro.mkv    -map 0:1 -b:a 128k -acodec libmp3lame totoro.mp3
# slow down
ffmpeg -i ichigo-02_00137.mp3 -filter:a "atempo=0.8" -vn ichigo-slow.mp3
```

### Resize

```sh
ffmpeg -i input.mkv -s 640x360 -c:a copy     output.mkv
ffmpeg -i input.mkv -s 640x360 -c:a copy -an silent.mkv # without audio
```

## Manual

This gets progressively slower as it seeks to the correct timestamp. `-y` overwrites.
```sh
ffmpeg -ss 0 -t 30 -i file.mp3 file.wav
ffmpeg -i input.mkv -ss 0:01:39.62 -frames:v 1    -y output.jpeg
ffmpeg -i input.mkv -ss 0:01:39.62 -to 0:01:41.62 -y output.mp3
```

## Reference

* https://pest.rs/book/
* https://blog.logrocket.com/building-rust-parser-pest-peg/
* https://gist.github.com/cetra3/eafaf107252d7b3845d9fd0363f08cf7
* https://blog.logrocket.com/decoding-encoding-images-rust-using-image-crate/
* https://github.com/zmwangx/rust-ffmpeg
* https://gitlab.com/anthony-tron/mp3cut
* http://www.tcax.org/docs/ass-specs.htm
* https://developer.mozilla.org/en-US/docs/Web/API/WebVTT_API
