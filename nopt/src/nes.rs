use crate::Rom;

pub struct Nes {
    pub rom: Rom,
    pub cpu: Cpu,
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
            cpu: Cpu::new(cpu_pc),
        }
    }

    pub fn peek(&self, address: u16) -> u8 {
        match address {
            0..0x2000 => self.cpu.ram[usize::from(address) & 0x7ff],
            0x2000..0x8000 => unimplemented!("peek 0x{address:04x}"),
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
