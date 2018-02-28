use std::collections::HashMap;

macro_rules! map(
  { $($key:expr => $value:expr),+ } => {
    {
      let mut m = ::std::collections::HashMap::new();
        $(
          m.insert($key, $value);
        )+
      m
    }
  };
);

#[derive(Copy, Clone, Debug)]
pub enum Token {
  // operators
  Equals,
  Plus,
  Minus,
  Asterix,
  Slash,
  SemiColon,
  Colon,
  Dot,

  ParOpen,
  ParClose,

  LtOrEq,
  GtOrEq,

  // keywords
  Print,
}

use self::Token::*;

/// Find string literals
#[derive(Debug)]
pub enum PreLexed<'a> {
  String(&'a str, i32),
  Rest(&'a str, i32),
}

#[derive(Debug)]
pub enum Literal {
  String(String),
  Int(i32),
}

/// 
#[derive(Debug)]
pub enum Lexed<'a> {
  Literal(Literal, i32),
  Operator(Token, i32),
  Identifier(&'a str, i32),
}

#[derive(Debug)]
pub enum LexErr<'a> {
  MismatchedQuotes(i32),
  UnknownToken(i32, &'a str)
}

fn pre_lex(query: &str) -> Result<Vec<PreLexed>, LexErr> {
  let mut pre_lexed: Vec<PreLexed> = Vec::new();

  let mut quote_count = 0;
  let mut in_quote = false;
  let mut start = 0;
  let mut skip_next = false;

  for (i, c) in query.chars().enumerate() {
    match c {
      '\\' => {
        skip_next = true;
        continue;
      },
      '"' => {
        if skip_next {
          continue;
        }

        quote_count += 1;
        in_quote = !in_quote;

        let s = &query[start..i];
        pre_lexed.push(if in_quote {
          PreLexed::Rest(s, start as i32)
        } else {
          PreLexed::String(s, start as i32)
        });

        start = i+1;
      },
      _ => { }
    }
    skip_next = false;
  }

  if quote_count % 2 != 0 {
    return Err(LexErr::MismatchedQuotes((start - 1) as i32));
  }

  pre_lexed.push(PreLexed::Rest(&query[start..query.len()], start as i32));

  Ok(pre_lexed)
}

fn is_identifier(val: &str) -> bool {
  if val.len() <= 0 { return false; }

  match val.chars().next().unwrap() {
    'a' ... 'z' | 'A' ... 'Z' | '_' => { },
    _ => { return false; }
  }

  for i in val.chars() {
    match i {
      'a' ... 'z' | 'A' ... 'Z' | '_' | '1' ... '9' => { },
      _ => { return false; }
    }
  }

  true
}

fn is_number(val: &str) -> bool {
  if val.len() <= 0 { return false; }

  for i in val.chars() {
    match i {
      '1' ... '9' => { },
      _ => { return false; }
    }
  }

  true
}

/// Trims whitespace
/// .1 = start, .2 = end
fn trim<'a>(val: &'a str) -> (&'a str, i32, i32) {
  if val.len() <= 0 {
    return (val, 0, 0);
  }

  let mut start: i32 = 0;
  let mut end: i32 = val.len() as i32 - 1;

  for (k, v) in val.chars().enumerate() {
    match v {
      ' ' => {}
      _ => {
        start = k as i32;
        break;
      }
    }
  };

  for (k, v) in val.chars().rev().enumerate() {
    match v {
      ' ' => {}
      _ => {
        end -= k as i32 - 1;
        break;
      }
    }
  };

  // println!("{}, {}, {}, {}", val, val.len(), start, end);

  (&val[start as usize..end as usize], start, end)
}

