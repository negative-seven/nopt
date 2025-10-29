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
    fn prg_rom(&self) -> &[u8] {
        &self.prg_rom
    }

    fn prg_ram(&self) -> &[u8] {
        &self.prg_ram
    }

    fn prg_ram_mut(&mut self) -> &mut [u8] {
        &mut self.prg_ram
    }
}
