use std::path::Path;
use std::process::{Command, ExitStatus};

use ffmpeg_next::codec::context;
use ffmpeg_next::Error;
use ffmpeg_next::format::{input, Pixel};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::{context::Context, flag::Flags};
use ffmpeg_next::util::frame::video::Video;
use jpeg_encoder::{ColorType, Encoder};
use log::info;

use crate::{Dialogue, sample_range, Time};
use crate::Result;

pub fn extract_screenshots(video_file: &str, folder: &str, prefix: &str, subtitles: &[Dialogue], sample: &Option<u32>)
                           -> Result<()> {
  ffmpeg_next::init().unwrap();
  info!("Extracting screenshots from {}", video_file);

  if let Ok(mut input) = input(&video_file) {
    let stream = input
      .streams()
      .best(Type::Video)
      .ok_or(Error::StreamNotFound)?;
    let context = context::Context::from_parameters(stream.parameters())?;
    let mut video = context.decoder().video()?;
    let mut scaler = create_scaler(&video)?;

    let mut dialogues = subtitles.iter();
    let mut dialogue = dialogues.next().unwrap();
    let mut half = dialogue.start.half_way(&dialogue.end);

    let (start, end) = sample_range(sample);

    let stream_index = stream.index();
    for (stream, packet) in input.packets() {
      if stream.index() != stream_index { continue; }
      video.send_packet(&packet)?;
      let mut decoded = Video::empty();
      while video.receive_frame(&mut decoded).is_ok() {
        let timestamp = decoded.timestamp().unwrap();
        let timestamp = Time::from_nanos(timestamp as u64);

        if timestamp.milliseconds() > end { return Ok(()); }

        if timestamp.milliseconds() > half.milliseconds() {
          if timestamp.milliseconds() > start {
            let snapshot_file = format!("{}/{}_{}", folder, prefix, dialogue.start.hms());
            info!("Saving {}", snapshot_file);
            let mut rgb_frame = Video::empty();
            scaler.run(&decoded, &mut rgb_frame)?;
            save_snapshot(&rgb_frame, snapshot_file.clone()).unwrap();
          }
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
  info!("Done!");
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
  use crate::offset_subtitle_file;

  use super::*;

  #[test]
  fn it_extracts_image() {
    let mut dialogue = offset_subtitle_file("tests/totoro.ja.srt", None).unwrap();
    let one = dialogue.remove(50);
    let dialogue = vec![one];
    extract_screenshots("totoro.mkv", "target", "totoro", &dialogue, &None).unwrap();
  }

  #[test]
  fn it_extracts_images() {
    let dialogue = offset_subtitle_file("tests/totoro.ja.srt", None).unwrap();
    extract_screenshots("totoro.mkv", "target", "totoro", &dialogue, &None).unwrap();
  }

  #[test]
  fn it_runs_ffmpeg() -> Result<()> {
    let start = Time { hour: 0, min: 1, sec: 39, mil: 620 };
    let end = Time { hour: 0, min: 1, sec: 41, mil: 620 };
    let half = start.half_way(&end);
    let video = "ichigo-01.mkv";

    let output = format!("test.{}-{}.mp3", start.hms(), end.hms());
    let status = audio(video.as_ref(), &start, &end, output)?;
    assert!(status.success());

    let output = format!("test.{}.jpg", half.hms());
    let status = snapshot(video.as_ref(), half, output)?;
    assert!(status.success());

    Ok(())
  }
}