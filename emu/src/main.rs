use std::{fs, fmt, thread};
use std::time::Duration;
use std::sync::atomic::{AtomicU16, Ordering::Relaxed};
use std::cmp::Ordering;
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
      OpCode::Jmp
      | OpCode::Jeq
      | OpCode::Jne
      | OpCode::Jlt
      | OpCode::Jle
      | OpCode::Jgt
      | OpCode::Jge => {
        let to = get_operand(&ram, &registers);
        let flags = registers.cflgs.load();
        if match opcode {
          OpCode::Jmp => true,
          OpCode::Jeq | OpCode::Jle | OpCode::Jge if flags == EQUAL => true,
          OpCode::Jne if flags != EQUAL => true,
          OpCode::Jlt if flags == LESS => true,
          OpCode::Jgt if flags == GREATER => true,
          _ => false,
        } {
          registers.pc.store(to);
        }
      }
      OpCode::Cmp | OpCode::Add | OpCode::Mov => {
        registers.pc.add(1);
        let src = get_operand(&ram, &registers);
        load_b(&ram, &registers);
        let dest = get_reg(&registers); // use get operand
        match opcode {
          OpCode::Cmp => match dest.load().cmp(&src) {
            Ordering::Less => registers.cflgs.store(LESS),
            Ordering::Equal => registers.cflgs.store(EQUAL),
            Ordering::Greater => registers.cflgs.store(GREATER),
          },
          OpCode::Add => dest.add(src),
          OpCode::Mov => dest.store(src),
          _ => unreachable!(),
        }
      }
    };

    thread::sleep(Duration::from_millis(50));
  }
}

fn load(ram: &[u8], registers: &Registers) {
  let i = registers.mar.load() as usize;
  registers
    .mdr
    .store(u16::from_le_bytes([ram[i], ram[i + 1]]));
}

fn store(ram: &mut Vec<u8>, registers: &Registers) {
  let i = registers.mar.load() as usize;
  ram.splice(i..=i + 1, registers.mdr.load().to_le_bytes());
}

fn load_b(ram: &[u8], registers: &Registers) {
  registers.mdr.store(ram[registers.mar.load() as usize] as _);
}

fn get_reg(registers: &Registers) -> &Register {
  registers.get(Reg::from(registers.mdr.load() as _))
}

fn get_operand(ram: &[u8], registers: &Registers) -> u16 {
  registers.mar.add(1);
  load_b(ram, registers);
  let addr_mode = AddrMode::from(registers.mdr.load() as _);
  registers.mar.add(1);
  match addr_mode {
    AddrMode::Reg => {
      load_b(ram, registers);
      registers.pc.add(2);
      registers.mar.add(1);
      get_reg(registers).load()
    }
    AddrMode::DerefReg => {
      load_b(ram, registers);
      registers.mar.store(get_reg(registers).load());
      load(ram, registers);
      registers.pc.add(2);
      registers.mar.add(1);
      registers.mdr.load()
    }
    AddrMode::Const => {
      load(ram, registers);
      registers.pc.add(3);
      registers.mar.add(2);
      registers.mdr.load()
    }
    AddrMode::Deref => {
      load(ram, registers);
      registers.mar.store(registers.mdr.load());
      load(ram, registers);
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
  cflgs: Register,
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
      Reg::Cflgs => &self.cflgs,
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
      "pc: {}, mar: {}, mdr: {}, cflgs: {:08b}\na: {}, b: {}, c: {}, d: {}, e: {}, f: {}, g: {}, h: {}",
      self.pc, self.mar, self.mdr, self.cflgs.load(), self.a, self.b, self.c, self.d, self.e, self.f, self.g, self.h
    )
  }
}

#[derive(Default)]
struct Register(AtomicU16);

impl Register {
  fn load(&self) -> u16 {
    self.0.load(Relaxed)
  }

  fn store(&self, val: u16) {
    self.0.store(val, Relaxed);
  }

  fn add(&self, val: u16) {
    self.0.fetch_add(val, Relaxed);
  }
}

impl fmt::Display for Register {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.load())
  }
}

const LESS: u16 = 0b01;
const EQUAL: u16 = 0b10;
const GREATER: u16 = 0b11;
