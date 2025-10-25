use crate::compiler::{
    frontend::r#abstract::CompilerVisitor,
    ir::{Variable8, Variable16},
};
use std::ops::RangeInclusive;

pub(super) fn read(visitor: &mut CompilerVisitor, address: Variable16) -> Variable8 {
    let if_address_in_range =
        |visitor: &mut CompilerVisitor,
         address_range: RangeInclusive<u16>,
         visit_true_block: fn(&mut CompilerVisitor, Variable16) -> Variable8,
         false_value: Variable8|
         -> Variable8 {
            let condition = {
                let lower_bound_condition = {
                    let start = visitor.immediate_u16(*address_range.start());
                    visitor.less_than_or_equal(start, address)
                };
                let upper_bound_condition = {
                    let end = visitor.immediate_u16(*address_range.end());
                    visitor.less_than_or_equal(address, end)
                };
                visitor.define_1(lower_bound_condition & upper_bound_condition)
            };

            visitor.if_else_with_result(
                condition,
                |visitor| visit_true_block(visitor, address),
                |_| false_value,
            )
        };

    let value = visitor.immediate_u8(0);
    let value = if_address_in_range(visitor, 0x0..=0x7ff, CompilerVisitor::cpu_ram, value);
    let value = if_address_in_range(
        visitor,
        0x2007..=0x2007,
        |visitor, _| {
            let address = visitor.ppu_current_address();
            visitor.ppu_ram(address)
        },
        value,
    );
    let value = if_address_in_range(visitor, 0x6000..=0x7fff, CompilerVisitor::prg_ram, value);
    if_address_in_range(visitor, 0x8000..=0xffff, CompilerVisitor::rom, value)
}

pub(super) fn write(visitor: &mut CompilerVisitor, address: Variable16, value: Variable8) {
    let mut if_address_in_range =
        |range: RangeInclusive<u16>,
         visit_true_block: fn(&mut CompilerVisitor, Variable16, Variable8)| {
            let condition = {
                let lower_bound_condition = {
                    let start = visitor.immediate_u16(*range.start());
                    visitor.less_than_or_equal(start, address)
                };
                let upper_bound_condition = {
                    let end = visitor.immediate_u16(*range.end());
                    visitor.less_than_or_equal(address, end)
                };
                visitor.define_1(lower_bound_condition & upper_bound_condition)
            };

            visitor.if_else(
                condition,
                |visitor| visit_true_block(visitor, address, value),
                |_| {},
            );
        };

    if_address_in_range(0x0..=0x7ff, |visitor, address, value| {
        visitor.set_cpu_ram(address, value);
    });
    if_address_in_range(0x2006..=0x2006, |visitor, _, value| {
        let old_address = visitor.ppu_current_address();
        let new_address_high = visitor.low_byte(old_address);
        let new_address_low = value;
        let new_address = visitor.concatenate(new_address_high, new_address_low);
        visitor.set_ppu_current_address(new_address);
    });
    if_address_in_range(0x2007..=0x2007, |visitor, _, value| {
        let address = visitor.ppu_current_address();
        visitor.set_ppu_ram(address, value);
    });
    if_address_in_range(0x6000..=0x7fff, |visitor, address, value| {
        visitor.set_prg_ram(address, value);
    });
}
