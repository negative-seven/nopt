use crate::compiler::ir::{
    BasicBlock, Definition1, Definition8, Definition16, Destination8, Destination16, Instruction,
    Jump, Variable1, Variable8, Variable16,
};
use std::{cell::RefCell, ops::RangeInclusive, rc::Rc};

pub(super) fn compile_read(
    current_block: &mut Rc<RefCell<BasicBlock>>,
    address: Variable16,
) -> Variable8 {
    let if_address_in_range =
        |current_block: &mut Rc<RefCell<BasicBlock>>,
         address_range: RangeInclusive<u16>,
         populate_true_block: fn(&mut BasicBlock, Variable16) -> Variable8,
         false_value: Variable8|
         -> Variable8 {
            let condition = {
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

            if_else_with_result(
                current_block,
                condition,
                |block| populate_true_block(block, address),
                |_| false_value,
            )
        };

    let value = current_block.borrow_mut().define_8(0.into());
    let value = if_address_in_range(
        current_block,
        0x0..=0x7ff,
        |block, address| block.define_8(Definition8::CpuRam(address)),
        value,
    );
    let value = if_address_in_range(
        current_block,
        0x2007..=0x2007,
        |block, _| {
            let address = block.define_16(Definition16::PpuCurrentAddress);
            block.define_8(Definition8::PpuRam(address))
        },
        value,
    );
    let value = if_address_in_range(
        current_block,
        0x6000..=0x7fff,
        |block, address| block.define_8(Definition8::PrgRam(address)),
        value,
    );
    if_address_in_range(
        current_block,
        0x8000..=0xffff,
        |block, address| block.define_8(Definition8::Rom(address)),
        value,
    )
}

pub(super) fn compile_write(
    current_block: &mut Rc<RefCell<BasicBlock>>,
    address: Variable16,
    value: Variable8,
) {
    let mut if_address_in_range =
        |range: RangeInclusive<u16>,
         populate_write_block: fn(&mut BasicBlock, Variable16, Variable8)| {
            let condition = {
                let lower_bound_condition = {
                    let start = current_block
                        .borrow_mut()
                        .define_16((*range.start()).into());
                    current_block
                        .borrow_mut()
                        .define_1(Definition1::LessThanOrEqual16(start, address))
                };
                let upper_bound_condition = {
                    let end = current_block.borrow_mut().define_16((*range.end()).into());
                    current_block
                        .borrow_mut()
                        .define_1(Definition1::LessThanOrEqual16(address, end))
                };
                current_block
                    .borrow_mut()
                    .define_1(lower_bound_condition & upper_bound_condition)
            };

            if_else(
                current_block,
                condition,
                |block| populate_write_block(block, address, value),
                |_| {},
            );
        };

    if_address_in_range(0x0..=0x7ff, |block, address, value| {
        block.instructions.push(Instruction::Store8 {
            destination: Destination8::CpuRam(address),
            variable: value,
        });
    });
    if_address_in_range(0x2006..=0x2006, |block, _, value| {
        let old_address = block.define_16(Definition16::PpuCurrentAddress);
        let new_address_high = block.define_8(Definition8::LowByte(old_address));
        let new_address_low = value;
        let new_address = block.define_16(new_address_high % new_address_low);
        block.instructions.push(Instruction::Store16 {
            destination: Destination16::PpuCurrentAddress,
            variable: new_address,
        });
    });
    if_address_in_range(0x2007..=0x2007, |block, _, value| {
        let address = block.define_16(Definition16::PpuCurrentAddress);
        block.instructions.push(Instruction::Store8 {
            destination: Destination8::PpuRam(address),
            variable: value,
        });
    });
    if_address_in_range(0x6000..=0x7fff, |block, address, value| {
        block.instructions.push(Instruction::Store8 {
            destination: Destination8::PrgRam(address),
            variable: value,
        });
    });
}

fn if_else(
    current_block: &mut Rc<RefCell<BasicBlock>>,
    condition: Variable1,
    populate_true_block: impl Fn(&mut BasicBlock),
    populate_false_block: impl Fn(&mut BasicBlock),
) {
    let unused_variable = current_block.borrow_mut().define_8(0.into());

    if_else_with_result(
        current_block,
        condition,
        |block| {
            populate_true_block(block);
            unused_variable
        },
        |block| {
            populate_false_block(block);
            unused_variable
        },
    );
}

fn if_else_with_result(
    current_block: &mut Rc<RefCell<BasicBlock>>,
    condition: Variable1,
    populate_true_block: impl Fn(&mut BasicBlock) -> Variable8,
    populate_false_block: impl Fn(&mut BasicBlock) -> Variable8,
) -> Variable8 {
    let r#true = current_block.borrow_mut().define_1(true.into());

    let variable_id_counter = Rc::clone(&current_block.borrow().variable_id_counter);

    let exit_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
        &variable_id_counter,
    ))));
    exit_block.borrow_mut().set_has_argument(true);

    let true_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
        &variable_id_counter,
    ))));
    let true_value = populate_true_block(&mut true_block.borrow_mut());
    true_block.borrow_mut().jump = Jump::BasicBlock {
        condition: r#true,
        target_if_true: Rc::clone(&exit_block),
        target_if_true_argument: Some(true_value),
        target_if_false: Rc::clone(&exit_block),
        target_if_false_argument: Some(true_value),
    };

    let false_block = Rc::new(RefCell::new(BasicBlock::new(Rc::clone(
        &variable_id_counter,
    ))));
    let false_value = populate_false_block(&mut false_block.borrow_mut());
    false_block.borrow_mut().jump = Jump::BasicBlock {
        condition: r#true,
        target_if_true: Rc::clone(&exit_block),
        target_if_true_argument: Some(false_value),
        target_if_false: Rc::clone(&exit_block),
        target_if_false_argument: Some(false_value),
    };

    current_block.borrow_mut().jump = Jump::BasicBlock {
        condition,
        target_if_true: Rc::clone(&true_block),
        target_if_true_argument: None,
        target_if_false: Rc::clone(&false_block),
        target_if_false_argument: None,
    };

    let result = exit_block
        .borrow_mut()
        .define_8(Definition8::BasicBlockArgument);
    *current_block = exit_block;
    result
}
