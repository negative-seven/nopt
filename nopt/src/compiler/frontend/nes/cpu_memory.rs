use crate::compiler::frontend::nes::{Nes, ppu};
use std::ops::RangeInclusive;

pub(super) fn read<Visitor: super::Visitor>(
    nes: &mut Nes,
    visitor: &mut Visitor,
    address: Visitor::U16,
) -> Visitor::U8 {
    let mut if_address_in_range = |visitor: &mut Visitor,
                                   address_range: RangeInclusive<u16>,
                                   visit_true_block: fn(&mut Nes, Visitor, Visitor::U16),
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
            |visitor| visit_true_block(nes, visitor, address),
            |visitor| {
                visitor.terminate(Some(false_value));
            },
        )
    };

    let value = visitor.immediate_u8(0);
    let value = if_address_in_range(
        visitor,
        0x0..=0x7ff,
        |nes, mut visitor, address| {
            let value = visitor.memory_with_offset_u8(nes.cpu.ram.as_ptr(), address);
            visitor.terminate(Some(value));
        },
        value,
    );
    let value = if_address_in_range(
        visitor,
        0x2007..=0x2007,
        |nes, mut visitor, _| {
            let value = ppu::read_ppudata(&mut nes.ppu, &mut visitor);
            visitor.terminate(Some(value));
        },
        value,
    );
    let value = if_address_in_range(
        visitor,
        0x6000..=0x7fff,
        |nes, mut visitor, address| {
            let address_mask = visitor.immediate_u16(0x1fff);
            let address = visitor.and_u16(address, address_mask);
            let value = visitor.memory_with_offset_u8(nes.prg_ram.as_ptr(), address);
            visitor.terminate(Some(value));
        },
        value,
    );
    if_address_in_range(
        visitor,
        0x8000..=0xffff,
        |nes, mut visitor, address| {
            let address_mask = visitor.immediate_u16(0x7fff);
            let address = visitor.and_u16(address, address_mask);
            let value = visitor.memory_with_offset_u8(nes.rom.prg_rom().as_ptr(), address);
            visitor.terminate(Some(value));
        },
        value,
    )
}

pub(super) fn write<Visitor: super::Visitor>(
    nes: &mut Nes,
    visitor: &mut Visitor,
    address: Visitor::U16,
    value: Visitor::U8,
) {
    let mut if_address_in_range =
        |range: RangeInclusive<u16>,
         visit_true_block: fn(&mut Nes, Visitor, Visitor::U16, Visitor::U8)| {
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
                visit_true_block(nes, visitor, address, value);
            });
        };

    if_address_in_range(0x0..=0x7ff, |nes, mut visitor, address, value| {
        visitor.set_memory_with_offset_u8(nes.cpu.ram.as_mut_ptr(), address, value);
        visitor.terminate(None);
    });
    if_address_in_range(0x2000..=0x2000, |nes, mut visitor, _, value| {
        visitor.set_memory_u8(&raw mut nes.ppu.control_register, value);
        visitor.terminate(None);
    });
    if_address_in_range(0x2006..=0x2006, |nes, mut visitor, _, value| {
        ppu::write_ppuaddr(&mut nes.ppu, &mut visitor, value);
        visitor.terminate(None);
    });
    if_address_in_range(0x2007..=0x2007, |nes, mut visitor, _, value| {
        ppu::write_ppudata(&mut nes.ppu, &mut visitor, value);
        visitor.terminate(None);
    });
    if_address_in_range(0x6000..=0x7fff, |nes, mut visitor, address, value| {
        let address_mask = visitor.immediate_u16(0x1fff);
        let address = visitor.and_u16(address, address_mask);
        visitor.set_memory_with_offset_u8(nes.prg_ram.as_mut_ptr(), address, value);
        visitor.terminate(None);
    });
}
