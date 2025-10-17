use std::mem::MaybeUninit;

pub(crate) struct Instruction {
    address: u16,
    operation: Operation,
    operand: u16,
}

impl Instruction {
    pub(crate) fn new(address: u16, operation: Operation, operand: u16) -> Self {
        Self {
            address,
            operation,
            operand,
        }
    }

    pub(crate) fn address_end(&self) -> u16 {
        self.address.wrapping_add(u16::from(self.operation.len()))
    }

    pub(crate) fn operation(&self) -> Operation {
        self.operation
    }

    pub(crate) fn operand_u8(&self) -> u8 {
        self.operand.to_le_bytes()[0]
    }

    #[expect(clippy::cast_possible_wrap)]
    pub(crate) fn operand_i8(&self) -> i8 {
        self.operand_u8() as i8
    }

    pub(crate) fn operand_u16(&self) -> u16 {
        self.operand
    }
}

impl std::fmt::Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.operation.mnemonic)?;
        match self.operation.addressing_mode {
            AddressingMode::Absolute => write!(f, " ${:04x}", self.operand),
            AddressingMode::AbsoluteX => write!(f, " ${:04x},x", self.operand),
            AddressingMode::AbsoluteY => write!(f, " ${:04x},y", self.operand),
            AddressingMode::Accumulator => write!(f, " a"),
            AddressingMode::Immediate => write!(f, " #${:02x}", self.operand),
            AddressingMode::Implied => write!(f, ""),
            AddressingMode::Indirect => write!(f, " (${:04x})", self.operand),
            AddressingMode::IndirectY => write!(f, " (${:02x}),y", self.operand),
            AddressingMode::Relative => {
                write!(
                    f,
                    " ${:04x}",
                    self.address_end()
                        .wrapping_add_signed(self.operand_i8().into())
                )
            }
            AddressingMode::XIndirect => write!(f, " (${:02x},x)", self.operand),
            AddressingMode::Zeropage => write!(f, " ${:02x}", self.operand),
            AddressingMode::ZeropageX => write!(f, " ${:02x},x", self.operand),
            AddressingMode::ZeropageY => write!(f, " ${:02x},y", self.operand),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Operation {
    mnemonic: Mnemonic,
    addressing_mode: AddressingMode,
}

impl Operation {
    #[expect(clippy::too_many_lines)]
    pub(crate) const fn from_opcode(opcode: u8) -> Self {
        const OPCODE_TO_OPERATION: [Operation; 256] = {
            let data = [
                (0x00, Mnemonic::Brk, AddressingMode::Implied),
                (0x01, Mnemonic::Ora, AddressingMode::XIndirect),
                (0x02, Mnemonic::Unimplemented, AddressingMode::Implied),
                (0x03, Mnemonic::Unimplemented, AddressingMode::XIndirect),
                (0x04, Mnemonic::Unimplemented, AddressingMode::Zeropage),
                (0x05, Mnemonic::Ora, AddressingMode::Zeropage),
                (0x06, Mnemonic::Asl, AddressingMode::Zeropage),
                (0x07, Mnemonic::Unimplemented, AddressingMode::Zeropage),
                (0x08, Mnemonic::Php, AddressingMode::Implied),
                (0x09, Mnemonic::Ora, AddressingMode::Immediate),
                (0x0a, Mnemonic::Asl, AddressingMode::Accumulator),
                (0x0b, Mnemonic::Unimplemented, AddressingMode::Immediate),
                (0x0c, Mnemonic::Unimplemented, AddressingMode::Absolute),
                (0x0d, Mnemonic::Ora, AddressingMode::Absolute),
                (0x0e, Mnemonic::Asl, AddressingMode::Absolute),
                (0x0f, Mnemonic::Unimplemented, AddressingMode::Absolute),
                (0x10, Mnemonic::Bpl, AddressingMode::Relative),
                (0x11, Mnemonic::Ora, AddressingMode::IndirectY),
                (0x12, Mnemonic::Unimplemented, AddressingMode::Implied),
                (0x13, Mnemonic::Unimplemented, AddressingMode::IndirectY),
                (0x14, Mnemonic::Unimplemented, AddressingMode::ZeropageX),
                (0x15, Mnemonic::Ora, AddressingMode::ZeropageX),
                (0x16, Mnemonic::Asl, AddressingMode::ZeropageX),
                (0x17, Mnemonic::Unimplemented, AddressingMode::ZeropageX),
                (0x18, Mnemonic::Clc, AddressingMode::Implied),
                (0x19, Mnemonic::Ora, AddressingMode::AbsoluteY),
                (0x1a, Mnemonic::Unimplemented, AddressingMode::Implied),
                (0x1b, Mnemonic::Unimplemented, AddressingMode::AbsoluteY),
                (0x1c, Mnemonic::Unimplemented, AddressingMode::AbsoluteX),
                (0x1d, Mnemonic::Ora, AddressingMode::AbsoluteX),
                (0x1e, Mnemonic::Asl, AddressingMode::AbsoluteX),
                (0x1f, Mnemonic::Unimplemented, AddressingMode::AbsoluteX),
                (0x20, Mnemonic::Jsr, AddressingMode::Absolute),
                (0x21, Mnemonic::And, AddressingMode::XIndirect),
                (0x22, Mnemonic::Unimplemented, AddressingMode::Implied),
                (0x23, Mnemonic::Unimplemented, AddressingMode::XIndirect),
                (0x24, Mnemonic::Bit, AddressingMode::Zeropage),
                (0x25, Mnemonic::And, AddressingMode::Zeropage),
                (0x26, Mnemonic::Rol, AddressingMode::Zeropage),
                (0x27, Mnemonic::Unimplemented, AddressingMode::Zeropage),
                (0x28, Mnemonic::Plp, AddressingMode::Implied),
                (0x29, Mnemonic::And, AddressingMode::Immediate),
                (0x2a, Mnemonic::Rol, AddressingMode::Accumulator),
                (0x2b, Mnemonic::Unimplemented, AddressingMode::Immediate),
                (0x2c, Mnemonic::Bit, AddressingMode::Absolute),
                (0x2d, Mnemonic::And, AddressingMode::Absolute),
                (0x2e, Mnemonic::Rol, AddressingMode::Absolute),
                (0x2f, Mnemonic::Unimplemented, AddressingMode::Absolute),
                (0x30, Mnemonic::Bmi, AddressingMode::Relative),
                (0x31, Mnemonic::And, AddressingMode::IndirectY),
                (0x32, Mnemonic::Unimplemented, AddressingMode::Implied),
                (0x33, Mnemonic::Unimplemented, AddressingMode::IndirectY),
                (0x34, Mnemonic::Unimplemented, AddressingMode::ZeropageX),
                (0x35, Mnemonic::And, AddressingMode::ZeropageX),
                (0x36, Mnemonic::Rol, AddressingMode::ZeropageX),
                (0x37, Mnemonic::Unimplemented, AddressingMode::ZeropageX),
                (0x38, Mnemonic::Sec, AddressingMode::Implied),
                (0x39, Mnemonic::And, AddressingMode::AbsoluteY),
                (0x3a, Mnemonic::Unimplemented, AddressingMode::Implied),
                (0x3b, Mnemonic::Unimplemented, AddressingMode::AbsoluteY),
                (0x3c, Mnemonic::Unimplemented, AddressingMode::AbsoluteX),
                (0x3d, Mnemonic::And, AddressingMode::AbsoluteX),
                (0x3e, Mnemonic::Rol, AddressingMode::AbsoluteX),
                (0x3f, Mnemonic::Unimplemented, AddressingMode::AbsoluteX),
                (0x40, Mnemonic::Rti, AddressingMode::Implied),
                (0x41, Mnemonic::Eor, AddressingMode::XIndirect),
                (0x42, Mnemonic::Unimplemented, AddressingMode::Implied),
                (0x43, Mnemonic::Unimplemented, AddressingMode::XIndirect),
                (0x44, Mnemonic::Unimplemented, AddressingMode::Zeropage),
                (0x45, Mnemonic::Eor, AddressingMode::Zeropage),
                (0x46, Mnemonic::Lsr, AddressingMode::Zeropage),
                (0x47, Mnemonic::Unimplemented, AddressingMode::Zeropage),
                (0x48, Mnemonic::Pha, AddressingMode::Implied),
                (0x49, Mnemonic::Eor, AddressingMode::Immediate),
                (0x4a, Mnemonic::Lsr, AddressingMode::Accumulator),
                (0x4b, Mnemonic::Unimplemented, AddressingMode::Immediate),
                (0x4c, Mnemonic::Jmp, AddressingMode::Absolute),
                (0x4d, Mnemonic::Eor, AddressingMode::Absolute),
                (0x4e, Mnemonic::Lsr, AddressingMode::Absolute),
                (0x4f, Mnemonic::Unimplemented, AddressingMode::Absolute),
                (0x50, Mnemonic::Bvc, AddressingMode::Relative),
                (0x51, Mnemonic::Eor, AddressingMode::IndirectY),
                (0x52, Mnemonic::Unimplemented, AddressingMode::Implied),
                (0x53, Mnemonic::Unimplemented, AddressingMode::IndirectY),
                (0x54, Mnemonic::Unimplemented, AddressingMode::ZeropageX),
                (0x55, Mnemonic::Eor, AddressingMode::ZeropageX),
                (0x56, Mnemonic::Lsr, AddressingMode::ZeropageX),
                (0x57, Mnemonic::Unimplemented, AddressingMode::ZeropageX),
                (0x58, Mnemonic::Cli, AddressingMode::Implied),
                (0x59, Mnemonic::Eor, AddressingMode::AbsoluteY),
                (0x5a, Mnemonic::Unimplemented, AddressingMode::Implied),
                (0x5b, Mnemonic::Unimplemented, AddressingMode::AbsoluteY),
                (0x5c, Mnemonic::Unimplemented, AddressingMode::AbsoluteX),
                (0x5d, Mnemonic::Eor, AddressingMode::AbsoluteX),
                (0x5e, Mnemonic::Lsr, AddressingMode::AbsoluteX),
                (0x5f, Mnemonic::Unimplemented, AddressingMode::AbsoluteX),
                (0x60, Mnemonic::Rts, AddressingMode::Implied),
                (0x61, Mnemonic::Adc, AddressingMode::XIndirect),
                (0x62, Mnemonic::Unimplemented, AddressingMode::Implied),
                (0x63, Mnemonic::Unimplemented, AddressingMode::XIndirect),
                (0x64, Mnemonic::Unimplemented, AddressingMode::Zeropage),
                (0x65, Mnemonic::Adc, AddressingMode::Zeropage),
                (0x66, Mnemonic::Ror, AddressingMode::Zeropage),
                (0x67, Mnemonic::Unimplemented, AddressingMode::Zeropage),
                (0x68, Mnemonic::Pla, AddressingMode::Implied),
                (0x69, Mnemonic::Adc, AddressingMode::Immediate),
                (0x6a, Mnemonic::Ror, AddressingMode::Accumulator),
                (0x6b, Mnemonic::Unimplemented, AddressingMode::Immediate),
                (0x6c, Mnemonic::Jmp, AddressingMode::Indirect),
                (0x6d, Mnemonic::Adc, AddressingMode::Absolute),
                (0x6e, Mnemonic::Ror, AddressingMode::Absolute),
                (0x6f, Mnemonic::Unimplemented, AddressingMode::Absolute),
                (0x70, Mnemonic::Bvs, AddressingMode::Relative),
                (0x71, Mnemonic::Adc, AddressingMode::IndirectY),
                (0x72, Mnemonic::Unimplemented, AddressingMode::Implied),
                (0x73, Mnemonic::Unimplemented, AddressingMode::IndirectY),
                (0x74, Mnemonic::Unimplemented, AddressingMode::ZeropageX),
                (0x75, Mnemonic::Adc, AddressingMode::ZeropageX),
                (0x76, Mnemonic::Ror, AddressingMode::ZeropageX),
                (0x77, Mnemonic::Unimplemented, AddressingMode::ZeropageX),
                (0x78, Mnemonic::Sei, AddressingMode::Implied),
                (0x79, Mnemonic::Adc, AddressingMode::AbsoluteY),
                (0x7a, Mnemonic::Unimplemented, AddressingMode::Implied),
                (0x7b, Mnemonic::Unimplemented, AddressingMode::AbsoluteY),
                (0x7c, Mnemonic::Unimplemented, AddressingMode::AbsoluteX),
                (0x7d, Mnemonic::Adc, AddressingMode::AbsoluteX),
                (0x7e, Mnemonic::Ror, AddressingMode::AbsoluteX),
                (0x7f, Mnemonic::Unimplemented, AddressingMode::AbsoluteX),
                (0x80, Mnemonic::Unimplemented, AddressingMode::Immediate),
                (0x81, Mnemonic::Sta, AddressingMode::XIndirect),
                (0x82, Mnemonic::Unimplemented, AddressingMode::Immediate),
                (0x83, Mnemonic::Unimplemented, AddressingMode::XIndirect),
                (0x84, Mnemonic::Sty, AddressingMode::Zeropage),
                (0x85, Mnemonic::Sta, AddressingMode::Zeropage),
                (0x86, Mnemonic::Stx, AddressingMode::Zeropage),
                (0x87, Mnemonic::Unimplemented, AddressingMode::Zeropage),
                (0x88, Mnemonic::Dey, AddressingMode::Implied),
                (0x89, Mnemonic::Unimplemented, AddressingMode::Immediate),
                (0x8a, Mnemonic::Txa, AddressingMode::Implied),
                (0x8b, Mnemonic::Unimplemented, AddressingMode::Immediate),
                (0x8c, Mnemonic::Sty, AddressingMode::Absolute),
                (0x8d, Mnemonic::Sta, AddressingMode::Absolute),
                (0x8e, Mnemonic::Stx, AddressingMode::Absolute),
                (0x8f, Mnemonic::Unimplemented, AddressingMode::Absolute),
                (0x90, Mnemonic::Bcc, AddressingMode::Relative),
                (0x91, Mnemonic::Sta, AddressingMode::IndirectY),
                (0x92, Mnemonic::Unimplemented, AddressingMode::Implied),
                (0x93, Mnemonic::Unimplemented, AddressingMode::IndirectY),
                (0x94, Mnemonic::Sty, AddressingMode::ZeropageX),
                (0x95, Mnemonic::Sta, AddressingMode::ZeropageX),
                (0x96, Mnemonic::Stx, AddressingMode::ZeropageY),
                (0x97, Mnemonic::Unimplemented, AddressingMode::ZeropageY),
                (0x98, Mnemonic::Tya, AddressingMode::Implied),
                (0x99, Mnemonic::Sta, AddressingMode::AbsoluteY),
                (0x9a, Mnemonic::Txs, AddressingMode::Implied),
                (0x9b, Mnemonic::Unimplemented, AddressingMode::AbsoluteY),
                (0x9c, Mnemonic::Unimplemented, AddressingMode::AbsoluteX),
                (0x9d, Mnemonic::Sta, AddressingMode::AbsoluteX),
                (0x9e, Mnemonic::Unimplemented, AddressingMode::AbsoluteY),
                (0x9f, Mnemonic::Unimplemented, AddressingMode::AbsoluteY),
                (0xa0, Mnemonic::Ldy, AddressingMode::Immediate),
                (0xa1, Mnemonic::Lda, AddressingMode::XIndirect),
                (0xa2, Mnemonic::Ldx, AddressingMode::Immediate),
                (0xa3, Mnemonic::Unimplemented, AddressingMode::XIndirect),
                (0xa4, Mnemonic::Ldy, AddressingMode::Zeropage),
                (0xa5, Mnemonic::Lda, AddressingMode::Zeropage),
                (0xa6, Mnemonic::Ldx, AddressingMode::Zeropage),
                (0xa7, Mnemonic::Unimplemented, AddressingMode::Zeropage),
                (0xa8, Mnemonic::Tay, AddressingMode::Implied),
                (0xa9, Mnemonic::Lda, AddressingMode::Immediate),
                (0xaa, Mnemonic::Tax, AddressingMode::Implied),
                (0xab, Mnemonic::Unimplemented, AddressingMode::Immediate),
                (0xac, Mnemonic::Ldy, AddressingMode::Absolute),
                (0xad, Mnemonic::Lda, AddressingMode::Absolute),
                (0xae, Mnemonic::Ldx, AddressingMode::Absolute),
                (0xaf, Mnemonic::Unimplemented, AddressingMode::Absolute),
                (0xb0, Mnemonic::Bcs, AddressingMode::Relative),
                (0xb1, Mnemonic::Lda, AddressingMode::IndirectY),
                (0xb2, Mnemonic::Unimplemented, AddressingMode::Implied),
                (0xb3, Mnemonic::Unimplemented, AddressingMode::IndirectY),
                (0xb4, Mnemonic::Ldy, AddressingMode::ZeropageX),
                (0xb5, Mnemonic::Lda, AddressingMode::ZeropageX),
                (0xb6, Mnemonic::Ldx, AddressingMode::ZeropageY),
                (0xb7, Mnemonic::Unimplemented, AddressingMode::ZeropageY),
                (0xb8, Mnemonic::Clv, AddressingMode::Implied),
                (0xb9, Mnemonic::Lda, AddressingMode::AbsoluteY),
                (0xba, Mnemonic::Tsx, AddressingMode::Implied),
                (0xbb, Mnemonic::Unimplemented, AddressingMode::AbsoluteY),
                (0xbc, Mnemonic::Ldy, AddressingMode::AbsoluteX),
                (0xbd, Mnemonic::Lda, AddressingMode::AbsoluteX),
                (0xbe, Mnemonic::Ldx, AddressingMode::AbsoluteY),
                (0xbf, Mnemonic::Unimplemented, AddressingMode::AbsoluteY),
                (0xc0, Mnemonic::Cpy, AddressingMode::Immediate),
                (0xc1, Mnemonic::Cmp, AddressingMode::XIndirect),
                (0xc2, Mnemonic::Unimplemented, AddressingMode::Immediate),
                (0xc3, Mnemonic::Unimplemented, AddressingMode::XIndirect),
                (0xc4, Mnemonic::Cpy, AddressingMode::Zeropage),
                (0xc5, Mnemonic::Cmp, AddressingMode::Zeropage),
                (0xc6, Mnemonic::Dec, AddressingMode::Zeropage),
                (0xc7, Mnemonic::Unimplemented, AddressingMode::Zeropage),
                (0xc8, Mnemonic::Iny, AddressingMode::Implied),
                (0xc9, Mnemonic::Cmp, AddressingMode::Immediate),
                (0xca, Mnemonic::Dex, AddressingMode::Implied),
                (0xcb, Mnemonic::Unimplemented, AddressingMode::Immediate),
                (0xcc, Mnemonic::Cpy, AddressingMode::Absolute),
                (0xcd, Mnemonic::Cmp, AddressingMode::Absolute),
                (0xce, Mnemonic::Dec, AddressingMode::Absolute),
                (0xcf, Mnemonic::Unimplemented, AddressingMode::Absolute),
                (0xd0, Mnemonic::Bne, AddressingMode::Relative),
                (0xd1, Mnemonic::Cmp, AddressingMode::IndirectY),
                (0xd2, Mnemonic::Unimplemented, AddressingMode::Implied),
                (0xd3, Mnemonic::Unimplemented, AddressingMode::IndirectY),
                (0xd4, Mnemonic::Unimplemented, AddressingMode::ZeropageX),
                (0xd5, Mnemonic::Cmp, AddressingMode::ZeropageX),
                (0xd6, Mnemonic::Dec, AddressingMode::ZeropageX),
                (0xd7, Mnemonic::Unimplemented, AddressingMode::ZeropageX),
                (0xd8, Mnemonic::Cld, AddressingMode::Implied),
                (0xd9, Mnemonic::Cmp, AddressingMode::AbsoluteY),
                (0xda, Mnemonic::Unimplemented, AddressingMode::Implied),
                (0xdb, Mnemonic::Unimplemented, AddressingMode::AbsoluteY),
                (0xdc, Mnemonic::Unimplemented, AddressingMode::AbsoluteX),
                (0xdd, Mnemonic::Cmp, AddressingMode::AbsoluteX),
                (0xde, Mnemonic::Dec, AddressingMode::AbsoluteX),
                (0xdf, Mnemonic::Unimplemented, AddressingMode::AbsoluteX),
                (0xe0, Mnemonic::Cpx, AddressingMode::Immediate),
                (0xe1, Mnemonic::Sbc, AddressingMode::XIndirect),
                (0xe2, Mnemonic::Unimplemented, AddressingMode::Immediate),
                (0xe3, Mnemonic::Unimplemented, AddressingMode::XIndirect),
                (0xe4, Mnemonic::Cpx, AddressingMode::Zeropage),
                (0xe5, Mnemonic::Sbc, AddressingMode::Zeropage),
                (0xe6, Mnemonic::Inc, AddressingMode::Zeropage),
                (0xe7, Mnemonic::Unimplemented, AddressingMode::Zeropage),
                (0xe8, Mnemonic::Inx, AddressingMode::Implied),
                (0xe9, Mnemonic::Sbc, AddressingMode::Immediate),
                (0xea, Mnemonic::Nop, AddressingMode::Implied),
                (0xeb, Mnemonic::Unimplemented, AddressingMode::Immediate),
                (0xec, Mnemonic::Cpx, AddressingMode::Absolute),
                (0xed, Mnemonic::Sbc, AddressingMode::Absolute),
                (0xee, Mnemonic::Inc, AddressingMode::Absolute),
                (0xef, Mnemonic::Unimplemented, AddressingMode::Absolute),
                (0xf0, Mnemonic::Beq, AddressingMode::Relative),
                (0xf1, Mnemonic::Sbc, AddressingMode::IndirectY),
                (0xf2, Mnemonic::Unimplemented, AddressingMode::Implied),
                (0xf3, Mnemonic::Unimplemented, AddressingMode::IndirectY),
                (0xf4, Mnemonic::Unimplemented, AddressingMode::ZeropageX),
                (0xf5, Mnemonic::Sbc, AddressingMode::ZeropageX),
                (0xf6, Mnemonic::Inc, AddressingMode::ZeropageX),
                (0xf7, Mnemonic::Unimplemented, AddressingMode::ZeropageX),
                (0xf8, Mnemonic::Sed, AddressingMode::Implied),
                (0xf9, Mnemonic::Sbc, AddressingMode::AbsoluteY),
                (0xfa, Mnemonic::Unimplemented, AddressingMode::Implied),
                (0xfb, Mnemonic::Unimplemented, AddressingMode::AbsoluteY),
                (0xfc, Mnemonic::Unimplemented, AddressingMode::AbsoluteX),
                (0xfd, Mnemonic::Sbc, AddressingMode::AbsoluteX),
                (0xfe, Mnemonic::Inc, AddressingMode::AbsoluteX),
                (0xff, Mnemonic::Unimplemented, AddressingMode::AbsoluteX),
            ];

            let mut mapping = [MaybeUninit::uninit(); 256];

            let mut index = 0;
            while index < mapping.len() {
                let (opcode, mnemonic, addressing_mode) = data[index];
                assert!(opcode == index);
                mapping[index] = MaybeUninit::new(Operation {
                    mnemonic,
                    addressing_mode,
                });

                index += 1;
            }

            unsafe { std::mem::transmute(mapping) }
        };

        OPCODE_TO_OPERATION[opcode as usize]
    }

    pub(crate) fn mnemonic(self) -> Mnemonic {
        self.mnemonic
    }

    pub(crate) fn addressing_mode(self) -> AddressingMode {
        self.addressing_mode
    }

    pub(crate) fn len(self) -> u8 {
        1 + self.addressing_mode.len()
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Mnemonic {
    Adc,
    And,
    Asl,
    Bcc,
    Bcs,
    Beq,
    Bit,
    Bmi,
    Bne,
    Bpl,
    Brk,
    Bvc,
    Bvs,
    Clc,
    Cld,
    Cli,
    Clv,
    Cmp,
    Cpx,
    Cpy,
    Dec,
    Dex,
    Dey,
    Eor,
    Inc,
    Inx,
    Iny,
    Jmp,
    Jsr,
    Lda,
    Ldx,
    Ldy,
    Lsr,
    Nop,
    Ora,
    Pha,
    Php,
    Pla,
    Plp,
    Rol,
    Ror,
    Rti,
    Rts,
    Sbc,
    Sec,
    Sed,
    Sei,
    Sta,
    Stx,
    Sty,
    Tax,
    Tay,
    Tsx,
    Txa,
    Txs,
    Tya,
    Unimplemented,
}

impl std::fmt::Debug for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Adc => write!(f, "adc"),
            Self::And => write!(f, "and"),
            Self::Asl => write!(f, "asl"),
            Self::Bcc => write!(f, "bcc"),
            Self::Bcs => write!(f, "bcs"),
            Self::Beq => write!(f, "beq"),
            Self::Bit => write!(f, "bit"),
            Self::Bmi => write!(f, "bmi"),
            Self::Bne => write!(f, "bne"),
            Self::Bpl => write!(f, "bpl"),
            Self::Brk => write!(f, "brk"),
            Self::Bvc => write!(f, "bvc"),
            Self::Bvs => write!(f, "bvs"),
            Self::Clc => write!(f, "clc"),
            Self::Cld => write!(f, "cld"),
            Self::Cli => write!(f, "cli"),
            Self::Clv => write!(f, "clv"),
            Self::Cmp => write!(f, "cmp"),
            Self::Cpx => write!(f, "cpx"),
            Self::Cpy => write!(f, "cpy"),
            Self::Dec => write!(f, "dec"),
            Self::Dex => write!(f, "dex"),
            Self::Dey => write!(f, "dey"),
            Self::Eor => write!(f, "eor"),
            Self::Inc => write!(f, "inc"),
            Self::Inx => write!(f, "inx"),
            Self::Iny => write!(f, "iny"),
            Self::Jmp => write!(f, "jmp"),
            Self::Jsr => write!(f, "jsr"),
            Self::Lda => write!(f, "lda"),
            Self::Ldx => write!(f, "ldx"),
            Self::Ldy => write!(f, "ldy"),
            Self::Lsr => write!(f, "lsr"),
            Self::Nop => write!(f, "nop"),
            Self::Ora => write!(f, "ora"),
            Self::Pha => write!(f, "pha"),
            Self::Php => write!(f, "php"),
            Self::Pla => write!(f, "pla"),
            Self::Plp => write!(f, "plp"),
            Self::Rol => write!(f, "rol"),
            Self::Ror => write!(f, "ror"),
            Self::Rti => write!(f, "rti"),
            Self::Rts => write!(f, "rts"),
            Self::Sbc => write!(f, "sbc"),
            Self::Sec => write!(f, "sec"),
            Self::Sed => write!(f, "sed"),
            Self::Sei => write!(f, "sei"),
            Self::Sta => write!(f, "sta"),
            Self::Stx => write!(f, "stx"),
            Self::Sty => write!(f, "sty"),
            Self::Tax => write!(f, "tax"),
            Self::Tay => write!(f, "tay"),
            Self::Tsx => write!(f, "tsx"),
            Self::Txa => write!(f, "txa"),
            Self::Txs => write!(f, "txs"),
            Self::Tya => write!(f, "tya"),
            Self::Unimplemented => write!(f, "???"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum AddressingMode {
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Accumulator,
    Immediate,
    Implied,
    Indirect,
    IndirectY,
    Relative,
    XIndirect,
    Zeropage,
    ZeropageX,
    ZeropageY,
}

impl AddressingMode {
    pub(crate) const fn len(self) -> u8 {
        match self {
            AddressingMode::Accumulator | AddressingMode::Implied => 0,
            AddressingMode::Immediate
            | AddressingMode::IndirectY
            | AddressingMode::Relative
            | AddressingMode::XIndirect
            | AddressingMode::Zeropage
            | AddressingMode::ZeropageX
            | AddressingMode::ZeropageY => 1,
            AddressingMode::Absolute
            | AddressingMode::AbsoluteX
            | AddressingMode::AbsoluteY
            | AddressingMode::Indirect => 2,
        }
    }
}
