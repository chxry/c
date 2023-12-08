#![feature(array_chunks)]
use std::{fs, fmt};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU16, Ordering::Relaxed};
use std::cmp::Ordering;
use eframe::egui;
use shared::{Result, Reg, OpCode, AddrMode};

fn main() -> Result {
  eframe::run_native(
    "emu",
    eframe::NativeOptions {
      vsync: false,
      ..Default::default()
    },
    Box::new(|cc| Box::new(Emulator::new(cc).unwrap())),
  )?;
  Ok(())
}

struct Emulator {
  ram: Vec<u8>,
  registers: Registers,
  run: bool,
  last_cycle: Instant,
}

impl Emulator {
  fn new(_: &eframe::CreationContext<'_>) -> Result<Self> {
    Ok(Self {
      ram: fs::read("out.o")?,
      registers: Registers::default(),
      run: true,
      last_cycle: Instant::now(),
    })
  }
}

impl eframe::App for Emulator {
  fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
    if self.run && Instant::now().duration_since(self.last_cycle) > Duration::from_millis(10) {
      self.run = cycle(&mut self.ram, &mut self.registers).unwrap();
      self.last_cycle = Instant::now();
    }
    egui::Window::new("state").show(ctx, |ui| {
      ui.label(format!("running: {}", self.run));
    });
    egui::Window::new("registers")
      .default_width(640.0)
      .show(ctx, |ui| {
        ui.monospace(format!("{}", self.registers));
      });
    egui::Window::new("memory")
      .default_height(320.0)
      .vscroll(true)
      .show(ctx, |ui| {
        for (i, r) in self.ram.array_chunks::<16>().enumerate() {
          ui.horizontal(|ui| {
            ui.monospace(format!("{:08x}", i * 16));
            ui.code(r.map(|b| format!("{:02x}", b)).join(" "));
          });
        }
      });
    ctx.request_repaint();
  }
}

