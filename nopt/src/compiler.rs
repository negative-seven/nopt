mod cranelift_backend;
mod frontend;
mod ir;

use crate::nes::Nes;
use iced_x86::Formatter as _;
use memmap2::Mmap;
use std::collections::HashMap;
use tracing::trace;

pub(crate) struct Compiler {
    optimize: bool,
}

impl Compiler {
    pub(crate) fn new(optimize: bool) -> Self {
        Self { optimize }
    }

    pub(crate) fn compile(&self, nes: &mut Nes) -> (Mmap, bool) {
        let (ir, is_prg_rom_only) = frontend::compile_instruction(nes, nes.cpu.pc);
        Self::trace_ir_function(&ir);

        let bytes = cranelift_backend::compile(&ir, nes, self.optimize);

        let mut decoder = iced_x86::Decoder::new(64, &bytes, iced_x86::DecoderOptions::NONE);
        decoder.set_ip(bytes.as_ptr() as u64);
        let mut formatter = iced_x86::IntelFormatter::with_options(
            Some(Box::new(NesStateSymbolResolver::new(nes))),
            None,
        );
        for instruction in decoder {
            let mut formatted_instruction = String::new();
            formatter.format(&instruction, &mut formatted_instruction);
            trace!("native: {formatted_instruction}");
        }

        (bytes, is_prg_rom_only)
    }

    fn trace_ir_function(function: &ir::Function) {
        Self::trace_ir_basic_block(&function.basic_block);
    }

    fn trace_ir_basic_block(basic_block: &ir::BasicBlock) {
        for instruction in &basic_block.instructions {
            match instruction {
                ir::Instruction::Define1 {
                    variable,
                    definition,
                } => trace!("ir: {variable:?} = {definition:?}"),
                ir::Instruction::Define8 {
                    variable,
                    definition,
                } => trace!("ir: {variable:?} = {definition:?}"),
                ir::Instruction::Define16 {
                    variable,
                    definition,
                } => trace!("ir: {variable:?} = {definition:?}"),
                ir::Instruction::Store1 {
                    destination,
                    variable,
                } => trace!("ir: {destination:?} = {variable:?}"),
                ir::Instruction::Store8 {
                    destination,
                    variable,
                } => trace!("ir: {destination:?} = {variable:?}"),
            }
        }
        trace!("ir: {:?}", basic_block.jump);
    }
}

struct NesStateSymbolResolver(HashMap<u64, &'static str>);

impl NesStateSymbolResolver {
    pub(crate) fn new(nes: *mut Nes) -> Self {
        let mut mapping = HashMap::new();
        mapping.insert(unsafe { &raw const (*nes).cpu.a as u64 }, "cpu_a");
        mapping.insert(unsafe { &raw const (*nes).cpu.x as u64 }, "cpu_x");
        mapping.insert(unsafe { &raw const (*nes).cpu.y as u64 }, "cpu_y");
        mapping.insert(unsafe { &raw const (*nes).cpu.s as u64 }, "cpu_s");
        mapping.insert(unsafe { &raw const (*nes).cpu.p as u64 }, "cpu_p");
        mapping.insert(unsafe { &raw const (*nes).cpu.pc as u64 }, "cpu_pc");
        mapping.insert(unsafe { (*nes).cpu.ram.as_ptr() as u64 }, "cpu_ram");
        Self(mapping)
    }
}

impl iced_x86::SymbolResolver for NesStateSymbolResolver {
    fn symbol(
        &mut self,
        _instruction: &iced_x86::Instruction,
        _operand: u32,
        _instruction_operand: Option<u32>,
        address: u64,
        _address_size: u32,
    ) -> Option<iced_x86::SymbolResult<'_>> {
        self.0
            .get(&address)
            .map(|symbol| iced_x86::SymbolResult::with_str(address, symbol))
    }
}
