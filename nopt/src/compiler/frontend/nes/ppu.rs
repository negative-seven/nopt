use crate::compiler::frontend::nes::Nes;
use std::ops::RangeInclusive;

pub struct Ppu {
    pub ram: [u8; 0x800],
    pub palette_ram: [u8; 0x20],
    pub control_register: u8,
    pub read_buffer: u8,
    pub current_address: u16,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            ram: [0; 0x800],
            palette_ram: [0; 0x20],
            control_register: 0,
            read_buffer: 0,
            current_address: 0,
        }
    }

    pub(super) fn write_ppuaddr<Visitor: super::Visitor>(
        &mut self,
        visitor: &mut Visitor,
        value: Visitor::U8,
    ) {
        let old_address = visitor.memory_u16(&raw const self.current_address);
        let new_address_high = visitor.low_byte(old_address);
        let new_address_low = value;
        let new_address = visitor.concatenate(new_address_high, new_address_low);
        visitor.set_memory_u16(&raw mut self.current_address, new_address);
    }

    pub(super) fn read_ppudata<Cartridge: crate::cartridge::Cartridge, Visitor: super::Visitor>(
        nes: &mut Nes<Cartridge>,
        visitor: &mut Visitor,
    ) -> Visitor::U8 {
        let address = visitor.memory_u16(&raw const nes.ppu.current_address);
        nes.ppu.increment_ppu_current_address(visitor);
        Self::read(nes, visitor, address)
    }

    pub(super) fn write_ppudata<Cartridge: crate::cartridge::Cartridge, Visitor: super::Visitor>(
        nes: &mut Nes<Cartridge>,
        visitor: &mut Visitor,
        value: Visitor::U8,
    ) {
        let address = visitor.memory_u16(&raw const nes.ppu.current_address);
        nes.ppu.increment_ppu_current_address(visitor);
        Self::write(nes, visitor, address, value);
    }

    fn increment_ppu_current_address<Visitor: super::Visitor>(&mut self, visitor: &mut Visitor) {
        let n0 = visitor.immediate_u8(0);

        let address = visitor.memory_u16(&raw const self.current_address);
        let address_increment = {
            let control_register = visitor.memory_u8(&raw const self.control_register);
            let control_register_increment_bit = visitor.get_bit(control_register, 2);
            let increment = visitor.if_else_with_result(
                control_register_increment_bit,
                |mut visitor| {
                    let n32 = visitor.immediate_u8(32);
                    visitor.terminate(Some(n32));
                },
                |mut visitor| {
                    let n1 = visitor.immediate_u8(1);
                    visitor.terminate(Some(n1));
                },
            );
            visitor.concatenate(n0, increment)
        };
        let incremented_address = visitor.add_u16(address, address_increment);
        visitor.set_memory_u16(&raw mut self.current_address, incremented_address);
    }

    fn read<Cartridge: crate::cartridge::Cartridge, Visitor: super::Visitor>(
        nes: &mut Nes<Cartridge>,
        visitor: &mut Visitor,
        address: Visitor::U16,
    ) -> Visitor::U8 {
        let mut if_address_in_range =
            |visitor: &mut Visitor,
             address_range: RangeInclusive<u16>,
             visit_true_block: fn(&mut Nes<Cartridge>, Visitor, Visitor::U16),
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
            0x2000..=0x3eff,
            |nes, mut visitor, address| {
                let previous_value = visitor.memory_u8(&raw const nes.ppu.read_buffer);
                let address = {
                    let is_mirroring_horizontal =
                        nes.cartridge.read_is_mirroring_horizontal(&mut visitor);

                    // TODO: simplify this logic once if_else_with_result can return a U16
                    let address_low = visitor.low_byte(address);
                    let address_high = visitor.if_else_with_result(
                        is_mirroring_horizontal,
                        |mut visitor| {
                            let address_high = visitor.high_byte(address);
                            let address_high_low = {
                                let mask = visitor.immediate_u8(0b0000_0111);
                                visitor.and_u8(address_high, mask)
                            };
                            let address_high_high = {
                                let mask = visitor.immediate_u8(0b0001_0000);
                                let address = visitor.and_u8(address_high, mask);
                                visitor.shift_right(address)
                            };
                            let address_high = visitor.or(address_high_high, address_high_low);
                            visitor.terminate(Some(address_high));
                        },
                        |mut visitor| {
                            let address_high = visitor.high_byte(address);
                            visitor.terminate(Some(address_high));
                        },
                    );
                    let address = visitor.concatenate(address_high, address_low);
                    let mask = visitor.immediate_u16(0x7ff);
                    visitor.and_u16(address, mask)
                };
                let value = visitor.memory_with_offset_u8(nes.ppu.ram.as_ptr(), address);
                visitor.set_memory_u8(&raw mut nes.ppu.read_buffer, value);
                visitor.terminate(Some(previous_value));
            },
            value,
        );
        if_address_in_range(
            visitor,
            0x3f00..=0x3fff,
            |nes, mut visitor, address| {
                let address_mask = visitor.immediate_u16(0x1f);
                let address = visitor.and_u16(address, address_mask);
                let value = visitor.memory_with_offset_u8(nes.ppu.palette_ram.as_ptr(), address);
                visitor.terminate(Some(value));
            },
            value,
        )
    }

    fn write<Cartridge: crate::cartridge::Cartridge, Visitor: super::Visitor>(
        nes: &mut Nes<Cartridge>,
        visitor: &mut Visitor,
        address: Visitor::U16,
        value: Visitor::U8,
    ) {
        let mut if_address_in_range = |range: RangeInclusive<u16>,
                                       visit_true_block: fn(
            &mut Nes<Cartridge>,
            Visitor,
            Visitor::U16,
            Visitor::U8,
        )| {
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

        if_address_in_range(0x2000..=0x3eff, |nes, mut visitor, address, value| {
            let address = {
                let is_mirroring_horizontal =
                    nes.cartridge.read_is_mirroring_horizontal(&mut visitor);

                // TODO: simplify this logic once if_else_with_result can return a U16
                let address_low = visitor.low_byte(address);
                let address_high = visitor.if_else_with_result(
                    is_mirroring_horizontal,
                    |mut visitor| {
                        let address_high = visitor.high_byte(address);
                        let address_high_low = {
                            let mask = visitor.immediate_u8(0b111);
                            visitor.and_u8(address_high, mask)
                        };
                        let address_high_high = {
                            let mask = visitor.immediate_u8(0b0001_0000);
                            let address = visitor.and_u8(address_high, mask);
                            visitor.shift_right(address)
                        };
                        let address_high = visitor.or(address_high_high, address_high_low);
                        visitor.terminate(Some(address_high));
                    },
                    |mut visitor| {
                        let address_high = visitor.high_byte(address);
                        visitor.terminate(Some(address_high));
                    },
                );
                let address = visitor.concatenate(address_high, address_low);

                let mask = visitor.immediate_u16(0x7ff);
                visitor.and_u16(address, mask)
            };
            visitor.set_memory_with_offset_u8(nes.ppu.ram.as_mut_ptr(), address, value);
            visitor.terminate(None);
        });
        if_address_in_range(0x3f00..=0x3fff, |nes, mut visitor, address, value| {
            let address_mask = visitor.immediate_u16(0x1f);
            let address = visitor.and_u16(address, address_mask);
            visitor.set_memory_with_offset_u8(nes.ppu.palette_ram.as_mut_ptr(), address, value);
            visitor.terminate(None);
        });
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}
