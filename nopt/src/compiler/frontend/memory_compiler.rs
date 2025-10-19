use std::{cell::RefCell, rc::Rc};

use crate::compiler::ir::{
    BasicBlock, Definition1, Definition8, Destination8, Instruction, Jump, Variable8, Variable16,
};

pub(super) fn compile_read(
    basic_block: &mut Rc<RefCell<BasicBlock>>,
    address: Variable16,
) -> Variable8 {
    // TODO: rework without complex branchless logic

    let n0x800 = basic_block.borrow_mut().define_16(0x800.into());
    let n0x7fff = basic_block.borrow_mut().define_16(0x7fff.into());

    let result = basic_block.borrow_mut().define_8(0.into());

    // read ram
    let ram_condition = basic_block
        .borrow_mut()
        .define_1(Definition1::LessThan16(address, n0x800));
    let ram_result = basic_block.borrow_mut().define_8(Definition8::Ram(address));
    let result = basic_block.borrow_mut().define_8(Definition8::Select {
        condition: ram_condition,
        result_if_true: ram_result,
        result_if_false: result,
    });

    // read rom
    let prg_rom_condition = basic_block
        .borrow_mut()
        .define_1(Definition1::LessThan16(n0x7fff, address));
    let prg_rom_result = basic_block.borrow_mut().define_8(Definition8::Rom(address));
    basic_block.borrow_mut().define_8(Definition8::Select {
        condition: prg_rom_condition,
        result_if_true: prg_rom_result,
        result_if_false: result,
    })
}

pub(super) fn compile_write(
    basic_block: &mut Rc<RefCell<BasicBlock>>,
    address: Variable16,
    value: Variable8,
) {
    let r#true = basic_block.borrow_mut().define_1(true.into());
    let n0x800 = basic_block.borrow_mut().define_16(0x800.into());
    let n0x5fff = basic_block.borrow_mut().define_16(0x5fff.into());
    let n0x8000 = basic_block.borrow_mut().define_16(0x8000.into());

    let variable_counter = Rc::clone(&basic_block.borrow().variable_id_counter);

    let last_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(&variable_counter))));
    last_block.borrow_mut().jump = basic_block.borrow().jump.clone();

    // write RAM
    let is_ram_condition = basic_block
        .borrow_mut()
        .define_1(Definition1::LessThan16(address, n0x800));
    let is_ram_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(&variable_counter))));
    is_ram_block
        .borrow_mut()
        .instructions
        .push(Instruction::Store8 {
            destination: Destination8::Ram(address),
            variable: value,
        });
    is_ram_block.borrow_mut().jump = Jump::BasicBlock {
        condition: r#true,
        target_if_true: Rc::clone(&last_block),
        target_if_false: Rc::clone(&last_block),
    };
    let is_not_ram_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(&variable_counter))));
    basic_block.borrow_mut().jump = Jump::BasicBlock {
        condition: is_ram_condition,
        target_if_true: Rc::clone(&is_ram_block),
        target_if_false: Rc::clone(&is_not_ram_block),
    };

    // write PRG RAM
    let is_prg_ram_condition = {
        let condition_a = is_not_ram_block
            .borrow_mut()
            .define_1(Definition1::LessThan16(address, n0x8000));
        let condition_b = is_not_ram_block
            .borrow_mut()
            .define_1(Definition1::LessThan16(n0x5fff, address));
        is_not_ram_block
            .borrow_mut()
            .define_1(condition_a & condition_b)
    };
    let is_prg_ram_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(&variable_counter))));
    is_prg_ram_block
        .borrow_mut()
        .instructions
        .push(Instruction::Store8 {
            destination: Destination8::PrgRam(address),
            variable: value,
        });
    is_prg_ram_block.borrow_mut().jump = Jump::BasicBlock {
        condition: r#true,
        target_if_true: Rc::clone(&last_block),
        target_if_false: Rc::clone(&last_block),
    };
    is_not_ram_block.borrow_mut().jump = Jump::BasicBlock {
        condition: is_prg_ram_condition,
        target_if_true: Rc::clone(&is_prg_ram_block),
        target_if_false: Rc::clone(&last_block),
    };

    *basic_block = last_block;
}
