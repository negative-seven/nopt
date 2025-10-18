mod instruction_compiler;
mod instruction_decoder;
mod memory_compiler;

use crate::{compiler::ir, nes::Nes};

pub(super) fn compile_instruction(nes: &mut Nes, address: u16) -> (ir::Function, bool) {
    let (nes_cpu_instruction, is_prg_rom_only) =
        instruction_decoder::decode_instruction(nes, address);
    (
        instruction_compiler::compile(nes_cpu_instruction),
        is_prg_rom_only,
    )
}