fn cycle(ram: &mut Vec<u8>, registers: &mut Registers) -> Result<bool> {
  registers.mar.store(registers.pc.load());
  load_b(ram, registers);

  let opcode = OpCode::from(registers.mdr.load() as _);
  registers.pc.add(1);

  match opcode {
    OpCode::Hlt => return Ok(false),
    OpCode::Jmp
    | OpCode::Jeq
    | OpCode::Jne
    | OpCode::Jlt
    | OpCode::Jle
    | OpCode::Jgt
    | OpCode::Jge => {
      registers.mar.add(1);
      let to = get_operand(ram, registers);
      let flgs = registers.flgs.load();
      if match opcode {
        OpCode::Jmp => true,
        OpCode::Jeq | OpCode::Jle | OpCode::Jge if flgs == EQUAL => true,
        OpCode::Jne if flgs != EQUAL => true,
        OpCode::Jlt if flgs == LESS => true,
        OpCode::Jgt if flgs == GREATER => true,
        _ => false,
      } {
        registers.pc.store(to.load(ram, registers));
      }
    }
    OpCode::Call => {
      registers.mar.add(1);
      let f = get_operand(ram, registers);
      registers.mdr.store(registers.pc.load());
      push(ram, registers);
      registers.pc.store(f.load(ram, registers));
    }
    OpCode::Ret => {
      pop(ram, registers);
      registers.pc.store(registers.mdr.load());
    }
    OpCode::Push => {
      registers.mar.add(1);
      let val = get_operand(ram, registers);
      registers.mdr.store(val.load(ram, registers));
      push(ram, registers);
    }
    OpCode::Pop => {
      registers.mar.add(1);
      let dest = get_operand(ram, registers);
      pop(ram, registers);
      dest.store(ram, registers, registers.mdr.load());
    }
    OpCode::Cmp
    | OpCode::Add
    | OpCode::Sub
    | OpCode::Mul
    | OpCode::Div
    | OpCode::Mov
    | OpCode::Out => {
      registers.mar.add(1);
      let src = get_operand(ram, registers);
      let dest = get_operand(ram, registers);
      if matches!(src, Operand::Mem(_)) && matches!(dest, Operand::Mem(_)) {
        panic!("invalid operand type");
      }
      let src_val = src.load(ram, registers);
      let dest_val = dest.load(ram, registers);
      match opcode {
        OpCode::Cmp => match dest.load(ram, registers).cmp(&src_val) {
          Ordering::Less => registers.flgs.store(LESS),
          Ordering::Equal => registers.flgs.store(EQUAL),
          Ordering::Greater => registers.flgs.store(GREATER),
        },
        OpCode::Add => dest.store(ram, registers, dest_val + src_val),
        OpCode::Sub => dest.store(ram, registers, dest_val - src_val),
        OpCode::Mul => dest.store(ram, registers, dest_val * src_val),
        OpCode::Div => {
          dest.store(ram, registers, dest_val / src_val);
          registers.im.store(dest_val % src_val);
        }
        OpCode::Mov => dest.store(ram, registers, src_val),
        OpCode::Out => {
          println!("OUT {} {}", src_val, dest_val);
        }
        _ => unreachable!(),
      }
    }
  };
  Ok(true)
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

fn push(ram: &mut Vec<u8>, registers: &Registers) {
  registers.sp.sub(2);
  registers.mar.store(registers.sp.load());
  store(ram, registers);
}

fn pop(ram: &[u8], registers: &Registers) {
  registers.mar.store(registers.sp.load());
  load(ram, registers);
  registers.sp.add(2);
}

fn get_operand<'a>(ram: &[u8], registers: &'a Registers) -> Operand<'a> {
  load_b(ram, registers);
  let addr_mode = AddrMode::from(registers.mdr.load() as _);
  registers.pc.add(1);
  registers.mar.add(1);
  match addr_mode {
    AddrMode::Reg => {
      load_b(ram, registers);
      registers.pc.add(1);
      registers.mar.add(1);
      Operand::Reg(get_reg(registers))
    }
    AddrMode::DerefReg => {
      load_b(ram, registers);
      registers.pc.add(1);
      registers.mar.add(1);
      Operand::Mem(get_reg(registers).load())
    }
    AddrMode::Const => {
      load(ram, registers);
      let val = registers.mdr.load();
      registers.pc.add(2);
      registers.mar.add(2);
      Operand::Const(val)
    }
    AddrMode::Deref => {
      load(ram, registers);
      let val = registers.mdr.load();
      registers.pc.add(2);
      registers.mar.add(2);
      Operand::Mem(val)
    }
  }
}

enum Operand<'a> {
  Reg(&'a Register),
  Mem(u16),
  Const(u16),
}

impl Operand<'_> {
  fn load(&self, ram: &[u8], registers: &Registers) -> u16 {
    match self {
      Self::Reg(r) => r.load(),
      Self::Mem(a) => {
        registers.mar.store(*a);
        load(ram, registers);
        registers.mdr.load()
      }
      Self::Const(c) => *c,
    }
  }

  fn store(&self, ram: &mut Vec<u8>, registers: &Registers, val: u16) {
    match self {
      Self::Reg(r) => r.store(val),
      Self::Mem(a) => {
        registers.mar.store(*a);
        store(ram, registers);
      }
      Self::Const(_) => panic!("attempted to set const"),
    }
  }
}

#[derive(Default)]
struct Registers {
  pc: Register,
  mar: Register,
  mdr: Register,
  im: Register,
  sp: Register,
  flgs: Register,
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
      Reg::Im => &self.im,
      Reg::Sp => &self.sp,
      Reg::Flgs => &self.flgs,
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
      "pc: {}, mar: {}, mdr: {}, im: {}, sp: {}, flgs: {:08b}\na: {}, b: {}, c: {}, d: {}, e: {}, f: {}, g: {}, h: {}",
      self.pc, self.mar, self.mdr, self.im, self.sp, self.flgs.load(), self.a, self.b, self.c, self.d, self.e, self.f, self.g, self.h
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

  fn sub(&self, val: u16) {
    self.0.fetch_sub(val, Relaxed);
  }
}

impl fmt::Display for Register {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:04x}", self.load())
  }
}

const LESS: u16 = 0b01;
const EQUAL: u16 = 0b10;
const GREATER: u16 = 0b100;
