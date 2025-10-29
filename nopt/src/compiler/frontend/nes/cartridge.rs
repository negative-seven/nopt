mod nrom;

pub use nrom::Nrom;

#[must_use]
pub fn from_bytes_with_header(bytes: &[u8]) -> AnyCartridge {
    let (header_bytes, rom_bytes) = bytes.split_at(0x10);

    let prg_rom_chunks = header_bytes[4];
    let (prg_rom, _chr_rom) = rom_bytes.split_at(usize::from(prg_rom_chunks) * 0x4000);

    AnyCartridge::Nrom(Nrom::new(prg_rom))
}

pub trait Cartridge {
    fn prg_rom(&self) -> &[u8];

    fn prg_ram(&self) -> &[u8];

    fn prg_ram_mut(&mut self) -> &mut [u8];
}

pub enum AnyCartridge {
    Nrom(Nrom),
}

impl Cartridge for AnyCartridge {
    fn prg_rom(&self) -> &[u8] {
        match self {
            AnyCartridge::Nrom(cartridge) => cartridge.prg_rom(),
        }
    }

    fn prg_ram(&self) -> &[u8] {
        match self {
            AnyCartridge::Nrom(cartridge) => cartridge.prg_ram(),
        }
    }

    fn prg_ram_mut(&mut self) -> &mut [u8] {
        match self {
            AnyCartridge::Nrom(cartridge) => cartridge.prg_ram_mut(),
        }
    }
}
