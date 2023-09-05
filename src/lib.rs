use pest::iterators::Pair;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "ass.pest"]
pub struct AssParser;

#[derive(Debug)]
pub struct Dialogue {
  start: String,
  end: String,
  text: String,
}

pub fn parse_rules(pair: Pair<Rule>, mut list: Vec<Dialogue>) -> Vec<Dialogue> {
  for pair in pair.into_inner() {
    match pair.as_rule() {
      Rule::dialogue => {
        let mut inner = pair.into_inner();
        let _layer = inner.next().unwrap();
        let start = inner.next().unwrap();
        let end = inner.next().unwrap();
        let _style = inner.next().unwrap();
        let _name = inner.next().unwrap();
        let _margin_l = inner.next().unwrap();
        let _margin_r = inner.next().unwrap();
        let _margin_v = inner.next().unwrap();
        let _effect = inner.next().unwrap();
        let text = inner.next().unwrap();

        let dialogue = Dialogue { start: start.to_string(), end: end.to_string(), text: text.to_string() };
        println!("Dialogue: {:?}", dialogue);
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