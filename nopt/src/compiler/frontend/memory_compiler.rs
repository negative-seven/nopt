use crate::compiler::ir::{
    BasicBlock, Definition1, Definition8, Destination8, Instruction, Variable8, Variable16,
};

pub(super) fn compile_read(basic_block: &mut BasicBlock, address: Variable16) -> Variable8 {
    // TODO: rework without complex branchless logic

    let n0x800 = basic_block.define_16(0x800.into());
    let n0x7fff = basic_block.define_16(0x7fff.into());

    let result = basic_block.define_8(0.into());

    // read ram
    let ram_condition = basic_block.define_1(Definition1::LessThan16(address, n0x800));
    let ram_result = basic_block.define_8(Definition8::Ram(address));
    let result = basic_block.define_8(Definition8::Select {
        condition: ram_condition,
        result_if_true: ram_result,
        result_if_false: result,
    });

    // read rom
    let prg_rom_condition = basic_block.define_1(Definition1::LessThan16(n0x7fff, address));
    let prg_rom_result = basic_block.define_8(Definition8::Rom(address));
    basic_block.define_8(Definition8::Select {
        condition: prg_rom_condition,
        result_if_true: prg_rom_result,
        result_if_false: result,
    })
}

pub(super) fn compile_write(basic_block: &mut BasicBlock, address: Variable16, value: Variable8) {
    // TODO: rework without complex branchless logic

    let n0x800 = basic_block.define_16(0x800.into());
    let n0x5fff = basic_block.define_16(0x5fff.into());
    let n0x8000 = basic_block.define_16(0x8000.into());

    // write ram
    let ram_byte_old = basic_block.define_8(Definition8::Ram(address));
    let ram_condition = basic_block.define_1(Definition1::LessThan16(address, n0x800));
    let ram_byte_new = basic_block.define_8(Definition8::Select {
        condition: ram_condition,
        result_if_true: value,
        result_if_false: ram_byte_old,
    });
    basic_block.instructions.push(Instruction::Store8 {
        destination: Destination8::Ram(address),
        variable: ram_byte_new,
    });

    // write prg ram
    let prg_ram_byte_old = basic_block.define_8(Definition8::PrgRam(address));
    let prg_ram_condition = {
        let condition_a = basic_block.define_1(Definition1::LessThan16(address, n0x8000));
        let condition_b = basic_block.define_1(Definition1::LessThan16(n0x5fff, address));
        basic_block.define_1(condition_a & condition_b)
    };
    let prg_ram_byte_new = basic_block.define_8(Definition8::Select {
        condition: prg_ram_condition,
        result_if_true: value,
        result_if_false: prg_ram_byte_old,
    });
    basic_block.instructions.push(Instruction::Store8 {
        destination: Destination8::PrgRam(address),
        variable: prg_ram_byte_new,
    });
}
