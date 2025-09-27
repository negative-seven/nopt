use crate::compiler::ir;

mod instruction_compiler;
mod instruction_decoder;
mod memory_compiler;

pub(super) fn compile_instruction<B>(bytes: B) -> ir::Function
where
    B: Iterator<Item = (u16, u8)>,
{
    let nes_cpu_instruction = instruction_decoder::decode_instruction(bytes);
    instruction_compiler::compile(std::iter::once(nes_cpu_instruction))
}
