use crate::compiler::ir::{
    BasicBlock, Definition1, Definition8, Destination8, Instruction, Jump, Variable8, Variable16,
};
use std::{cell::RefCell, ops::RangeInclusive, rc::Rc};

pub(super) fn compile_read(
    basic_block: &mut Rc<RefCell<BasicBlock>>,
    address: Variable16,
) -> Variable8 {
    let r#true = basic_block.borrow_mut().define_1(true.into());
    let n0 = basic_block.borrow_mut().define_8(0.into());

    let variable_counter = Rc::clone(&basic_block.borrow().variable_id_counter);
    let exit_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(&variable_counter))));
    exit_block.borrow_mut().set_has_argument(true);
    exit_block.borrow_mut().jump = basic_block.borrow().jump.clone();

    let mut current_block = Rc::clone(basic_block);

    let mut compile_read_in_range =
        |address_range: RangeInclusive<u16>,
         read_block_instruction_provider: fn(&mut BasicBlock, Variable16) -> Variable8| {
        let variable_counter = Rc::clone(&current_block.borrow().variable_id_counter);

        let not_read_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
            &current_block.borrow().variable_id_counter,
        ))));

        let read_condition = {
            let lower_bound_condition = {
                let start = current_block
                    .borrow_mut()
                    .define_16((*address_range.start()).into());
                current_block
                    .borrow_mut()
                    .define_1(Definition1::LessThanOrEqual16(start, address))
            };
            let upper_bound_condition = {
                let end = current_block
                    .borrow_mut()
                    .define_16((*address_range.end()).into());
                current_block
                    .borrow_mut()
                    .define_1(Definition1::LessThanOrEqual16(address, end))
            };
            current_block
                .borrow_mut()
                .define_1(lower_bound_condition & upper_bound_condition)
        };
        let read_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(&variable_counter))));
        let read_value = read_block_instruction_provider(&mut read_block.borrow_mut(), address);
        read_block.borrow_mut().jump = Jump::BasicBlock {
            condition: r#true,
            target_if_true: Rc::clone(&exit_block),
            target_if_true_argument: Some(read_value),
            target_if_false: Rc::clone(&exit_block),
            target_if_false_argument: Some(read_value),
        };
        current_block.borrow_mut().jump = Jump::BasicBlock {
            condition: read_condition,
            target_if_true: read_block,
            target_if_true_argument: None,
            target_if_false: Rc::clone(&not_read_block),
            target_if_false_argument: None,
        };

        current_block = not_read_block;
    };

    compile_read_in_range(0x0..=0x7ff, |block, address| {
        block.define_8(Definition8::Ram(address))
    });
    compile_read_in_range(0x6000..=0x7fff, |block, address| {
        block.define_8(Definition8::PrgRam(address))
    });
    compile_read_in_range(0x8000..=0xffff, |block, address| {
        block.define_8(Definition8::Rom(address))
    });

    // Default to a value of 0.
    current_block.borrow_mut().jump = Jump::BasicBlock {
        condition: r#true,
        target_if_true: Rc::clone(&exit_block),
        target_if_true_argument: Some(n0),
        target_if_false: Rc::clone(&exit_block),
        target_if_false_argument: Some(n0),
    };

    let value = exit_block
        .borrow_mut()
        .define_8(Definition8::BasicBlockArgument);
    *basic_block = exit_block;
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
