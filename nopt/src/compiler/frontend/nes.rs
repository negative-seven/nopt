pub mod cartridge;
mod cpu;
mod ppu;
mod visitor;

pub(crate) use cpu::Cpu;
pub(crate) use ppu::Ppu;
pub(crate) use visitor::Visitor;

pub struct Nes<Cartridge: cartridge::Cartridge> {
    pub cartridge: Cartridge,
    pub cpu: Cpu,
    pub ppu: Ppu,
}

impl<Cartridge: cartridge::Cartridge> Nes<Cartridge> {
    pub fn new(cartridge: Cartridge) -> Self {
        let cpu_pc = cartridge.reset_vector();
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
            0x6000..0x8000 => self.cartridge.peek_prg_ram(address & 0x1fff),
            0x8000..=0xffff => self.cartridge.peek_prg_rom(address & 0x7fff),
        }
    }
}
