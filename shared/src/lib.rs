use std::mem;

pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Copy, Clone, Debug)]
pub enum Reg {
  Pc,
  Mar,
  Mdr,
  Flgs,
  Im,
  A,
  B,
  C,
  D,
  E,
  F,
  G,
  H,
}

impl Reg {
  pub fn parse(s: &str) -> Result<Self> {
    match &*s.to_uppercase() {
      "PC" => Ok(Self::Pc),
      "MAR" => Ok(Self::Mar),
      "MDR" => Ok(Self::Mdr),
      "FLGS" => Ok(Self::Flgs),
      "IM" => Ok(Self::Im),
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

  pub fn to(&self) -> u8 {
    unsafe { mem::transmute(*self) }
  }

  pub fn from(b: u8) -> Self {
    unsafe { mem::transmute(b) }
  }
}

#[derive(Copy, Clone, Debug)]
pub enum OpCode {
  Hlt,
  Jmp,
  Jeq,
  Jne,
  Jlt,
  Jle,
  Jgt,
  Jge,
  Cmp,
  Add,
  Sub,
  Mul,
  Div,
  Mov,
  Out,
}

impl OpCode {
  pub fn parse(s: &str) -> Result<Self> {
    match &*s.to_uppercase() {
      "HLT" => Ok(Self::Hlt),
      "JMP" => Ok(Self::Jmp),
      "JEQ" => Ok(Self::Jeq),
      "JNE" => Ok(Self::Jne),
      "JLT" => Ok(Self::Jlt),
      "JLE" => Ok(Self::Jle),
      "JGT" => Ok(Self::Jgt),
      "JGE" => Ok(Self::Jge),
      "CMP" => Ok(Self::Cmp),
      "ADD" => Ok(Self::Add),
      "SUB" => Ok(Self::Sub),
      "MUL" => Ok(Self::Mul),
      "DIV" => Ok(Self::Div),
      "MOV" => Ok(Self::Mov),
      "OUT" => Ok(Self::Out),
      _ => Err(format!("unknown opcode '{}'", s).into()),
    }
  }

  pub fn to(&self) -> u8 {
    unsafe { mem::transmute(*self) }
  }

  pub fn from(b: u8) -> Self {
    unsafe { mem::transmute(b) }
  }
}

#[derive(Copy, Clone, Debug)]
pub enum AddrMode {
  Reg,
  DerefReg,
  Const,
  Deref,
}

impl AddrMode {
  pub fn to(&self) -> u8 {
    unsafe { mem::transmute(*self) }
  }

  pub fn from(b: u8) -> Self {
    unsafe { mem::transmute(b) }
  }
}
