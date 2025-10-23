use crate::compiler::{
    frontend::r#abstract::CompilerVisitor,
    ir::{
        BasicBlock, Definition1, Definition8, Definition16, Destination8, Destination16,
        Instruction, Variable8, Variable16,
    },
};
use std::ops::RangeInclusive;

pub(super) fn read(visitor: &mut CompilerVisitor, address: Variable16) -> Variable8 {
    let if_address_in_range =
        |visitor: &mut CompilerVisitor,
         address_range: RangeInclusive<u16>,
         populate_true_block: fn(&mut BasicBlock, Variable16) -> Variable8,
         false_value: Variable8|
         -> Variable8 {
            let condition = {
                let lower_bound_condition = {
                    let start = visitor.define_16(*address_range.start());
                    visitor.define_1(Definition1::LessThanOrEqual16(start, address))
                };
                let upper_bound_condition = {
                    let end = visitor.define_16(*address_range.end());
                    visitor.define_1(Definition1::LessThanOrEqual16(address, end))
                };
                visitor.define_1(lower_bound_condition & upper_bound_condition)
            };

            visitor.if_else_with_result(
                condition,
                |block| populate_true_block(block, address),
                |_| false_value,
            )
        };

    let value = visitor.define_8(0);
    let value = if_address_in_range(
        visitor,
        0x0..=0x7ff,
        |block, address| block.define_8(Definition8::CpuRam(address)),
        value,
    );
    let value = if_address_in_range(
        visitor,
        0x2007..=0x2007,
        |block, _| {
            let address = block.define_16(Definition16::PpuCurrentAddress);
            block.define_8(Definition8::PpuRam(address))
        },
        value,
    );
    let value = if_address_in_range(
        visitor,
        0x6000..=0x7fff,
        |block, address| block.define_8(Definition8::PrgRam(address)),
        value,
    );
    if_address_in_range(
        visitor,
        0x8000..=0xffff,
        |block, address| block.define_8(Definition8::Rom(address)),
        value,
    )
}

pub(super) fn write(visitor: &mut CompilerVisitor, address: Variable16, value: Variable8) {
    let mut if_address_in_range =
        |range: RangeInclusive<u16>,
         populate_write_block: fn(&mut BasicBlock, Variable16, Variable8)| {
            let condition = {
                let lower_bound_condition = {
                    let start = visitor.define_16(*range.start());
                    visitor.define_1(Definition1::LessThanOrEqual16(start, address))
                };
                let upper_bound_condition = {
                    let end = visitor.define_16(*range.end());
                    visitor.define_1(Definition1::LessThanOrEqual16(address, end))
                };
                visitor.define_1(lower_bound_condition & upper_bound_condition)
            };

            visitor.if_else(
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
