use std::io::Write;
use std::time::Duration;
use std::{fs, io, ops::RangeInclusive};

use deku::{bitvec::BitView, prelude::*};

use crossterm::event::{Event, KeyCode};
use crossterm::{cursor, event, style, terminal, ExecutableCommand, QueueableCommand};

fn main() -> io::Result<()> {
    let program_path = std::env::args()
        .nth(1)
        .expect("USAGE: ./chip8 <PROGRAM.ch8>");
    let src = fs::read(&program_path)?;

    let original_terminal_size = terminal::size()?;
    prepare_ui(original_terminal_size)?;

    let screen = Screen::new(io::stdout());
    let mut chip8 = Chip8::new(screen);
    chip8.load_program(&src);

    let result = run(chip8);

    restore_ui(original_terminal_size)?;
    result
}

fn run(mut chip8: Chip8) -> io::Result<()> {
    loop {
        chip8.tick()?;

        if event::poll(Duration::ZERO)? {
            if event::read()? == Event::Key(KeyCode::Char('q').into()) {
                return Ok(());
            }
        }
    }
}

fn prepare_ui((rows, cols): (u16, u16)) -> io::Result<()> {
    if rows < 128 || cols < 32 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Minimum supported terminal size is 128x64, but current size is: {rows}x{cols}"
            ),
        ));
    }
    terminal::enable_raw_mode()?;
    io::stdout()
        .execute(terminal::SetSize(SCREEN_WIDTH as u16, SCREEN_HEIGHT as u16))?
        .execute(terminal::Clear(terminal::ClearType::All))?
        .execute(cursor::Hide)?;
    Ok(())
}

