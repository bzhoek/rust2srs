use std::path::Path;
use std::process::{Command, ExitStatus};

use ffmpeg_next::codec::context;
use ffmpeg_next::Error;
use ffmpeg_next::format::{input, Pixel};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::{context::Context, flag::Flags};
use ffmpeg_next::util::frame::video::Video;
use jpeg_encoder::{ColorType, Encoder};

use crate::{Dialogue, Time};
use crate::Result;

pub fn extract_dialogue(video_file: &str, folder: &str, prefix: &str, source: Vec<Dialogue>)
                        -> Result<()> {
  ffmpeg_next::init().unwrap();
  if let Ok(mut input) = input(&video_file) {
    let stream = input
      .streams()
      .best(Type::Video)
      .ok_or(Error::StreamNotFound)?;
    let context = context::Context::from_parameters(stream.parameters())?;
    let mut video = context.decoder().video()?;
    let mut scaler = create_scaler(&video)?;

    let mut dialogues = source.into_iter();
    let mut dialogue = dialogues.next().unwrap();
    let mut half = dialogue.start.half_way(&dialogue.end);

    let stream_index = stream.index();
    for (stream, packet) in input.packets() {
      if stream.index() != stream_index { continue; }
      video.send_packet(&packet)?;
      let mut decoded = Video::empty();
      while video.receive_frame(&mut decoded).is_ok() {
        let timestamp = decoded.timestamp().unwrap();
        let timestamp = Time::from_nanos(timestamp as u64);
        if timestamp.milliseconds() > half.milliseconds() {
          let snapshot_file = format!("{}/{}_{}", folder, prefix, half.dot());
          println!("Saving {}", snapshot_file);
          let mut rgb_frame = Video::empty();
          scaler.run(&decoded, &mut rgb_frame)?;
          save_snapshot(&rgb_frame, snapshot_file.clone()).unwrap();
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
  Ok(())
}

fn create_scaler(video: &ffmpeg_next::decoder::Video) -> Result<Context> {
  let scaler = Context::get(
    video.format(),
    video.width(),
    video.height(),
    Pixel::RGB24,
    video.width(),
    video.height(),
    Flags::BILINEAR,
  )?;
  Ok(scaler)
}

fn save_snapshot(frame: &Video, filename: String) -> Result<()> {
  let encoder = Encoder::new_file(format!("{}.jpg", filename), 65)?;
  encoder.encode(
    frame.data(0),
    frame.width() as u16,
    frame.height() as u16,
    ColorType::Rgb)?;
  Ok(())
}

pub fn snapshot(video: &Path, time: Time, output: String) -> std::io::Result<ExitStatus> {
  Command::new("ffmpeg")
    .arg("-i")
    .arg(video)
    .arg("-ss")
    .arg(time.colon())
    .arg("-frames:v")
    .arg("1")
    .arg("-loglevel")
    .arg("error")
    .arg("-y")
    .arg(output)
    .status()
}

pub fn audio(video: &Path, start: &Time, end: &Time, output: String) -> std::io::Result<ExitStatus> {
  Command::new("ffmpeg")
    .arg("-i")
    .arg(video)
    .arg("-ss")
    .arg(start.colon())
    .arg("-to")
    .arg(end.colon())
    .arg("-y")
    .arg(output)
    .status()
}


#[cfg(test)]
mod tests {
  use crate::parse_subtitle_file;

  use super::*;

  #[test]
  fn it_extracts_image() {
    let mut dialogue = parse_subtitle_file("tests/totoro.ja.vtt").unwrap();
    assert_eq!(839, dialogue.len());
    let one = dialogue.remove(26);
    let dialogue = vec![one];
    extract_dialogue("totoro.mkv", "target", "totoro", dialogue).unwrap();
  }

  #[test]
  fn it_extracts_images() {
    let dialogue = parse_subtitle_file("tests/totoro.ja.srt").unwrap();
    extract_dialogue("totoro.mkv", "target", "totoro", dialogue).unwrap();
  }

  #[test]
  fn it_runs_ffmpeg() -> Result<()> {
    let start = Time { hour: 0, min: 1, sec: 39, mil: 620 };
    let end = Time { hour: 0, min: 1, sec: 41, mil: 620 };
    let half = start.half_way(&end);
    let video = "ichigo-01.mkv";

    let output = format!("test.{}-{}.mp3", start.dot(), end.dot());
    let status = audio(video.as_ref(), &start, &end, output)?;
    assert!(status.success());

    let output = format!("test.{}.jpg", half.dot());
    let status = snapshot(video.as_ref(), half, output)?;
    assert!(status.success());

    Ok(())
  }
}