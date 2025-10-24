use crate::compiler::{
    frontend::r#abstract::CompilerVisitor,
    ir::{
        Definition1, Definition8, Definition16, Destination8, Destination16, Variable8, Variable16,
    },
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
                |visitor| visit_true_block(visitor, address),
                |_| false_value,
            )
        };

    let value = visitor.define_8(0);
    let value = if_address_in_range(
        visitor,
        0x0..=0x7ff,
        |visitor, address| visitor.define_8(Definition8::CpuRam(address)),
        value,
    );
    let value = if_address_in_range(
        visitor,
        0x2007..=0x2007,
        |visitor, _| {
            let address = visitor.define_16(Definition16::PpuCurrentAddress);
            visitor.define_8(Definition8::PpuRam(address))
        },
        value,
    );
    let value = if_address_in_range(
        visitor,
        0x6000..=0x7fff,
        |visitor, address| visitor.define_8(Definition8::PrgRam(address)),
        value,
    );
    if_address_in_range(
        visitor,
        0x8000..=0xffff,
        |visitor, address| visitor.define_8(Definition8::Rom(address)),
        value,
    )
}

pub(super) fn write(visitor: &mut CompilerVisitor, address: Variable16, value: Variable8) {
    let mut if_address_in_range =
        |range: RangeInclusive<u16>,
         visit_true_block: fn(&mut CompilerVisitor, Variable16, Variable8)| {
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
                |visitor| visit_true_block(visitor, address, value),
                |_| {},
            );
        };

    if_address_in_range(0x0..=0x7ff, |visitor, address, value| {
        visitor.store_8(Destination8::CpuRam(address), value);
    });
    if_address_in_range(0x2006..=0x2006, |visitor, _, value| {
        let old_address = visitor.define_16(Definition16::PpuCurrentAddress);
        let new_address_high = visitor.define_8(Definition8::LowByte(old_address));
        let new_address_low = value;
        let new_address = visitor.define_16(new_address_high % new_address_low);
        visitor.store_16(Destination16::PpuCurrentAddress, new_address);
    });
    if_address_in_range(0x2007..=0x2007, |visitor, _, value| {
        let address = visitor.define_16(Definition16::PpuCurrentAddress);
        visitor.store_8(Destination8::PpuRam(address), value);
    });
    if_address_in_range(0x6000..=0x7fff, |visitor, address, value| {
        visitor.store_8(Destination8::PrgRam(address), value);
    });
}
