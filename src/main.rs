use std::path::PathBuf;

use clap::{arg, Command, value_parser};

use rust2srs::{audio, parse_ssa_file, snapshot};

fn main() {
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

  let video = matches.get_one::<PathBuf>("video").unwrap();
  let source = matches.get_one::<PathBuf>("source").unwrap();
  let folder = matches.get_one::<String>("output").unwrap();
  let target = matches.get_one::<PathBuf>("target").unwrap();
  let prefix = matches.get_one::<String>("prefix").unwrap();

  let source = parse_ssa_file(source);

  for dialogue in source.into_iter() {
    let audio_file = format!("{}/{}_{}-{}.mp3", folder, prefix, dialogue.start.dot(), dialogue.end.dot());
    let status = audio(&video, &dialogue.start, &dialogue.end, audio_file).unwrap();
    assert!(status.success());

    let half = dialogue.start.half_way(&dialogue.end);
    let snapshot_file = format!("{}/{}_{}.jpg", folder, prefix, half.dot());
    let status = snapshot(&video, half, snapshot_file).unwrap();
    assert!(status.success());
  }
}
