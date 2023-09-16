use std::{error, fmt, fs};
use std::cmp::Ordering;
use std::path::Path;
use std::process::{Command, ExitStatus};
use crate::assa::parse_assa_to_dialogue;

use crate::webvtt::parse_webvtt_to_dialogue;

mod assa;
mod mp3;
mod webvtt;

use std::{error, fmt, fs};
use std::cmp::Ordering;
use std::convert::Into;
use std::path::Path;
use std::process::{Command, ExitStatus};

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Parser)]
#[grammar = "assa.pest"]
pub struct AssaParser;

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

pub fn parse_ssa_file(path: &Path) -> Vec<Dialogue> {
  let contents = fs::read_to_string(path)
    .expect("cannot read file");
  let file = parse_to_rules(&contents);
  parse_to_dialogue(file, vec![])
}

fn parse_to_rules(contents: &str) -> Pair<Rule> {
  let file = AssaParser::parse(Rule::file, contents)
    .expect("unsuccessful parse")
    .next().unwrap();
  file
}

fn parse_to_dialogue(pair: Pair<Rule>, mut list: Vec<Dialogue>) -> Vec<Dialogue> {
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
        let string = text.as_str().to_string();
        if !string.starts_with("{\\") {
          let dialogue = Dialogue { start, end, text: string };
          list.push(dialogue);
        }
      }
      _ => {
        list = parse_to_dialogue(pair, list);
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
  use assert_matches::assert_matches;

  use crate::{parse_to_dialogue, Time};

  use super::*;

  #[test]
  fn it_parses_substation() {
    let contents = fs::read_to_string("tests/ichigo-01.ass").unwrap();
    let file = parse_to_rules(&contents);
    dump_rules(1, file.clone());
    assert_eq!(file.clone().into_inner().len(), 370);

    let dialogue = parse_to_dialogue(file.clone(), vec![]);
    assert_eq!(dialogue.len(), 350);
  }

  #[test]
  fn it_parses_substation_secondary() {
    let contents = fs::read_to_string("tests/ichigo-01_en.ass").unwrap();
    let file = parse_to_rules(&contents);
    assert_eq!(file.clone().into_inner().len(), 557);

    let dialogue = parse_to_dialogue(file.clone(), vec![]);
    assert_eq!(dialogue.len(), 359);
  }

  fn get_dialogue(path: &str) -> Vec<Dialogue> {
    let contents = fs::read_to_string(path).unwrap();
    let rules = parse_to_rules(&contents);
    let dialogue = parse_to_dialogue(rules, vec![]);
    dialogue
  }

  #[test]
  fn it_matches_secondary_subtitle() {
    let primary = get_dialogue("tests/ichigo-01.ass");
    assert_eq!(primary.len(), 350);
    let first = primary.first().unwrap();
    let secondary = get_dialogue("tests/ichigo-01_en.ass");
    assert_eq!(secondary.len(), 359);
    let second = find_secondary_matches(first, &secondary);
    assert_matches!(second.first(), Some(Dialogue {text, .. }) if text == "What lovely weather.");
    let last = primary.last().unwrap();
    let second = find_secondary_matches(last, &secondary);
    assert_matches!(second.first(), Some(Dialogue {text, .. }) if text == "Uh, like what?");
  }

  #[test]
  fn it_matches_multiple_lines() {
    let primary = get_dialogue("tests/ichigo-01.ass");
    let first = primary.get(4).unwrap();
    let secondary = get_dialogue("tests/ichigo-01_en.ass");
    let second = find_secondary_matches(first, &secondary);

    assert_eq!(2, second.len());
    assert_matches!(second.get(0), Some(Dialogue {text, .. }) if text == "I'm lying. Despite my girlish looks...");
    assert_matches!(second.get(1), Some(Dialogue {text, .. }) if text == "...I'm a 20-year-old junior-college student.");
  }

  #[test]
  fn it_generates_tab_separated() {
    let primary = get_dialogue("tests/ichigo-01.ass");
    let secondary = get_dialogue("tests/ichigo-01_en.ass");
    for first in primary.iter() {
      let second = find_secondary_matches(first, &secondary);
      let text = first.text
        .replace("\\N", " ")
        .replace("\\n", " ");
      let second: String = second.iter().map(|d| d.text.clone()).collect::<Vec<_>>().join(" ")
        .replace("\\N", " ")
        .replace("\\n", " ");
      println!("{}\t{}", text, second);
    }
  }

  fn find_secondary_matches<'a>(dialogue: &'a Dialogue, secondary: &'a Vec<Dialogue>) ->
  Vec<&'a Dialogue> {
    secondary
      .iter().filter(
      |second| second.start >= dialogue.start && second.start < dialogue.end)
      .collect()
  }

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