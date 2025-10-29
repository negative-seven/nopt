mod cpu;
mod cpu_memory;
mod ppu;
mod visitor;

use crate::Rom;
pub(crate) use cpu::Cpu;
pub(crate) use ppu::Ppu;
pub(crate) use visitor::Visitor;

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
