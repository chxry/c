use std::fs;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::vec::IntoIter;
use shared::{Result, Reg, OpCode, AddrMode};

fn main() -> Result {
  let src = fs::read_to_string("test.asm")?;
  let tokens = parse(&src)?;
  println!("{:?}", tokens);
  let out = assemble(tokens)?;
  println!("{:?}", out);
  fs::write("out.o", out)?;
  Ok(())
}

fn parse(src: &str) -> Result<Vec<Token>> {
  let mut tokens = vec![];
  for line in src.lines() {
    if !line.starts_with(';') {
      for s in line.split_whitespace() {
        tokens.push(Token::parse(s)?);
      }
    }
  }
  tokens.push(Token::Eof);
  Ok(tokens)
}

fn assemble(tokens: Vec<Token>) -> Result<Vec<u8>> {
  let mut labels = HashMap::new();
  let mut out = vec![];
  let mut tokens = tokens.into_iter();
  while let Some(t) = tokens.next() {
    match t {
      Token::Label(s) => match labels.entry(s) {
        Entry::Occupied(_) => return Err(format!("label already declared '{}'", s).into()),
        Entry::Vacant(e) => {
          e.insert(out.len() as _);
        }
      },
      Token::Instruction(i) => {
        out.push(i.to());
        match i {
          OpCode::Hlt => {}
          OpCode::Jmp => {
            op_any(&mut tokens, &labels, &mut out)?;
          }
          OpCode::Add | OpCode::Mov => {
            op_any(&mut tokens, &labels, &mut out)?;
            op_reg(&mut tokens, &mut out)?;
          }
        }
      }
      Token::Eof => {}
      _ => return Err(format!("expected label/instruction, found {:?}", t).into()),
    };
  }
  Ok(out)
}

fn op_reg(tokens: &mut IntoIter<Token>, out: &mut Vec<u8>) -> Result {
  match tokens.next().unwrap_or(Token::Eof) {
    Token::Reg(r) => out.push(r.to()),
    t => return Err(format!("expected register, found {:?}", t).into()),
  };
  Ok(())
}

fn op_any(tokens: &mut IntoIter<Token>, labels: &HashMap<&str, u16>, out: &mut Vec<u8>) -> Result {
  match tokens.next().unwrap_or(Token::Eof) {
    Token::Reg(r) => out.extend([AddrMode::Reg.to(), r.to()]),
    Token::Const(c) => {
      out.push(AddrMode::Const.to());
      out.extend(c.to_le_bytes())
    }
    Token::Label(l) => match labels.get(l) {
      Some(i) => {
        out.push(AddrMode::Const.to());
        out.extend(i.to_le_bytes())
      }
      None => return Err(format!("unknown label '{}'", l).into()),
    },
    Token::Deref(t) => match *t {
      Token::Reg(r) => out.extend([AddrMode::DerefReg.to(), r.to()]),
      Token::Const(c) => {
        out.push(AddrMode::Deref.to());
        out.extend(c.to_le_bytes())
      }
      t => return Err(format!("expected register/const, found {:?}", t).into()),
    },
    t => return Err(format!("expected register/const/deref, found {:?}", t).into()),
  };
  Ok(())
}

#[derive(Debug)]
enum Token<'a> {
  Label(&'a str),
  Reg(Reg),
  Const(u16),
  Instruction(OpCode),
  Deref(Box<Token<'a>>),
  Eof,
}

impl<'a> Token<'a> {
  fn parse(s: &'a str) -> Result<Self> {
    let mut chars = s.chars().peekable();
    match chars.peek().unwrap() {
      '.' => Ok(Self::Label(&s[1..])),
      '#' => Ok(Self::Reg(Reg::parse(&s[1..])?)),
      '*' => Ok(Self::Deref(Box::new(Token::parse(&s[1..])?))),
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
      _ => Ok(Self::Instruction(OpCode::parse(s)?)),
    }
  }

  fn const_from_radix(s: &str, radix: u32) -> Result<Self> {
    Ok(Self::Const(u16::from_str_radix(&s[2..], radix)?))
  }
}
