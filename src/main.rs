extern crate ffmpeg_next as ffmpeg;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use clap::{arg, Command, value_parser};
use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;

use rust2srs::{parse_ssa_file, Time};

fn main() -> Result<(), ffmpeg::Error> {
  let matches = Command::new("rust2srs")
    .arg(
      arg!(-v --video <FILE> "Video file")
        .required(true)
        .value_parser(value_parser!(PathBuf)),
    )
    .arg(
      arg!(-s --source <FILE> "Subtitle in source language")
        .required(true)
        .value_parser(value_parser!(PathBuf)),
    )
    .arg(
      arg!(-t --target <FILE> "Subtitle in target language")
        .required(true)
        .value_parser(value_parser!(PathBuf)),
    )
    .arg(
      arg!(-o --output <FOLDER> "Folder to save output to")
        .required(true)
    )
    .arg(
      arg!(-p --prefix <TEXT> "Prefix for output files")
        .required(true)
    )
    .get_matches();

  let video_file = matches.get_one::<PathBuf>("video").unwrap();
  let source = matches.get_one::<PathBuf>("source").unwrap();
  let folder = matches.get_one::<String>("output").unwrap();
  let target = matches.get_one::<PathBuf>("target").unwrap();
  let prefix = matches.get_one::<String>("prefix").unwrap();
  let source = parse_ssa_file(source);

  ffmpeg::init().unwrap();
  if let Ok(mut input) = input(&video_file) {
    let stream = input
      .streams()
      .best(Type::Video)
      .ok_or(ffmpeg::Error::StreamNotFound)?;
    let stream_index = stream.index();
    let context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
    let mut video = context.decoder().video()?;

    let mut scaler = Context::get(
      video.format(),
      video.width(),
      video.height(),
      Pixel::RGB24,
      video.width(),
      video.height(),
      Flags::BILINEAR,
    )?;

    let mut dialogues = source.into_iter();
    let mut dialogue = dialogues.next().unwrap();
    let mut half = dialogue.start.half_way(&dialogue.end);

    for (stream, packet) in input.packets() {
      if stream.index() == stream_index {
        video.send_packet(&packet)?;
        let mut decoded = Video::empty();
        while video.receive_frame(&mut decoded).is_ok() {
          let timestamp = decoded.timestamp().unwrap();
          let timestamp = Time::from_nanos(timestamp as u64);
          if timestamp.milliseconds() > half.milliseconds() {
            println!("Saving {:?}", timestamp);
            let snapshot_file = format!("{}/{}_{}.jpg", folder, prefix, half.dot());
            let mut rgb_frame = Video::empty();
            scaler.run(&decoded, &mut rgb_frame)?;
            save_file(&rgb_frame, snapshot_file.clone()).unwrap();
            match dialogues.next() {
              None => { return Ok(()); }
              Some(next) => {
                dialogue = next;
                half = dialogue.start.half_way(&dialogue.end);
              }
            }
          }
        }
      }
    }
  }

  Ok(())
}

fn save_file(frame: &Video, filename: String) -> Result<(), std::io::Error> {
  let mut file = File::create(filename)?;
  file.write_all(format!("P6\n{} {}\n255\n", frame.width(), frame.height()).as_bytes())?;
  file.write_all(frame.data(0))?;
  Ok(())
}
