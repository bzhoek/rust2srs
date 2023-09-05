use pest::iterators::Pair;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "ass.pest"]
pub struct AssParser;

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

  use crate::{AssParser, dump_rules, Rule};

  #[test]
  fn it_parses_substation() {
    let contents = fs::read_to_string("tests/ichigo-01.ass").expect("cannot read file");
    let file = AssParser::parse(Rule::file, &contents)
      .expect("unsuccessful parse")
      .next().unwrap();

    dump_rules(1, file.clone());
    assert_eq!(file.into_inner().len(), 370);
  }
}