fn restore_ui((rows, cols): (u16, u16)) -> io::Result<()> {
    io::stdout()
        .execute(terminal::SetSize(cols, rows))?
        .execute(terminal::Clear(terminal::ClearType::All))?
        .execute(cursor::Show)?
        .execute(style::ResetColor)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u16", endian = "big")]
enum Op {
    #[deku(id = "0x00E0")]
    Clear,

    #[deku(id = "0x00EE")]
    Return,

    #[deku(id_pat = "0x1000..=0x1FFF")]
    AbsJump(#[deku(pad_bits_before = "4", bits = "12")] u16),

    #[deku(id_pat = "0x2000..=0x2FFF")]
    Call(#[deku(pad_bits_before = "4", bits = "12")] u16),

    #[deku(id_pat = "0xB000..=0xBFFF")]
    OffsetJump(#[deku(pad_bits_before = "4", bits = "12")] u16),

    #[deku(id_pat = "0x3000..=0x3FFF")]
    SkipEqVal(#[deku(pad_bits_before = "4", bits = "4")] u8, u8),

    #[deku(id_pat = "0x4000..=0x4FFF")]
    SkipNeqVal(#[deku(pad_bits_before = "4", bits = "4")] u8, u8),

    #[deku(id_pat = "0x5000..=0x5FFF")]
    SkipEqReg(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "0x6000..=0x6FFF")]
    SetVal(#[deku(pad_bits_before = "4", bits = "4")] u8, u8),

    #[deku(id_pat = "0x7000..=0x7FFF")]
    AddVal(#[deku(pad_bits_before = "4", bits = "4")] u8, u8),

    #[deku(id_pat = "0x9000..=0x9FFF")]
    SkipNeqReg(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "0xC000..=0xCFFF")]
    Rand(#[deku(pad_bits_before = "4", bits = "4")] u8, u8),

    #[deku(id_pat = "x @ 0x8000..=0x8FFF if (x & 0xF) == 0x0")]
    Mov(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "x @ 0x8000..=0x8FFF if (x & 0xF) == 0x1")]
    Or(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "x @ 0x8000..=0x8FFF if (x & 0xF) == 0x2")]
    And(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "x @ 0x8000..=0x8FFF if (x & 0xF) == 0x3")]
    Xor(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "x @ 0x8000..=0x8FFF if (x & 0xF) == 0x4")]
    Add(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "x @ 0x8000..=0x8FFF if (x & 0xF) == 0x5")]
    Sub(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "x @ 0x8000..=0x8FFF if (x & 0xF) == 0x6")]
    Shr(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "x @ 0x8000..=0x8FFF if (x & 0xF) == 0x7")]
    SubN(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "x @ 0x8000..=0x8FFF if (x & 0xF) == 0xE")]
    Shl(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(pad_bits_after = "4", bits = "4")] u8,
    ),

    #[deku(id_pat = "0xD000..=0xDFFF")]
    Draw(
        #[deku(pad_bits_before = "4", bits = "4")] u8,
        #[deku(bits = "4")] u8,
        #[deku(bits = "4")] u8,
    ),

    #[deku(
        id_pat = "0xE09E | 0xE19E | 0xE29E | 0xE39E | 0xE49E | 0xE59E | 0xE69E | 0xE79E | 0xE89E | 0xE99E | 0xEA9E | 0xEB9E  | 0xEC9E  | 0xED9E  | 0xEE9E  | 0xEF9E"
    )]
    SkipKey(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(
        id_pat = "0xE0A1 | 0xE1A1 | 0xE2A1 | 0xE3A1 | 0xE4A1 | 0xE5A1 | 0xE6A1 | 0xE7A1 | 0xE8A1 | 0xE9A1 | 0xEAA1 | 0xEBA1  | 0xECA1  | 0xEDA1  | 0xEEA1  | 0xEFA1"
    )]
    SkipNoKey(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(
        id_pat = "0xF00A | 0xF10A | 0xF20A | 0xF30A | 0xF40A | 0xF50A | 0xF60A | 0xF70A | 0xF80A | 0xF90A | 0xFA0A | 0xFB0A  | 0xFC0A  | 0xFD0A  | 0xFE0A  | 0xFF0A"
    )]
    GetKey(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(
        id_pat = "0xF007 | 0xF107 | 0xF207 | 0xF307 | 0xF407 | 0xF507 | 0xF607 | 0xF707 | 0xF807 | 0xF907 | 0xFA07 | 0xFB07  | 0xFC07  | 0xFD07  | 0xFE07  | 0xFF07"
    )]
    GetDelay(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(
        id_pat = "0xF015 | 0xF115 | 0xF215 | 0xF315 | 0xF415 | 0xF515 | 0xF615 | 0xF715 | 0xF815 | 0xF915 | 0xFA15 | 0xFB15  | 0xFC15  | 0xFD15  | 0xFE15  | 0xFF15"
    )]
    SetDelay(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(
        id_pat = "0xF018 | 0xF118 | 0xF218 | 0xF318 | 0xF418 | 0xF518 | 0xF618 | 0xF718 | 0xF818 | 0xF918 | 0xFA18 | 0xFB18  | 0xFC18  | 0xFD18  | 0xFE18  | 0xFF18"
    )]
    SetSoundTimer(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(
        id_pat = "0xF01E | 0xF11E | 0xF21E | 0xF31E | 0xF41E | 0xF51E | 0xF61E | 0xF71E | 0xF81E | 0xF91E | 0xFA1E | 0xFB1E  | 0xFC1E  | 0xFD1E  | 0xFE1E  | 0xFF1E"
    )]
    IncrIndex(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(id_pat = "0xA000..=0xAFFF")]
    SetIndex(#[deku(pad_bits_before = "4", bits = "12")] u16),

    #[deku(
        id_pat = "0xF029 | 0xF129 | 0xF229 | 0xF329 | 0xF429 | 0xF529 | 0xF629 | 0xF729 | 0xF829 | 0xF929 | 0xFA29 | 0xFB29  | 0xFC29  | 0xFD29  | 0xFE29  | 0xFF29"
    )]
    SetSpriteI(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(
        id_pat = "0xF033 | 0xF133 | 0xF233 | 0xF333 | 0xF433 | 0xF533 | 0xF633 | 0xF733 | 0xF833 | 0xF933 | 0xFA33 | 0xFB33  | 0xFC33  | 0xFD33  | 0xFE33  | 0xFF33"
    )]
    DecimalRepr(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(
        id_pat = "0xF055 | 0xF155 | 0xF255 | 0xF553 | 0xF455 | 0xF555 | 0xF655 | 0xF755 | 0xF855 | 0xF955 | 0xFA55 | 0xFB55  | 0xFC55  | 0xFD55  | 0xFE55  | 0xFF55"
    )]
    DumpRegisters(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(
        id_pat = "0xF065 | 0xF165 | 0xF265 | 0xF653 | 0xF465 | 0xF655 | 0xF665 | 0xF765 | 0xF865 | 0xF965 | 0xFA65 | 0xFB65  | 0xFC65  | 0xFD65  | 0xFE65  | 0xFF65"
    )]
    LoadRegisters(#[deku(pad_bits_before = "4", bits = "4", pad_bits_after = "8")] u8),

    #[deku(id_pat = "_")]
    Unknown(u16),
}

struct Chip8 {
    pc: u16,
    mem: Box<[u8; 1024 * 4]>,
    ireg: u16,
    stack: Vec<u16>,
    dt: u8,
    st: u8,
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
    fn new(screen: Screen) -> Self {
        let mut mem = Box::new([0; 1024 * 4]);
        mem[FONT_RANGE].copy_from_slice(FONT);

        Chip8 {
            pc: 512,
            mem,
            ireg: 0,
            stack: Vec::new(),
            dt: 0,
            st: 0,
            v: Registers([0; 16]),
            screen,
        }
    }

    fn load_program(&mut self, program: &[u8]) {
        let pc = self.pc as usize;
        self.mem[pc..pc + program.len()].copy_from_slice(program)
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
            _ => todo!("{:#04x?}", op),
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

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const N_PIXELS: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

struct Screen {
    pixels: [bool; N_PIXELS],
    output: io::Stdout,
}

impl Screen {
    const ON: &str = "\u{2588}\u{2588}";
    const OFF: &str = "  ";

    fn new(output: io::Stdout) -> Self {
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
        self.pixels.fill(false);
        self.output
            .execute(terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }
}
