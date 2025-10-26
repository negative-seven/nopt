use std::ops::RangeInclusive;

use crate::compiler::frontend::r#abstract::ppu;

pub(super) fn read<Visitor: super::Visitor>(
    visitor: &mut Visitor,
    address: Visitor::U16,
) -> Visitor::U8 {
    let if_address_in_range = |visitor: &mut Visitor,
                               address_range: RangeInclusive<u16>,
                               visit_true_block: fn(Visitor, Visitor::U16),
                               false_value: Visitor::U8|
     -> Visitor::U8 {
        let condition = {
            let lower_bound_condition = {
                let start = visitor.immediate_u16(*address_range.start());
                visitor.less_than_or_equal(start, address)
            };
            let upper_bound_condition = {
                let end = visitor.immediate_u16(*address_range.end());
                visitor.less_than_or_equal(address, end)
            };
            visitor.and_u1(lower_bound_condition, upper_bound_condition)
        };

        visitor.if_else_with_result(
            condition,
            |visitor| visit_true_block(visitor, address),
            |visitor| {
                visitor.terminate(Some(false_value));
            },
        )
    };

    let value = visitor.immediate_u8(0);
    let value = if_address_in_range(
        visitor,
        0x0..=0x7ff,
        |mut visitor, address| {
            let value = visitor.cpu_ram(address);
            visitor.terminate(Some(value));
        },
        value,
    );
    let value = if_address_in_range(
        visitor,
        0x2007..=0x2007,
        |mut visitor, _| {
            let value = ppu::read_ppudata(&mut visitor);
            visitor.terminate(Some(value));
        },
        value,
    );
    let value = if_address_in_range(
        visitor,
        0x6000..=0x7fff,
        |mut visitor, address| {
            let value = visitor.prg_ram(address);
            visitor.terminate(Some(value));
        },
        value,
    );
    if_address_in_range(
        visitor,
        0x8000..=0xffff,
        |mut visitor, address| {
            let value = visitor.rom(address);
            visitor.terminate(Some(value));
        },
        value,
    )
}

pub(super) fn write<Visitor: super::Visitor>(
    visitor: &mut Visitor,
    address: Visitor::U16,
    value: Visitor::U8,
) {
    let mut if_address_in_range =
        |range: RangeInclusive<u16>, visit_true_block: fn(Visitor, Visitor::U16, Visitor::U8)| {
            let condition = {
                let lower_bound_condition = {
                    let start = visitor.immediate_u16(*range.start());
                    visitor.less_than_or_equal(start, address)
                };
                let upper_bound_condition = {
                    let end = visitor.immediate_u16(*range.end());
                    visitor.less_than_or_equal(address, end)
                };
                visitor.and_u1(lower_bound_condition, upper_bound_condition)
            };

            visitor.r#if(condition, |visitor| {
                visit_true_block(visitor, address, value);
            });
        };

    if_address_in_range(0x0..=0x7ff, |mut visitor, address, value| {
        visitor.set_cpu_ram(address, value);
        visitor.terminate(None);
    });
    if_address_in_range(0x2006..=0x2006, |mut visitor, _, value| {
        ppu::write_ppuaddr(&mut visitor, value);
        visitor.terminate(None);
    });
    if_address_in_range(0x2007..=0x2007, |mut visitor, _, value| {
        ppu::write_ppudata(&mut visitor, value);
        visitor.terminate(None);
    });
    if_address_in_range(0x6000..=0x7fff, |mut visitor, address, value| {
        visitor.set_prg_ram(address, value);
        visitor.terminate(None);
    });
}
