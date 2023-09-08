use std::{fmt, fs};
use std::convert::Into;
use std::path::Path;
use std::process::{Command, ExitStatus};

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "ssa.pest"]
pub struct SsaParser;

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

impl Time {
  fn milliseconds(&self) -> u64 {
    ((self.hour as u64 * 60 + self.min as u64) * 60 + self.sec as u64) * 1000 + self.mil as u64
  }

  pub fn half_way(&self, later: &Time) -> Time {
    let half = (later.milliseconds() - self.milliseconds()) / 2;
    Time {
      hour: self.hour + (half / 3600_000 % 60) as u8,
      min: self.min + (half / 60_000 % 60) as u8,
      sec: self.sec + (half / 1000 % 60) as u8,
      mil: self.mil + (half % 1000) as u16,
    }
  }

  pub fn dot(&self) -> String {
    format!("{}.{:02}.{:02}.{:03}", self.hour, self.min, self.sec, self.mil)
  }

  fn colon(&self) -> String {
    format!("{}:{:02}:{:02}.{:03}", self.hour, self.min, self.sec, self.mil)
  }
}


impl From<Pair<'_, Rule>> for Time {
  fn from(value: Pair<Rule>) -> Self {
    let mut time = value.into_inner();
    let hour: u8 = time.next().unwrap().as_str().parse().unwrap();
    let min: u8 = time.next().unwrap().as_str().parse().unwrap();
    let sec: u8 = time.next().unwrap().as_str().parse().unwrap();
    let hun: u16 = time.next().unwrap().as_str().parse().unwrap();
    Time { hour, min, sec, mil: hun * 10 }
  }
}

#[derive(Debug)]
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

pub fn parse_ssa_file(path: &Path) -> Vec<Dialogue> {
  let contents = fs::read_to_string(path)
    .expect("cannot read file");
  let file = SsaParser::parse(Rule::file, &contents)
    .expect("unsuccessful parse")
    .next().unwrap();
  parse_rules(file, vec![])
}

fn parse_rules(pair: Pair<Rule>, mut list: Vec<Dialogue>) -> Vec<Dialogue> {
  for pair in pair.into_inner() {
    match pair.as_rule() {
      Rule::dialogue => {
        let mut inner = pair.into_inner();
        let _layer = inner.next().unwrap();
        let start: Time = inner.next().unwrap().into();
        let end: Time = inner.next().unwrap().into();
        let _style = inner.next().unwrap();
        let _name = inner.next().unwrap();
        let _margin_l = inner.next().unwrap();
        let _margin_r = inner.next().unwrap();
        let _margin_v = inner.next().unwrap();
        let _effect = inner.next().unwrap();
        let text = inner.next().unwrap();

        let dialogue = Dialogue { start, end, text: text.as_str().to_string() };
        list.push(dialogue);
      }
      _ => {
        list = parse_rules(pair, list);
      }
    }
  }
  list
}

pub fn dump_rules(level: usize, pair: Pair<Rule>) {
  for pair in pair.into_inner() {
    match pair.as_rule() {
      Rule::time => {}
      _ => {
        println!("{:level$} {:?}", level, pair);
        dump_rules(level + 1, pair);
      }
    }
  }
}

pub fn snapshot(video: &Path, time: Time, output: String) -> std::io::Result<ExitStatus> {
  Command::new("ffmpeg")
    .arg("-i")
    .arg(video)
    .arg("-ss")
    .arg(time.colon())
    .arg("-frames:v")
    .arg("1")
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
  use std::error;

  use crate::{dump_rules, parse_rules, Time};

  use super::*;

  #[test]
  fn it_parses_substation() {
    let contents = fs::read_to_string("tests/ichigo-01.ass")
      .expect("cannot read file");
    let file = SsaParser::parse(Rule::file, &contents)
      .expect("unsuccessful parse")
      .next().unwrap();
    dump_rules(1, file.clone());
    assert_eq!(file.clone().into_inner().len(), 370);

    let dialogue = parse_rules(file.clone(), vec![]);
    assert_eq!(dialogue.len(), 351);
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

  type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

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