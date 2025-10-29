pub struct Cartridge {
    pub prg_rom: [u8; 0x8000],
    pub prg_ram: [u8; 0x2000],
}

impl Cartridge {
    #[must_use]
    pub fn from_bytes_with_header(bytes: &[u8]) -> Cartridge {
        let (header_bytes, rom_bytes) = bytes.split_at(0x10);

        let prg_rom_chunks = header_bytes[4];
        let (prg_rom, _chr_rom) = rom_bytes.split_at(usize::from(prg_rom_chunks) * 0x4000);

        Cartridge::new(prg_rom)
    }

    #[must_use]
    fn new(prg_rom: &[u8]) -> Self {
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
