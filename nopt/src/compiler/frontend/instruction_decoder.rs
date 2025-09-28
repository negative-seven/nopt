use tracing::trace;

use crate::{nes::Nes, nes_assembly};

pub(super) fn decode_instruction(nes: &mut Nes, address: u16) -> (nes_assembly::Instruction, bool) {
    let mut next_byte_address = address;
    let mut is_prg_rom_only = true;
    let mut next_byte = || {
        if next_byte_address < 0x8000 {
            is_prg_rom_only = false;
        }

        let byte = nes.peek(next_byte_address);
        next_byte_address += 1;
        byte
    };

    let opcode = next_byte();
    let Some(operation) = nes_assembly::Operation::from_opcode(opcode) else {
        unimplemented!("opcode {opcode:#x}");
    };

    let operand = match operation.addressing_mode().len() {
        0 => 0,
        1 => u16::from(next_byte()),
        2 => u16::from_le_bytes([next_byte(), next_byte()]),
        _ => unreachable!(),
    };

    let instruction = nes_assembly::Instruction::new(address, operation, operand);
    debug_assert_eq!(instruction.address_end(), next_byte_address);
    trace!("instruction: {instruction:?}");
    (instruction, is_prg_rom_only)
}