fn tokenize(pre_lexed: Vec<PreLexed>) -> Result<Vec<Lexed>, LexErr> {

  let tokens: HashMap<&str, Token> = map!{
    "=" => Equals,
    "+" => Plus,
    "-" => Minus,
    "*" => Asterix,
    "/" => Slash,
    ";" => SemiColon,
    ":" => Colon,
    "." => Dot,

    "(" => ParOpen,
    ")" => ParClose,
    
    ">=" => GtOrEq,
    "<=" => LtOrEq,

    "print" => Print
  };

  let mut tokens: Vec<(&&str, &Token)> = tokens.iter().collect();
  tokens.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

  let longest_token = tokens[0].0.len();

  let mut lexed: Vec<Lexed> = Vec::new();
  for i in pre_lexed {
    match i {
      PreLexed::Rest(val, pos) => {
        let mut offset = 0;
        let mut r_offset: i32 = -1;
        let len = val.len();

        let mut found_op = false;

        while offset < len {
          r_offset += 1;
          found_op = false;

          if (len - r_offset as usize) - offset <= 0 {
            break;
          }

          let curr = &val[offset..len - r_offset as usize];
          let trimmed = trim(curr);
          let apos = pos + trimmed.1 + offset as i32;
          println!("curr: {}, trim: {}, off: {}, r_off: {}, lex: {}", curr, trimmed.0, offset, r_offset, lexed.len());

          if curr.len() <= 0 {
            break;
          }
        
          if curr.len() == 1 && curr.chars().next().unwrap() == ' ' {
            break;
          }

          for &i in tokens.iter() {
            println!("tokens: {} == {}", *i.0, trimmed.0);
            if *i.0 == trimmed.0 {
              lexed.push(Lexed::Operator(i.1.clone(), apos));
              r_offset = -1;
              offset += i.0.len() + trimmed.1 as usize;
              found_op = true;
              break;
            }
          }
          if found_op {
            continue;
          }

          if is_identifier(trimmed.0) {
            lexed.push(Lexed::Identifier(trimmed.0, apos));
            r_offset = -1;
            offset += trimmed.0.len() + trimmed.1 as usize;
            continue;
          }

          if is_number(trimmed.0) {
            lexed.push(Lexed::Literal(Literal::Int(trimmed.0.parse::<i32>().unwrap()), apos));
            r_offset = -1;
            offset += trimmed.0.len() + trimmed.1 as usize;
            continue;
          }

          // not found
          if trimmed.0.len() <= 1 {
            return Err(LexErr::UnknownToken(apos, trimmed.0));
          }
        }

      },
      PreLexed::String(val, pos) => {
        lexed.push(Lexed::Literal(Literal::String(String::from(val)), pos));
      },
      _ => { }
    };
  }

  Ok(lexed)
}

pub fn lex<'a>(query: &str) -> Result<Vec<Lexed<'a>>, LexErr> {
  let lexed = match pre_lex(query) {
    Ok(val) => val,
    Err(err) => {
      match err {
        LexErr::MismatchedQuotes(pos) => {
          println!("Lexer error: MismatchedQuotes");
          println!("{}", query);

          let mut offset = String::from("");
          for i in 0..pos { offset.push('-') }
          println!("{}^", offset);

          offset.clear();
          for i in 0..pos { offset.push(' ') }
          println!("{} mismatched quote", offset);
        },
        _ => {}
      }
      return Err(err);
    }
  };
  // println!("{:#?}", lexed);

  let tokenized = match tokenize(lexed) {
    Ok(val) => val,
    Err(err) => {
      match err {
        LexErr::UnknownToken(pos, token) => {
          println!("Lexer error: UnknownToken");
          println!("{}", query);

          let mut offset = String::from("");
          for i in 0..pos { offset.push('-') }
          println!("{}^", offset);

          offset.clear();
          for i in 0..pos { offset.push(' ') }
          println!("{} unknown token \"{}\"", offset, token);
        },
        _ => {}
      }
      return Err(err);
    }
  };
  println!("{:#?}", tokenized);

  // println!("{}", trim(" ="));

  Ok(Vec::new())
}