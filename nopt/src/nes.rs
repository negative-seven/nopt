use crate::Rom;

pub struct Nes {
    pub rom: Rom,
    pub prg_ram: [u8; 0x2000],
    pub cpu: Cpu,
    pub ppu: Ppu,
}

impl Nes {
    pub fn new(rom: Rom) -> Self {
        let cpu_pc = u16::from_le_bytes(
            rom.prg_rom()[rom.prg_rom().len() - 0x4..][..size_of::<u16>()]
                .try_into()
                .unwrap(),
        );
        Self {
            rom,
            prg_ram: [0; 0x2000],
            cpu: Cpu::new(cpu_pc),
            ppu: Ppu::new(),
        }
    }

    pub fn peek(&self, address: u16) -> u8 {
        match address {
            0..0x2000 => self.cpu.ram[usize::from(address) & 0x7ff],
            0x2000..0x6000 => unimplemented!("peek 0x{address:04x}"),
            0x6000..0x8000 => self.prg_ram[usize::from(address) & 0x1fff],
            0x8000..=0xffff => {
                self.rom.prg_rom()[usize::from(address) & (self.rom.prg_rom().len() - 1)]
            }
        }
    }
}

pub struct Cpu {
    pub ram: [u8; 0x800],
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
    pub s: u8,
    pub pc: u16,
}

impl Cpu {
    pub fn new(pc: u16) -> Self {
        Cpu {
            ram: [0; 0x800],
            a: 0,
            x: 0,
            y: 0,
            p: 0,
            s: 0,
            pc,
        }
    }
}

pub struct Ppu {
    pub ram: [u8; 0x1000],
    pub palette_ram: [u8; 0x20],
    pub control_register: u8,
    pub read_buffer: u8,
    pub current_address: u16,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            ram: [0; 0x1000],
            palette_ram: [0; 0x20],
            control_register: 0,
            read_buffer: 0,
            current_address: 0,
        }
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}
