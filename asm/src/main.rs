use std::fs;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::vec::IntoIter;
use shared::{Result, Reg, OpCode, AddrMode};

const PATH: &str = "test.asm";

fn main() -> Result {
  let src = fs::read_to_string(PATH)?;
  let tokens = parse(&src)?;
  let out = assemble(tokens)?;
  fs::write("out.o", out)?;
  println!("successfully assembled '{}'", PATH);
  Ok(())
}

fn parse(src: &str) -> Result<Vec<Token>> {
  let mut tokens = vec![];
  for (i, line) in src.lines().enumerate() {
    if !line.starts_with(';') {
      for s in line.split_whitespace() {
        match Token::parse(s) {
          Ok(t) => tokens.push(t),
          Err(e) => return Err(format!("syntax err on line {}: {}", i + 1, e).into()),
        }
      }
    }
  }
  tokens.push(Token::Eof);
  Ok(tokens)
}

fn assemble(tokens: Vec<Token>) -> Result<Vec<u8>> {
  let mut labels = HashMap::new();
  let mut resolves = vec![];
  let mut out = vec![];
  let mut tokens = tokens.into_iter();
  while let Some(t) = tokens.next() {
    match t {
      Token::Label(s) => match labels.entry(s) {
        Entry::Occupied(_) => return Err(format!("label already declared '{}'", s).into()),
        Entry::Vacant(e) => {
          e.insert(out.len() as u16);
        }
      },
      Token::Instruction(i) => {
        out.push(i.to());
        match i {
          OpCode::Hlt | OpCode::Ret => {}
          OpCode::Jmp
          | OpCode::Jeq
          | OpCode::Jne
          | OpCode::Jlt
          | OpCode::Jle
          | OpCode::Jgt
          | OpCode::Jge
          | OpCode::Call
          | OpCode::Push
          | OpCode::Out => {
            op_any(&mut tokens, &mut resolves, true, true, &mut out)?;
          }
          OpCode::Pop => {
            op_any(&mut tokens, &mut resolves, true, false, &mut out)?;
          }
          OpCode::Cmp => {
            let allow_deref = op_any(&mut tokens, &mut resolves, true, true, &mut out)?;
            op_any(&mut tokens, &mut resolves, allow_deref, true, &mut out)?;
          }
          OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div | OpCode::Mov => {
            let allow_deref = op_any(&mut tokens, &mut resolves, true, true, &mut out)?;
            op_any(&mut tokens, &mut resolves, allow_deref, false, &mut out)?;
          }
        }
      }
      Token::Pseudo(p) => match tokens.next().unwrap_or(Token::Eof) {
        Token::Const(c) => match p {
          Pseudo::Db => out.push(c as _),
          Pseudo::Dw => out.extend(c.to_le_bytes()),
          Pseudo::Dn => match tokens.next().unwrap_or(Token::Eof) {
            Token::Const(n) => out.extend(vec![c as u8; n as _]),
            t => return Err(format!("expected const, found {:?}", t).into()),
          },
        },
        t => return Err(format!("expected const, found {:?}", t).into()),
      },
      Token::Eof => {}
      _ => return Err(format!("expected label/instruction, found {:?}", t).into()),
    };
  }
  for (i, l) in resolves {
    match labels.get(l) {
      Some(c) => {
        out.splice(i..=i + 1, c.to_le_bytes());
      }
      None => return Err(format!("unknown label '{}'", l).into()),
    }
  }
  Ok(out)
}

fn op_any<'a>(
  tokens: &mut IntoIter<Token<'a>>,
  resolves: &mut Vec<(usize, &'a str)>,
  allow_deref: bool,
  allow_const: bool,
  out: &mut Vec<u8>,
) -> Result<bool> {
  match tokens.next().unwrap_or(Token::Eof) {
    Token::Reg(r) => out.extend([AddrMode::Reg.to(), r.to()]),
    Token::Const(_) | Token::Label(_) if !allow_const => {
      return Err("dest cannot be const/label".into())
    }
    Token::Const(c) => {
      out.push(AddrMode::Const.to());
      out.extend(c.to_le_bytes())
    }
    Token::Label(l) => {
      out.extend([AddrMode::Const.to(), 0, 0]);
      resolves.push((out.len() - 2, l));
    }
    Token::Deref(_) if !allow_deref => return Err("cannot have two deref operands".into()),
    Token::Deref(t) => {
      match *t {
        Token::Reg(r) => out.extend([AddrMode::DerefReg.to(), r.to()]),
        Token::Const(c) => {
          out.push(AddrMode::Deref.to());
          out.extend(c.to_le_bytes())
        }
        Token::Label(l) => {
          out.extend([AddrMode::Deref.to(), 0, 0]);
          resolves.push((out.len() - 2, l));
        }
        t => return Err(format!("expected register/const/label, found {:?}", t).into()),
      }
      return Ok(false);
    }
    t => return Err(format!("expected register/const/deref, found {:?}", t).into()),
  };
  Ok(true)
}

#[derive(Debug)]
enum Token<'a> {
  Label(&'a str),
  Reg(Reg),
  Const(u16),
  Instruction(OpCode),
  Pseudo(Pseudo),
  Deref(Box<Token<'a>>),
  Eof,
}

impl<'a> Token<'a> {
  fn parse(s: &'a str) -> Result<Self> {
    let mut chars = s.chars().peekable();
    match chars.peek().unwrap() {
      '.' => Ok(Self::Label(&s[1..])),
      '%' => Ok(Self::Reg(Reg::parse(&s[1..])?)),
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
      _ => match &*s.to_uppercase() {
        "DB" => Ok(Self::Pseudo(Pseudo::Db)),
        "DW" => Ok(Self::Pseudo(Pseudo::Dw)),
        "DN" => Ok(Self::Pseudo(Pseudo::Dn)),
        _ => Ok(Self::Instruction(OpCode::parse(s)?)),
      },
    }
  }

  fn const_from_radix(s: &str, radix: u32) -> Result<Self> {
    Ok(Self::Const(u16::from_str_radix(&s[2..], radix)?))
  }
}

#[derive(Debug)]
enum Pseudo {
  Db,
  Dw,
  Dn,
}
