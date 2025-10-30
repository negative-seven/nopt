mod nrom;

pub use nrom::Nrom;

#[must_use]
pub fn from_bytes_with_header(bytes: &[u8]) -> AnyCartridge {
    let (header_bytes, rom_bytes) = bytes.split_at(0x10);

    let prg_rom_chunks = header_bytes[4];
    let (prg_rom, _chr_rom) = rom_bytes.split_at(usize::from(prg_rom_chunks) * 0x4000);

    let is_mirroring_horizontal = (header_bytes[6] & (1 << 0)) != 0;

    AnyCartridge::Nrom(Nrom::new(prg_rom, is_mirroring_horizontal))
}

pub trait Cartridge {
    fn read_is_mirroring_horizontal<Visitor: super::Visitor>(
        &self,
        visitor: &mut Visitor,
    ) -> Visitor::U1;

    fn read_prg_rom<Visitor: super::Visitor>(
        &self,
        visitor: &mut Visitor,
        address: Visitor::U16,
    ) -> Visitor::U8;

    fn read_prg_ram<Visitor: super::Visitor>(
        &self,
        visitor: &mut Visitor,
        address: Visitor::U16,
    ) -> Visitor::U8;

    fn write_prg_ram<Visitor: super::Visitor>(
        &mut self,
        visitor: &mut Visitor,
        address: Visitor::U16,
        value: Visitor::U8,
    );

    // TODO: obsolete all functions below with an interpreting Visitor

    fn reset_vector(&self) -> u16;

    fn peek_prg_rom(&self, address: u16) -> u8;

    fn peek_prg_ram(&self, address: u16) -> u8;
}

pub enum AnyCartridge {
    Nrom(Nrom),
}

impl Cartridge for AnyCartridge {
    fn read_is_mirroring_horizontal<Visitor: super::Visitor>(
        &self,
        visitor: &mut Visitor,
    ) -> Visitor::U1 {
        match self {
            AnyCartridge::Nrom(cartridge) => cartridge.read_is_mirroring_horizontal(visitor),
        }
    }

    fn read_prg_rom<Visitor: super::Visitor>(
        &self,
        visitor: &mut Visitor,
        address: Visitor::U16,
    ) -> Visitor::U8 {
        match self {
            AnyCartridge::Nrom(cartridge) => cartridge.read_prg_rom(visitor, address),
        }
    }

    fn read_prg_ram<Visitor: super::Visitor>(
        &self,
        visitor: &mut Visitor,
        address: Visitor::U16,
    ) -> Visitor::U8 {
        match self {
            AnyCartridge::Nrom(cartridge) => cartridge.read_prg_ram(visitor, address),
        }
    }

    fn write_prg_ram<Visitor: super::Visitor>(
        &mut self,
        visitor: &mut Visitor,
        address: Visitor::U16,
        value: Visitor::U8,
    ) {
        match self {
            AnyCartridge::Nrom(cartridge) => cartridge.write_prg_ram(visitor, address, value),
        }
    }

    fn reset_vector(&self) -> u16 {
        match self {
            AnyCartridge::Nrom(cartridge) => cartridge.reset_vector(),
        }
    }

    fn peek_prg_rom(&self, address: u16) -> u8 {
        match self {
            AnyCartridge::Nrom(cartridge) => cartridge.peek_prg_rom(address),
        }
    }

    fn peek_prg_ram(&self, address: u16) -> u8 {
        match self {
            AnyCartridge::Nrom(cartridge) => cartridge.peek_prg_ram(address),
        }
    }
}
