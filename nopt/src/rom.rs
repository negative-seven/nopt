pub struct Rom {
    bytes: Vec<u8>,
    header: Header,
}

impl Rom {
    #[must_use]
    pub fn from_bytes_with_header(bytes: Vec<u8>) -> Self {
        let header = Header::from_bytes(bytes[0..0x10].try_into().unwrap());
        Self { bytes, header }
    }

    #[must_use]
    pub fn prg_rom(&self) -> &[u8] {
        let start = 0x10;
        let len = usize::from(self.header.prg_rom_chunks) * 0x4000;
        &self.bytes[start..][..len]
    }
}

struct Header {
    prg_rom_chunks: u8,
}

impl Header {
    fn from_bytes(bytes: [u8; 0x10]) -> Self {
        Self {
            prg_rom_chunks: bytes[4],
        }
    }
}
