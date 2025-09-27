use tracing::trace;

use crate::nes_assembly;

pub(super) fn decode_instruction<B>(mut bytes: B) -> nes_assembly::Instruction
where
    B: Iterator<Item = (u16, u8)>,
{
    let mut next_byte = || {
        bytes
            .next()
            .expect("not enough bytes to fetch next instruction")
    };

    let (address, opcode) = next_byte();
    let Some(operation) = nes_assembly::Operation::from_opcode(opcode) else {
        unimplemented!("opcode {opcode:#x}");
    };

    let operand = match operation.addressing_mode().len() {
        0 => 0,
        1 => u16::from(next_byte().1),
        2 => u16::from_le_bytes([next_byte().1, next_byte().1]),
        _ => unreachable!(),
    };

    let instruction = nes_assembly::Instruction::new(address, operation, operand);
    trace!("instruction: {instruction:?}");
    instruction
}
