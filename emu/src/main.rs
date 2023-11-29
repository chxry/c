use std::{fs, fmt, thread};
use std::time::Duration;
use std::sync::atomic::{AtomicU16, Ordering};
use shared::{Result, Reg, OpCode, AddrMode};

fn main() -> Result {
  let ram = fs::read("out.o")?;
  let registers = Registers::default();
  loop {
    println!("{}", registers);
    registers.mar.store(registers.pc.load());
    load_b(&ram, &registers);

    let opcode = OpCode::from(registers.mdr.load() as _);
    registers.pc.add(1);

    match opcode {
      OpCode::Hlt => return Ok(()),
      OpCode::Jmp => registers.pc.store(get_operand(&ram, &registers)),
      OpCode::Add => {
        registers.pc.add(1);
        let src = get_operand(&ram, &registers);
        load_b(&ram, &registers);
        get_reg(&registers).add(src);
      }
      OpCode::Mov => {
        registers.pc.add(1);
        let src = get_operand(&ram, &registers);
        load_b(&ram, &registers);
        get_reg(&registers).store(src);
      }
    };
    thread::sleep(Duration::from_millis(50));
  }
}

fn load(ram: &[u8], registers: &Registers) {
  let s = registers.mar.load() as usize;
  registers
    .mdr
    .store(u16::from_le_bytes([ram[s], ram[s + 1]]));
}

fn load_b(ram: &[u8], registers: &Registers) {
  registers.mdr.store(ram[registers.mar.load() as usize] as _);
}

fn get_reg(registers: &Registers) -> &Register {
  registers.get(Reg::from(registers.mdr.load() as _))
}

fn get_operand(ram: &[u8], registers: &Registers) -> u16 {
  registers.mar.add(1);
  load_b(&ram, &registers);
  let addr_mode = AddrMode::from(registers.mdr.load() as _);
  registers.mar.add(1);
  match addr_mode {
    AddrMode::Reg => {
      load_b(&ram, &registers);
      registers.pc.add(2);
      registers.mar.add(1);
      get_reg(&registers).load()
    }
    AddrMode::DerefReg => {
      load_b(&ram, &registers);
      registers.mar.store(get_reg(&registers).load());
      load(&ram, &registers);
      registers.pc.add(2);
      registers.mar.add(1);
      registers.mdr.load()
    }
    AddrMode::Const => {
      load(&ram, &registers);
      registers.pc.add(3);
      registers.mar.add(2);
      registers.mdr.load()
    }
    AddrMode::Deref => {
      load(&ram, &registers);
      registers.mar.store(registers.mdr.load());
      load(&ram, &registers);
      registers.pc.add(3);
      registers.mar.add(2);
      registers.mdr.load()
    }
  }
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
struct Register(AtomicU16);

impl Register {
  fn load(&self) -> u16 {
    self.0.load(Ordering::Relaxed)
  }

  fn store(&self, val: u16) {
    self.0.store(val, Ordering::Relaxed);
  }

  fn add(&self, val: u16) {
    self.0.fetch_add(val, Ordering::Relaxed);
  }
}

impl fmt::Display for Register {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.load())
  }
}
