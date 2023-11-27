#![feature(vec_into_raw_parts)]
use std::fs;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result {
  let src = fs::read_to_string("test.asm")?;
  let tokens = parse(&src)?;
  println!("{:?}", tokens);
  let out = assemble(tokens)?;
  println!("{:?}", out);
  Ok(())
}

fn parse(src: &str) -> Result<Vec<Token>> {
  let mut tokens = vec![];
  for line in src.lines() {
    for s in line.split_whitespace() {
      tokens.push(Token::parse(s)?);
    }
  }
  tokens.push(Token::EOF);
  Ok(tokens)
}

fn assemble(tokens: Vec<Token>) -> Result<Vec<u8>> {
  let mut labels = HashMap::new();
  let mut out = vec![];
  let mut tokens = tokens.iter();
  while let Some(t) = tokens.next() {
    match t {
      Token::Label(s) => match labels.entry(s) {
        Entry::Occupied(_) => return Err(format!("label already declared '{}'", s).into()),
        Entry::Vacant(e) => {
          e.insert(out.len());
        }
      },
      Token::Instruction(i) => match i {
        Instruction::Add => match tokens.next().unwrap_or(&Token::EOF) {
          Token::Reg(r) => match tokens.next().unwrap_or(&Token::EOF) {
            Token::Reg(r2) => out.push((OpCode::AddR, *r as u8, *r2 as u8)),
            Token::Const(c) => out.push((OpCode::AddC, *r as u8, *c as u8)),
            _ => return Err(format!("expected register/const, found {:?}", t).into()),
          },
          t => return Err(format!("expected register, found {:?}", t).into()),
        },
        Instruction::Hlt => out.push((OpCode::Hlt, 0, 0)),
      },
      Token::EOF => {}
      _ => return Err(format!("expected label/instruction, found {:?}", t).into()),
    };
  }
  println!("{:?}", out);
  let (ptr, len, cap) = out.into_raw_parts();
  Ok(unsafe { Vec::from_raw_parts(ptr as _, len * 4, cap * 4) })
}

#[derive(Debug)]
enum Token<'a> {
  Label(&'a str),
  Reg(Register),
  Const(u16),
  Instruction(Instruction),
  EOF,
}

impl<'a> Token<'a> {
  fn parse(s: &'a str) -> Result<Self> {
    let mut chars = s.chars().peekable();
    match chars.peek().unwrap() {
      '.' => Ok(Self::Label(&s[1..])),
      '#' => Ok(Self::Reg(Register::parse(&s[1..])?)),
      c if c.is_digit(10) => match chars.next().unwrap() {
        '0' => match chars.next() {
          Some('x') => Self::const_from_radix(s, 16),
          Some('o') => Self::const_from_radix(s, 8),
          Some('b') => Self::const_from_radix(s, 2),
          Some(c) if c.is_digit(10) => Ok(Self::Const(s.parse()?)),
          Some(b) => Err(format!("unknown base '{}'", b).into()),
          None => Ok(Self::Const(0)),
        },
        _ => Ok(Self::Const(s.parse()?)),
      },
      _ => Ok(Self::Instruction(Instruction::parse(s)?)),
    }
  }

  fn const_from_radix(s: &str, radix: u32) -> Result<Self> {
    Ok(Self::Const(u16::from_str_radix(&s[2..], radix)?))
  }
}

#[rustfmt::skip]
#[derive(Copy, Clone, Debug)]
enum Register {
  A, B, C, D, E, F, G, H
}

impl Register {
  fn parse(s: &str) -> Result<Self> {
    match &*s.to_uppercase() {
      "A" => Ok(Self::A),
      "B" => Ok(Self::B),
      "C" => Ok(Self::C),
      "D" => Ok(Self::D),
      "E" => Ok(Self::E),
      "F" => Ok(Self::F),
      "G" => Ok(Self::G),
      "H" => Ok(Self::H),
      _ => Err(format!("unknown register '{}'", s).into()),
    }
  }
}

#[derive(Debug)]
enum Instruction {
  Hlt,
  Add,
}

impl Instruction {
  fn parse(s: &str) -> Result<Self> {
    match &*s.to_uppercase() {
      "HLT" => Ok(Self::Hlt),
      "ADD" => Ok(Self::Add),
      _ => Err(format!("unknown instruction '{}'", s).into()),
    }
  }
}

#[derive(Copy, Clone, Debug)]
enum OpCode {
  Hlt,
  AddR,
  AddC,
}
