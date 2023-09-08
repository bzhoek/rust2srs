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
      arg!(-p --prefix <TEXT> "Prefix for output files")
        .required(true)
    )
    .get_matches();

  let video = matches.get_one::<PathBuf>("video").unwrap();
  let source = matches.get_one::<PathBuf>("source").unwrap();
  let target = matches.get_one::<PathBuf>("target").unwrap();
  let prefix = matches.get_one::<String>("prefix").unwrap();

  let source = parse_ssa_file(source);
  let first = source.first().unwrap();
  let output = format!("{}_{}-{}.mp3", prefix, first.start.dot(), first.end.dot());
  let status = audio(&video, &first.start, &first.end, output).unwrap();
  assert!(status.success());

  let half = first.start.half_way(&first.end);
  let output = format!("{}_{}.jpg", prefix, half.dot());
  let status = snapshot(&video, half, output).unwrap();
  assert!(status.success());

  println!("{:?} {:?} {:?} {}", video, first, target, prefix)
}
