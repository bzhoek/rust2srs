use std::convert::Into;
use std::fmt;

use pest::iterators::Pair;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "ass.pest"]
pub struct AssParser;

#[derive(Debug)]
pub struct Time {
  hour: u8,
  min: u8,
  sec: u8,
  hun: u8,
}

impl fmt::Display for Time {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}:{:02}:{:02}.{:02}", self.hour, self.min, self.sec, self.hun)
  }
}

impl From<Pair<'_, Rule>> for Time {
  fn from(value: Pair<Rule>) -> Self {
    let mut time = value.into_inner();
    let hour: u8 = time.next().unwrap().as_str().parse().unwrap();
    let min: u8 = time.next().unwrap().as_str().parse().unwrap();
    let sec: u8 = time.next().unwrap().as_str().parse().unwrap();
    let hun: u8 = time.next().unwrap().as_str().parse().unwrap();
    Time { hour, min, sec, hun }
  }
}

#[derive(Debug)]
pub struct Dialogue {
  start: Time,
  end: Time,
  text: String,
}

impl fmt::Display for Dialogue {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} - {}: {}", self.start, self.end, self.text)
  }
}

pub fn parse_rules(pair: Pair<Rule>, mut list: Vec<Dialogue>) -> Vec<Dialogue> {
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

        let dialogue = Dialogue { start: start, end: end, text: text.as_str().to_string() };
        println!("Dialogue: {:}", dialogue);
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

  use pest::Parser;

  use crate::{AssParser, dump_rules, parse_rules, Rule};

  #[test]
  fn it_parses_substation() {
    let contents = fs::read_to_string("tests/ichigo-01.ass").expect("cannot read file");
    let file = AssParser::parse(Rule::file, &contents)
      .expect("unsuccessful parse")
      .next().unwrap();

    dump_rules(1, file.clone());
    assert_eq!(file.clone().into_inner().len(), 370);

    let dialogue = parse_rules(file.clone(), vec![]);
    assert_eq!(dialogue.len(), 351);
  }
}