use std::fs;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::vec::IntoIter;
use shared::{Result, Reg, OpCode};

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
          e.insert(out.len());
        }
      },
      Token::Instruction(i) => match i {
        Instruction::Hlt => out.push(OpCode::Hlt.to()),
        Instruction::Jmp => match eval_val(&mut tokens, &labels)? {
          Value::Reg(to) => out.extend([OpCode::JmpR.to(), to.to()]),
          Value::DerefReg(to) => out.extend([OpCode::JmpDR.to(), to.to()]),
          Value::Const(to) => {
            out.push(OpCode::JmpC.to());
            out.extend(to.to_le_bytes());
          }
          Value::Deref(to) => {
            out.push(OpCode::JmpD.to());
            out.extend(to.to_le_bytes());
          }
        },
        Instruction::Db => {}
        Instruction::Add => {
          let dest = eval_reg(&mut tokens)?;
          match eval_val(&mut tokens, &labels)? {
            Value::Reg(src) => out.extend([OpCode::AddR.to(), dest.to(), src.to()]),
            Value::DerefReg(src) => out.extend([OpCode::AddDR.to(), dest.to(), src.to()]),
            Value::Const(src) => {
              out.extend([OpCode::AddC.to(), dest.to()]);
              out.extend(src.to_le_bytes());
            }
            Value::Deref(src) => {
              out.extend([OpCode::AddD.to(), dest.to()]);
              out.extend(src.to_le_bytes());
            }
          }
        }
      },
      Token::Eof => {}
      _ => return Err(format!("expected label/instruction, found {:?}", t).into()),
    };
  }
  Ok(out)
}

fn eval_reg(tokens: &mut IntoIter<Token>) -> Result<Reg> {
  match tokens.next().unwrap_or(Token::Eof) {
    Token::Reg(r) => Ok(r),
    t => Err(format!("expected register, found {:?}", t).into()),
  }
}

fn eval_val(tokens: &mut IntoIter<Token>, labels: &HashMap<&str, usize>) -> Result<Value> {
  match tokens.next().unwrap_or(Token::Eof) {
    Token::Reg(r) => Ok(Value::Reg(r)),
    Token::Const(c) => Ok(Value::Const(c)),
    Token::Label(l) => match labels.get(l) {
      Some(i) => Ok(Value::Const(*i as _)),
      None => Err(format!("unknown label '{}'", l).into()),
    },
    Token::Deref(t) => match *t {
      Token::Reg(r) => Ok(Value::DerefReg(r)),
      Token::Const(c) => Ok(Value::Deref(c)),
      t => Err(format!("expected register/const, found {:?}", t).into()),
    },
    t => Err(format!("expected register/const/deref, found {:?}", t).into()),
  }
}

#[derive(Debug)]
enum Value {
  Reg(Reg),
  DerefReg(Reg),
  Const(u16),
  Deref(u16),
}

#[derive(Debug)]
enum Token<'a> {
  Label(&'a str),
  Reg(Reg),
  Const(u16),
  Instruction(Instruction),
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
      _ => Ok(Self::Instruction(Instruction::parse(s)?)),
    }
  }

  fn const_from_radix(s: &str, radix: u32) -> Result<Self> {
    Ok(Self::Const(u16::from_str_radix(&s[2..], radix)?))
  }
}

#[derive(Debug)]
enum Instruction {
  Hlt,
  Jmp,
  Db,
  Add,
}

impl Instruction {
  fn parse(s: &str) -> Result<Self> {
    match &*s.to_uppercase() {
      "HLT" => Ok(Self::Hlt),
      "JMP" => Ok(Self::Jmp),
      "DB" => Ok(Self::Db),
      "ADD" => Ok(Self::Add),
      _ => Err(format!("unknown instruction '{}'", s).into()),
    }
  }
}
