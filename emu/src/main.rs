use std::{fs, fmt, thread};
use std::time::Duration;
use std::sync::atomic::{AtomicU8, Ordering};
use shared::{Result, Reg, OpCode};

fn main() -> Result {
  let ram = fs::read("out.o")?;
  let registers = Registers::default();
  loop {
    println!("{}", registers);
    registers.mar.store(registers.pc.load());
    load(&ram, &registers);

    let inst = OpCode::from(registers.mdr.load());
    println!("{:?}", inst);
    registers.pc.add(inst.len());

    match inst {
      OpCode::Hlt => return Ok(()),
      OpCode::JmpDR => {
        registers.mar.add(1);
        load(&ram, &registers);
        registers.mar.store(get_reg(&registers).load());
        load(&ram, &registers);
        registers.pc.store(registers.mdr.load());
      }
      OpCode::JmpR => {
        registers.mar.add(1);
        load(&ram, &registers);
        registers.mar.store(get_reg(&registers).load());
        registers.pc.store(get_reg(&registers).load());
      }
      OpCode::JmpC => {
        registers.mar.add(1);
        load(&ram, &registers);
        registers.pc.store(registers.mdr.load());
      }
      OpCode::JmpD => {
        registers.mar.add(1);
        load(&ram, &registers);
        registers.mar.store(registers.mdr.load());
        load(&ram, &registers);
        registers.pc.store(registers.mdr.load());
      }
      OpCode::AddDR | OpCode::AddR | OpCode::AddC | OpCode::AddD => {
        registers.mar.add(1);
        load(&ram, &registers);
        let dest = get_reg(&registers);
        registers.mar.add(1);
        load(&ram, &registers);
        match inst {
          OpCode::AddDR => {
            registers.mar.store(get_reg(&registers).load());
            load(&ram, &registers);
          }
          OpCode::AddR => dest.add(get_reg(&registers).load()),
          OpCode::AddC => dest.add(registers.mdr.load()),
          OpCode::AddD => {
            registers.mar.store(registers.mdr.load());
            load(&ram, &registers);
          }
          _ => unreachable!(),
        }
      }
    }
    thread::sleep(Duration::from_millis(50));
  }
}

fn load(ram: &[u8], registers: &Registers) {
  registers.mdr.store(ram[registers.mar.load() as usize]);
}

fn get_reg(registers: &Registers) -> &Register {
  registers.get(Reg::from(registers.mdr.load()))
}

#[derive(Default)]
struct Registers {
  pc: Register,
  mar: Register,
  mdr: Register,
  a: Register,
  b: Register,
  c: Register,
  d: Register,
  e: Register,
  f: Register,
  g: Register,
  h: Register,
}

impl Registers {
  fn get(&self, r: Reg) -> &Register {
    match r {
      Reg::Pc => &self.pc,
      Reg::Mar => &self.mar,
      Reg::Mdr => &self.mdr,
      Reg::A => &self.a,
      Reg::B => &self.b,
      Reg::C => &self.c,
      Reg::D => &self.d,
      Reg::E => &self.e,
      Reg::F => &self.f,
      Reg::G => &self.g,
      Reg::H => &self.h,
    }
  }
}

impl fmt::Display for Registers {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "pc: {}, mar: {}, mdr: {}\na: {}, b: {}, c: {}, d: {}, e: {}, f: {}, g: {}, h: {}",
      self.pc, self.mar, self.mdr, self.a, self.b, self.c, self.d, self.e, self.f, self.g, self.h
    )
  }
}

#[derive(Default)]
struct Register(AtomicU8);

impl Register {
  fn load(&self) -> u8 {
    self.0.load(Ordering::Relaxed)
  }

  fn store(&self, val: u8) {
    self.0.store(val, Ordering::Relaxed);
  }

  fn add(&self, val: u8) {
    self.0.fetch_add(val, Ordering::Relaxed);
  }
}

impl fmt::Display for Register {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.load())
  }
}
