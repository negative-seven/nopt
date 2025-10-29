mod cartridge;
mod cpu;
mod cpu_memory;
mod ppu;
mod visitor;

pub use cartridge::Cartridge;
pub(crate) use cpu::Cpu;
pub(crate) use ppu::Ppu;
pub(crate) use visitor::Visitor;

pub struct Nes {
    pub cartridge: Cartridge,
    pub cpu: Cpu,
    pub ppu: Ppu,
}

impl Nes {
    pub fn new(cartridge: Cartridge) -> Self {
        let cpu_pc = u16::from_le_bytes(
            cartridge.prg_rom[cartridge.prg_rom.len() - 0x4..][..size_of::<u16>()]
                .try_into()
                .unwrap(),
        );
        Self {
            cartridge,
            cpu: Cpu::new(cpu_pc),
            ppu: Ppu::new(),
        }
    }

    pub fn peek(&self, address: u16) -> u8 {
        match address {
            0..0x2000 => self.cpu.ram[usize::from(address) & 0x7ff],
            0x2000..0x6000 => unimplemented!("peek 0x{address:04x}"),
            0x6000..0x8000 => self.cartridge.prg_ram[usize::from(address) & 0x1fff],
            0x8000..=0xffff => {
                self.cartridge.prg_rom[usize::from(address) & (self.cartridge.prg_rom.len() - 1)]
            }
        }
    }
}
