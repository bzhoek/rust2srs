use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::{Dialogue, Time};

#[derive(Parser)]
#[grammar = "webvtt.pest"]
pub struct WebVttParser;

impl From<Pair<'_, Rule>> for Time {
  fn from(value: Pair<Rule>) -> Self {
    let mut time = value.into_inner();
    let hour: u8 = time.next().unwrap().as_str().parse().unwrap();
    let min: u8 = time.next().unwrap().as_str().parse().unwrap();
    let sec: u8 = time.next().unwrap().as_str().parse().unwrap();
    let mil: u16 = time.next().unwrap().as_str().parse().unwrap();
    Time { hour, min, sec, mil }
  }
}

fn parse_webvtt(contents: &str) -> Option<Pair<Rule>> {
  WebVttParser::parse(Rule::file, contents).ok()?.next()
}

pub fn parse_webvtt_to_dialogue(contents: &str) -> Option<Vec<Dialogue>> {
  let file = parse_webvtt(contents)?;
  Some(webvtt_to_dialogue(file, vec![]))
}

#[allow(dead_code)]
pub fn dump_rules(level: usize, pair: Pair<Rule>) {
  for pair in pair.into_inner() {
    match pair.as_rule() {
      Rule::timestamp => {}
      _ => {
        println!("{:level$} {:?}", level, pair);
        dump_rules(level + 1, pair);
      }
    }
  }
}

fn webvtt_to_dialogue(pair: Pair<Rule>, mut list: Vec<Dialogue>) -> Vec<Dialogue> {
  for pair in pair.into_inner() {
    match pair.as_rule() {
      Rule::cue => {
        let mut inner = pair.into_inner();
        let start: Time = inner.next().unwrap().into();
        let end: Time = inner.next().unwrap().into();
        let payload = inner.next().unwrap();
        let text = payload.into_inner().next().unwrap().as_str().to_string();
        let dialogue = Dialogue { start, end, text };
        list.push(dialogue);
      }
      _ => {
        list = webvtt_to_dialogue(pair, list);
      }
    }
  }
  list
}


#[cfg(test)]
mod tests {
  use std::fs;
  use assert_matches::assert_matches;
  use crate::{find_secondary_matches, offset_subtitle_file};

  use super::*;

  #[test]
  fn it_parses_japanese_webvtt() {
    let contents = fs::read_to_string("tests/totoro.ja.vtt").unwrap();
    let file = parse_webvtt(&contents).unwrap();
    assert_eq!(843, file.clone().into_inner().len());
    let cues = webvtt_to_dialogue(file.clone(), vec![]);
    assert_eq!("<c.japanese>♪～</c.japanese>", cues.first().unwrap().text);
    assert_eq!(839, cues.len());
  }

  #[test]
  fn it_parses_english_webvtt() {
    let contents = fs::read_to_string("tests/totoro.en.vtt").unwrap();
    let file = parse_webvtt(&contents).unwrap();
    assert_eq!(619, file.clone().into_inner().len());
    let cues = webvtt_to_dialogue(file.clone(), vec![]);
    assert_eq!("Dad, do you want some candy?", cues.first().unwrap().text);
    assert_eq!(615, cues.len());
  }

  #[test]
  fn it_matches_secondary_subtitle() {
    let primary = offset_subtitle_file("tests/totoro.ja.vtt").unwrap();
    let secondary = offset_subtitle_file("tests/totoro.en.vtt").unwrap();
    assert_eq!(839, primary.len());
    assert_eq!(615, secondary.len());
    let first = primary.get(26).unwrap();
    let second = find_secondary_matches(first, &secondary);
    assert_matches!(first, Dialogue {text, .. } if text == "<c.japanese>はやく！</c.japanese>");
    assert_matches!(second.first(), Some(Dialogue {text, .. }) if text == "Come on!");
  }

  #[test]
  fn it_generates_tab_separated() {
    let primary = offset_subtitle_file("tests/totoro.ja.vtt").unwrap();
    let secondary = offset_subtitle_file("tests/totoro.en.vtt").unwrap();
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
}