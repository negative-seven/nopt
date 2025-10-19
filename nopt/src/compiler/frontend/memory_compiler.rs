use std::{cell::RefCell, rc::Rc};

use crate::compiler::ir::{
    BasicBlock, Definition1, Definition8, Destination8, Instruction, Jump, Variable8, Variable16,
};

pub(super) fn compile_read(
    basic_block: &mut Rc<RefCell<BasicBlock>>,
    address: Variable16,
) -> Variable8 {
    let r#true = basic_block.borrow_mut().define_1(true.into());
    let n0x800 = basic_block.borrow_mut().define_16(0x800.into());
    let n0x5fff = basic_block.borrow_mut().define_16(0x5fff.into());
    let n0x7fff = basic_block.borrow_mut().define_16(0x7fff.into());
    let n0x8000 = basic_block.borrow_mut().define_16(0x8000.into());

    let variable_counter = Rc::clone(&basic_block.borrow().variable_id_counter);

    let last_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(&variable_counter))));
    last_block.borrow_mut().set_has_argument(true);
    last_block.borrow_mut().jump = basic_block.borrow().jump.clone();

    // read RAM
    let is_ram_condition = basic_block
        .borrow_mut()
        .define_1(Definition1::LessThan16(address, n0x800));
    let is_ram_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(&variable_counter))));
    let ram_value = is_ram_block
        .borrow_mut()
        .define_8(Definition8::Ram(address));
    is_ram_block.borrow_mut().jump = Jump::BasicBlock {
        condition: r#true,
        target_if_true: Rc::clone(&last_block),
        target_if_true_argument: Some(ram_value),
        target_if_false: Rc::clone(&last_block),
        target_if_false_argument: Some(ram_value),
    };
    let is_not_ram_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(&variable_counter))));
    basic_block.borrow_mut().jump = Jump::BasicBlock {
        condition: is_ram_condition,
        target_if_true: is_ram_block,
        target_if_true_argument: None,
        target_if_false: Rc::clone(&is_not_ram_block),
        target_if_false_argument: None,
    };

    // read PRG RAM
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
    let prg_ram_value = is_prg_ram_block
        .borrow_mut()
        .define_8(Definition8::PrgRam(address));
    is_prg_ram_block.borrow_mut().jump = Jump::BasicBlock {
        condition: is_prg_ram_condition,
        target_if_true: Rc::clone(&last_block),
        target_if_true_argument: Some(prg_ram_value),
        target_if_false: Rc::clone(&last_block),
        target_if_false_argument: Some(prg_ram_value),
    };
    let is_not_prg_ram_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(&variable_counter))));
    is_not_ram_block.borrow_mut().jump = Jump::BasicBlock {
        condition: is_prg_ram_condition,
        target_if_true: is_prg_ram_block,
        target_if_true_argument: None,
        target_if_false: Rc::clone(&is_not_prg_ram_block),
        target_if_false_argument: None,
    };

    // read ROM
    let is_rom_condition = is_not_prg_ram_block
        .borrow_mut()
        .define_1(Definition1::LessThan16(n0x7fff, address));
    let is_rom_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(&variable_counter))));
    let rom_value = is_rom_block
        .borrow_mut()
        .define_8(Definition8::Rom(address));
    is_rom_block.borrow_mut().jump = Jump::BasicBlock {
        condition: r#true,
        target_if_true: Rc::clone(&last_block),
        target_if_true_argument: Some(rom_value),
        target_if_false: Rc::clone(&last_block),
        target_if_false_argument: Some(rom_value),
    };
    let fallback_value = is_not_prg_ram_block.borrow_mut().define_8(0.into());
    is_not_prg_ram_block.borrow_mut().jump = Jump::BasicBlock {
        condition: is_rom_condition,
        target_if_true: is_rom_block,
        target_if_true_argument: None,
        target_if_false: Rc::clone(&last_block),
        target_if_false_argument: Some(fallback_value),
    };

    let value = last_block
        .borrow_mut()
        .define_8(Definition8::BasicBlockArgument);
    *basic_block = last_block;
    value
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
        target_if_true_argument: None,
        target_if_false: Rc::clone(&last_block),
        target_if_false_argument: None,
    };
    let is_not_ram_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(&variable_counter))));
    basic_block.borrow_mut().jump = Jump::BasicBlock {
        condition: is_ram_condition,
        target_if_true: Rc::clone(&is_ram_block),
        target_if_true_argument: None,
        target_if_false: Rc::clone(&is_not_ram_block),
        target_if_false_argument: None,
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
        target_if_true_argument: None,
        target_if_false: Rc::clone(&last_block),
        target_if_false_argument: None,
    };
    is_not_ram_block.borrow_mut().jump = Jump::BasicBlock {
        condition: is_prg_ram_condition,
        target_if_true: Rc::clone(&is_prg_ram_block),
        target_if_true_argument: None,
        target_if_false: Rc::clone(&last_block),
        target_if_false_argument: None,
    };

    *basic_block = last_block;
}
