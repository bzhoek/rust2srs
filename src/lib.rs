use std::{error, fmt, fs};
use std::cmp::Ordering;

use crate::assa::parse_assa_to_dialogue;
use crate::subrip::parse_subrip_to_dialogue;
use crate::webvtt::parse_webvtt_to_dialogue;

mod assa;
mod mp3;
mod subrip;
mod webvtt;
pub mod ffmpeg;

pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, PartialEq)]
pub struct Dialogue {
  pub start: Time,
  pub end: Time,
  pub text: String,
}

impl fmt::Display for Dialogue {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} - {}: {}", self.start, self.end, self.text)
  }
}

#[derive(Debug, PartialEq)]
pub struct Time {
  hour: u8,
  min: u8,
  sec: u8,
  mil: u16,
}

impl fmt::Display for Time {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}.{:02}.{:02}.{:03}", self.hour, self.min, self.sec, self.mil)
  }
}

impl PartialOrd for Time {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.milliseconds().cmp(&other.milliseconds()))
  }
}

impl Time {
  pub fn from_nanos(nanos: u64) -> Time {
    Time {
      hour: (nanos / 3_600_000 % 60) as u8,
      min: (nanos / 60_000 % 60) as u8,
      sec: (nanos / 1000 % 60) as u8,
      mil: (nanos % 1000) as u16,
    }
  }

  pub fn milliseconds(&self) -> u64 {
    ((self.hour as u64 * 60 + self.min as u64) * 60 + self.sec as u64) * 1000 + self.mil as u64
  }

  pub fn half_way(&self, later: &Time) -> Time {
    let half = self.milliseconds() + (later.milliseconds() - self.milliseconds()) / 2;
    Time::from_nanos(half)
  }

  pub fn dot(&self) -> String {
    format!("{}.{:02}.{:02}.{:03}", self.hour, self.min, self.sec, self.mil)
  }

  fn colon(&self) -> String {
    format!("{}:{:02}:{:02}.{:03}", self.hour, self.min, self.sec, self.mil)
  }
}

pub fn parse_subtitle_file(path: &str) -> Option<Vec<Dialogue>> {
  let contents = fs::read_to_string(path)
    .expect("cannot read file");
  if let Some(dialogue) = parse_assa_to_dialogue(&contents) {
    return Some(dialogue);
  }
  if let Some(dialogue) = parse_webvtt_to_dialogue(&contents) {
    return Some(dialogue);
  }
  if let Some(dialogue) = parse_subrip_to_dialogue(&contents) {
    return Some(dialogue);
  }
  None
}

pub fn find_secondary_matches<'a>(dialogue: &'a Dialogue, secondary: &'a Vec<Dialogue>) ->
Vec<&'a Dialogue> {
  secondary
    .iter().filter(
    |second| second.start >= dialogue.start && second.start < dialogue.end)
    .collect()
}

#[cfg(test)]
mod tests {
  use crate::ffmpeg::{audio, snapshot};

  use super::*;

  #[test]
  fn it_halves_properly() {
    let start = Time { hour: 0, min: 1, sec: 59, mil: 10 };
    let end = Time { hour: 0, min: 2, sec: 2, mil: 50 };
    let half = start.half_way(&end);
    assert_eq!(Time { hour: 0, min: 2, sec: 0, mil: 530 }, half);
  }

  #[test]
  fn it_halves_first_duration() {
    // ichigo-1_1_0.01.39.620-0.01.41.620.mp3
    // ichigo-1_1_0.01.40.620.jpg
    let start = Time { hour: 0, min: 1, sec: 39, mil: 620 };
    let end = Time { hour: 0, min: 1, sec: 41, mil: 620 };
    let diff = end.milliseconds() - start.milliseconds();
    assert_eq!(2000, diff);
    let result = start.half_way(&end);
    assert_eq!(Time { hour: 0, min: 1, sec: 40, mil: 620 }, result)
  }

  #[test]
  fn it_halves_last_duration() {
    // ichigo-1_1_0.24.03.080-0.24.04.250.mp3
    // ichigo-1_1_0.24.03.665.jpg
    let start = Time { hour: 0, min: 24, sec: 3, mil: 80 };
    let end = Time { hour: 0, min: 24, sec: 4, mil: 250 };
    let result = start.half_way(&end);
    assert_eq!(Time { hour: 0, min: 24, sec: 3, mil: 665 }, result);
    assert_eq!("0.24.03.665", format!("{}", result))
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

  #[test]
  fn it_converts_nanos() {
    let time = Time::from_nanos(1451951);
    assert_eq!("0.24.11.951", format!("{}", time))
  }
}