use std::io::{self, Write};
use std::ops::RangeInclusive;
use std::time::Duration;

use deku::{bitvec::BitView, prelude::*};

use crossterm::{ExecutableCommand, QueueableCommand};
use crossterm::event::{Event, KeyCode};
use crossterm::{cursor, event, style, terminal};

use crate::ops::Op;

pub struct Chip8 {
    pc: u16,
    mem: Box<[u8; 1024 * 4]>,
    ireg: u16,
    stack: Vec<u16>,
    _dt: u8,
    _st: u8,
    v: Registers,
    screen: Screen,
}

struct Registers([u8; 16]);

impl std::ops::Index<u8> for Registers {
    type Output = u8;

    #[inline]
    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl std::ops::IndexMut<u8> for Registers {
    #[inline]
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl Chip8 {
    pub fn new(screen: Screen) -> Self {
        let mut mem = Box::new([0; 1024 * 4]);
        mem[FONT_RANGE].copy_from_slice(FONT);

        Chip8 {
            pc: 512,
            mem,
            ireg: 0,
            stack: Vec::new(),
            _dt: 0,
            _st: 0,
            v: Registers([0; 16]),
            screen,
        }
    }

    pub fn load_program(&mut self, program: &[u8]) {
        let pc = self.pc as usize;
        self.mem[pc..pc + program.len()].copy_from_slice(program)
    }

    pub fn run(&mut self) -> io::Result<()> {
        loop {
            self.tick()?;

            if event::poll(Duration::ZERO)? {
                if event::read()? == Event::Key(KeyCode::Char('q').into()) {
                    return Ok(());
                }
            }
        }
    }

    fn tick(&mut self) -> io::Result<()> {
        let (_, op) = Op::from_bytes((&self.mem[self.pc as usize..], 0)).unwrap();
        self.pc += 2;
        self.execute(op)
    }

    fn execute(&mut self, op: Op) -> io::Result<()> {
        match op {
            Op::AbsJump(addr) => {
                self.pc = addr;
            }
            Op::Call(addr) => {
                self.stack.push(self.pc);
                self.pc = addr;
            }
            Op::Return => {
                self.pc = self.stack.pop().expect("stack underflow");
            }
            Op::Clear => {
                self.screen.clear()?;
            }
            Op::SkipEqVal(x, val) => {
                if self.v[x] == val {
                    self.pc += 2;
                }
            }
            Op::SkipNeqVal(x, val) => {
                if self.v[x] != val {
                    self.pc += 2;
                }
            }
            Op::SkipEqReg(x, y) => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            Op::SkipNeqReg(x, y) => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            Op::SetVal(x, val) => {
                self.v[x] = val;
            }
            Op::AddVal(x, val) => {
                let vx = &mut self.v[x];
                *vx = vx.wrapping_add(val);
            }
            Op::Mov(x, y) => {
                self.v[x] = self.v[y];
            }
            Op::Or(x, y) => {
                self.v[x] |= self.v[y];
            }
            Op::And(x, y) => {
                self.v[x] &= self.v[y];
            }
            Op::Xor(x, y) => {
                self.v[x] ^= self.v[y];
            }
            Op::Add(x, y) => {
                let (result, overflow) = self.v[x].overflowing_add(self.v[y]);
                self.v[0xF] = overflow as u8;
                self.v[x] = result;
            }
            Op::Sub(x, y) => {
                let (result, overflow) = self.v[x].overflowing_sub(self.v[y]);
                self.v[0xF] = overflow as u8;
                self.v[x] = result;
            }
            Op::SubN(x, y) => {
                let (result, overflow) = self.v[y].overflowing_sub(self.v[x]);
                self.v[0xF] = overflow as u8;
                self.v[x] = result;
            }
            Op::Shr(x, y) => {
                self.v[x] = self.v[y]; // TODO: add config to skip this
                self.v[0xF] = self.v[x] & 0b0000_0001;
                self.v[x] >>= 1;
            }
            Op::Shl(x, y) => {
                self.v[x] = self.v[y]; // TODO: add config to skip this
                self.v[0xF] = self.v[x] & 0b1000_0000;
                self.v[x] <<= 1;
            }
            Op::SetIndex(addr) => {
                self.ireg = addr;
            }
            Op::OffsetJump(addr) => {
                self.pc = addr + self.v[0] as u16;
            }
            Op::Rand(x, val) => {
                self.v[x] = fastrand::u8(..) & val;
            }
            Op::Draw(x, y, height) => {
                let sprite = &self.mem[self.ireg as usize..(self.ireg as usize + height as usize)];
                let start_x = self.v[x] as usize & (SCREEN_WIDTH - 1);
                let start_y = self.v[y] as usize & (SCREEN_HEIGHT - 1);
                self.v[0xF] = 0;

                for (screen_y, row) in (start_y..SCREEN_HEIGHT).zip(sprite) {
                    for (screen_x, bit) in
                        (start_x..SCREEN_WIDTH).zip(row.view_bits::<deku::bitvec::Msb0>())
                    {
                        let pixel = self.screen.pixel_mut(screen_x, screen_y);
                        match (*pixel, *bit) {
                            (true, true) => {
                                *pixel = false;
                                self.v[0xF] = 1;
                            }
                            (false, true) => *pixel = true,
                            _ => (),
                        }
                    }
                }
                self.screen.draw()?;
            }
            _ => todo!("{:?}", op),
        }

        Ok(())
    }
}

const FONT_RANGE: RangeInclusive<usize> = 0x50..=0x9F;
const FONT: &[u8] = &[
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const N_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

pub struct Screen {
    pixels: [bool; N_PIXELS],
    output: io::Stdout,
}

impl Screen {
    const ON: &str = "\u{2588}\u{2588}";
    const OFF: &str = "  ";

    pub fn new(output: io::Stdout) -> Self {
        Screen {
            pixels: [false; N_PIXELS],
            output,
        }
    }

    #[inline]
    fn pixel(&self, x: usize, y: usize) -> &bool {
        &self.pixels[SCREEN_WIDTH * y + x]
    }
    #[inline]
    fn pixel_mut(&mut self, x: usize, y: usize) -> &mut bool {
        &mut self.pixels[SCREEN_WIDTH * y + x]
    }

    fn draw(&mut self) -> io::Result<()> {
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let symbol = if *self.pixel(x, y) {
                    Self::ON
                } else {
                    Self::OFF
                };
                self.output
                    .queue(cursor::MoveTo(x as u16 * 2, y as u16))?
                    .queue(style::Print(symbol))?;
            }
        }
        self.output.flush()
    }

    #[inline]
    fn clear(&mut self) -> io::Result<()> {
        self.output
            .execute(terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }
}
