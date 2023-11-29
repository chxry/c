use std::mem;

pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[rustfmt::skip]
#[derive(Copy, Clone, Debug)]
pub enum Reg{
  Pc, Mar, Mdr, A, B, C, D, E, F, G, H
}

impl Reg {
  pub fn parse(s: &str) -> Result<Self> {
    match &*s.to_uppercase() {
      "PC" => Ok(Self::Pc),
      "MAR" => Ok(Self::Mar),
      "MDR" => Ok(Self::Mdr),
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
  JmpDR,
  JmpR,
  JmpC,
  JmpD,
  AddR,
  AddDR,
  AddC,
  AddD,
}

impl OpCode {
  pub fn to(&self) -> u8 {
    unsafe { mem::transmute(*self) }
  }

  pub fn from(b: u8) -> Self {
    unsafe { mem::transmute(b) }
  }

  pub fn len(&self) -> u16 {
    match self {
      Self::Hlt => 1,
      Self::JmpDR | Self::JmpR => 2,
      Self::JmpC | Self::JmpD | Self::AddDR | Self::AddR => 3,
      Self::AddC | Self::AddD => 4,
    }
  }
}
