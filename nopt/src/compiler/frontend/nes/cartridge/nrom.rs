use crate::cartridge::Cartridge;

pub struct Nrom {
    prg_ram: [u8; 0x2000],
    prg_rom: [u8; 0x8000],
}

impl Nrom {
    #[must_use]
    pub fn new(prg_rom: &[u8]) -> Self {
        Self {
            prg_ram: [0; 0x2000],
            prg_rom: match prg_rom.len() {
                0x4000 => [prg_rom, prg_rom].concat().try_into().unwrap(),
                0x8000 => prg_rom.try_into().unwrap(),
                _ => unimplemented!("NROM cartridge with PRG ROM size 0x{:x}", prg_rom.len()),
            },
        }
    }
}

impl Cartridge for Nrom {
    fn read_prg_rom<Visitor: crate::compiler::frontend::Visitor>(
        &self,
        visitor: &mut Visitor,
        address: Visitor::U16,
    ) -> Visitor::U8 {
        visitor.memory_with_offset_u8(self.prg_rom.as_ptr(), address)
    }

    fn read_prg_ram<Visitor: crate::compiler::frontend::Visitor>(
        &self,
        visitor: &mut Visitor,
        address: Visitor::U16,
    ) -> Visitor::U8 {
        visitor.memory_with_offset_u8(self.prg_ram.as_ptr(), address)
    }

    fn write_prg_ram<Visitor: crate::compiler::frontend::Visitor>(
        &mut self,
        visitor: &mut Visitor,
        address: Visitor::U16,
        value: Visitor::U8,
    ) {
        visitor.set_memory_with_offset_u8(self.prg_ram.as_mut_ptr(), address, value);
    }

    fn reset_vector(&self) -> u16 {
        u16::from_le_bytes(
            self.prg_rom[self.prg_rom.len() - 0x4..][..size_of::<u16>()]
                .try_into()
                .unwrap(),
        )
    }

    fn peek_prg_rom(&self, address: u16) -> u8 {
        self.prg_rom[usize::from(address)]
    }

    fn peek_prg_ram(&self, address: u16) -> u8 {
        self.prg_ram[usize::from(address)]
    }
}
