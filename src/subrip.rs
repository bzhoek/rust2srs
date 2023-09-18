use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::{Dialogue, Time};

#[derive(Parser)]
#[grammar = "subrip.pest"]
pub struct SubripParser;

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

fn parse_subrip(contents: &str) -> Option<Pair<Rule>> {
  SubripParser::parse(Rule::file, contents).ok()?.next()
}

pub fn parse_subrip_to_dialogue(contents: &str) -> Option<Vec<Dialogue>> {
  let file = parse_subrip(contents)?;
  Some(subrip_to_dialogue(file, vec![]))
}

fn subrip_to_dialogue(pair: Pair<Rule>, mut list: Vec<Dialogue>) -> Vec<Dialogue> {
  for pair in pair.into_inner() {
    match pair.as_rule() {
      Rule::cue => {
        let mut inner = pair.into_inner();
        let start: Time = inner.next().unwrap().into();
        let end: Time = inner.next().unwrap().into();
        let payload = inner.next().unwrap();
        let text = payload.into_inner().next().unwrap().as_str()
          .to_string().replace("\n", "<br/>");
        let dialogue = Dialogue { start, end, text };
        list.push(dialogue);
      }
      _ => {
        list = subrip_to_dialogue(pair, list);
      }
    }
  }
  list
}

#[cfg(test)]
mod tests {
  use std::fs;
  use assert_matches::assert_matches;

  use super::*;

  #[test]
  fn it_parses_japanese_subrip() {
    let contents = fs::read_to_string("tests/totoro.ja.srt").unwrap();
    let file = parse_subrip(&contents).unwrap();
    assert_eq!(845, file.clone().into_inner().len());
    let mut subtitles = subrip_to_dialogue(file.clone(), vec![]);
    assert_eq!(844, subtitles.len());
    let dialogue = subtitles.remove(50);
    assert_matches!(dialogue, Dialogue {text, .. } if text == "早く～！");
  }
}