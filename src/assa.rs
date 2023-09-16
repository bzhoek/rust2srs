use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::{Dialogue, Time};

#[derive(Parser)]
#[grammar = "assa.pest"]
pub struct AssaParser;

pub fn parse_assa(contents: &str) -> Option<Pair<Rule>> {
  AssaParser::parse(Rule::file, contents).ok()?.next()
}

pub fn parse_assa_to_dialogue(contents: &str) -> Option<Vec<Dialogue>> {
  let file = parse_assa(contents)?;
  Some(assa_to_dialogue(file, vec![]))
}

fn assa_to_dialogue(pair: Pair<Rule>, mut list: Vec<Dialogue>) -> Vec<Dialogue> {
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
        list = assa_to_dialogue(pair, list);
      }
    }
  }
  list
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

#[allow(dead_code)]
fn dump_rules(level: usize, pair: Pair<Rule>) {
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

#[cfg(test)]
mod tests {
  use std::fs;
  use assert_matches::assert_matches;

  use crate::assa::parse_assa;
  use crate::{find_secondary_matches, parse_subtitle_file};

  use super::*;

  #[test]
  fn it_parses_substation() {
    let contents = fs::read_to_string("tests/ichigo-01.ass").unwrap();
    let file = parse_assa(&contents).unwrap();
    dump_rules(1, file.clone());
    assert_eq!(file.clone().into_inner().len(), 370);

    let dialogue = assa_to_dialogue(file.clone(), vec![]);
    assert_eq!(dialogue.len(), 350);
  }

  #[test]
  fn it_parses_substation_secondary() {
    let contents = fs::read_to_string("tests/ichigo-01_en.ass").unwrap();
    let file = parse_assa(&contents).unwrap();
    assert_eq!(file.clone().into_inner().len(), 557);

    let dialogue = assa_to_dialogue(file.clone(), vec![]);
    assert_eq!(dialogue.len(), 359);
  }

  #[test]
  fn it_matches_secondary_subtitle() {
    let primary = parse_subtitle_file("tests/ichigo-01.ass").unwrap();
    assert_eq!(primary.len(), 350);
    let first = primary.first().unwrap();
    let secondary = parse_subtitle_file("tests/ichigo-01_en.ass").unwrap();
    assert_eq!(secondary.len(), 359);
    let second = find_secondary_matches(first, &secondary);
    assert_matches!(second.first(), Some(Dialogue {text, .. }) if text == "What lovely weather.");
    let last = primary.last().unwrap();
    let second = find_secondary_matches(last, &secondary);
    assert_matches!(second.first(), Some(Dialogue {text, .. }) if text == "Uh, like what?");
  }


  #[test]
  fn it_matches_multiple_lines() {
    let primary = parse_subtitle_file("tests/ichigo-01.ass").unwrap();
    let first = primary.get(4).unwrap();
    let secondary = parse_subtitle_file("tests/ichigo-01_en.ass").unwrap();
    let second = find_secondary_matches(first, &secondary);

    assert_eq!(2, second.len());
    assert_matches!(second.get(0), Some(Dialogue {text, .. }) if text == "I'm lying. Despite my girlish looks...");
    assert_matches!(second.get(1), Some(Dialogue {text, .. }) if text == "...I'm a 20-year-old junior-college student.");
  }

  #[test]
  fn it_generates_tab_separated() {
    let primary = parse_subtitle_file("tests/ichigo-01.ass").unwrap();
    let secondary = parse_subtitle_file("tests/ichigo-01_en.ass").unwrap();
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