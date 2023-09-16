extern crate ffmpeg_next as ffmpeg;

use std::path::PathBuf;

use clap::{arg, Command, value_parser};

use ::rust2srs::Result;
use rust2srs::ffmpeg::extract_dialogue;
use rust2srs::parse_subtitle_file;

fn main() -> Result<()> {
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

  let video_file = matches.get_one::<String>("video").unwrap();
  let source = matches.get_one::<String>("source").unwrap();
  let folder = matches.get_one::<String>("output").unwrap();
  let _target = matches.get_one::<PathBuf>("target").unwrap();
  let prefix = matches.get_one::<String>("prefix").unwrap();
  let source = parse_subtitle_file(source).expect("Unrecognized subtitle format");

  extract_dialogue(video_file, folder, prefix, source)?;
  Ok(())
}