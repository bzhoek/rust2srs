use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "webvtt.pest"]
pub struct WebVttParser;

fn parse_to_rules(contents: &str) -> Pair<Rule> {
  let file = WebVttParser::parse(Rule::file, contents)
    .expect("unsuccessful parse")
    .next().unwrap();
  file
}

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

pub struct Cue {
  pub text: String,
}

fn parse_to_cues(pair: Pair<Rule>, mut list: Vec<Cue>) -> Vec<Cue> {
  for pair in pair.into_inner() {
    match pair.as_rule() {
      Rule::cue_body => {
        // let inner = pair.into_inner();
        // println!("{}", inner.as_str());
        // let mut inner = pair.into_inner();
        // let text = inner.next().unwrap();
        // let string = text.as_str().to_string();
        let dialogue = Cue { text: pair.as_str().to_string() };
        list.push(dialogue);
      }
      _ => {
        list = parse_to_cues(pair, list);
      }
    }
  }
  list
}


#[cfg(test)]
mod tests {
  use std::fs;

  use super::*;

  #[test]
  fn it_parses_webvtt() {
    let contents = fs::read_to_string("tests/totoro.ja.vtt").unwrap();
    let file = parse_to_rules(&contents);
    // println!("{}", file);
    // dump_rules(1, file.clone());
    assert_eq!(840, file.clone().into_inner().len());
    let cues = parse_to_cues(file.clone(), vec![]);
    assert_eq!("<c.japanese>♪～</c.japanese>", cues.first().unwrap().text);
    assert_eq!(839, cues.len());
  }
}