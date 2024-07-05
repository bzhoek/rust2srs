extern crate ffmpeg_next as ffmpeg;

use clap::{Parser, Subcommand};
use clap::arg;
use env_logger::Env;
use env_logger::Target::Stdout;
use log::debug;

use ::rust2srs::Result;
use rust2srs::ffmpeg::extract_screenshots;
use rust2srs::mp3::{AudioSuffix, extract_sound_clips};
use rust2srs::{generate_tab_separated, offset_subtitle_file};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
  /// Source language subtitles
  #[arg(short, long)]
  source: String,

  /// Offset
  #[arg(long)]
  offset: Option<f32>,

  /// Output folder
  #[arg(short, long)]
  output: String,

  /// Prefix folder
  #[arg(short, long)]
  prefix: String,

  /// Sample 5 minutes from this minute
  #[arg(long)]
  sample: Option<u32>,

  /// Verbose logging
  #[arg(short, long, default_value = "false")]
  verbose: bool,

  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  Video {
    /// Media file
    #[arg(short, long)]
    video: String,
  },
  Audio {
    /// Media file
    #[arg(short, long)]
    audio: String,
  },
  Anki {
    /// Source language subtitles
    #[arg(short, long)]
    target: String,

    /// Offset
    #[arg(long)]
    offset: Option<f32>,
  },
}

fn main() -> Result<()> {
  let args = Cli::parse();
  let level = if args.verbose { "debug" } else { "info" };
  env_logger::Builder::from_env(
    Env::default().default_filter_or(level)
  ).target(Stdout).init();
  debug!("Verbose logging");

  let source = offset_subtitle_file(&args.source, &args.offset).expect("Unrecognized subtitle format");
  match args.command {
    Commands::Video { video } => {
      extract_screenshots(&video, &args.output, &args.prefix, &source, &args.sample)?;
    }
    Commands::Audio { audio } => {
      extract_sound_clips(&audio, &args.output, &args.prefix, &source, AudioSuffix::None, args.sample)?;
    }
    Commands::Anki { target, offset } => {
      let target = offset_subtitle_file(&target, &offset).expect("Unrecognized subtitle format");
      generate_tab_separated(source, target, &args.output, &args.prefix, AudioSuffix::None);
    }
  }
  Ok(())
}