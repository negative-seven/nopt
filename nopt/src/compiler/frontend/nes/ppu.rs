use std::ops::RangeInclusive;

pub struct Ppu {
    pub ram: [u8; 0x1000],
    pub palette_ram: [u8; 0x20],
    pub control_register: u8,
    pub read_buffer: u8,
    pub current_address: u16,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            ram: [0; 0x1000],
            palette_ram: [0; 0x20],
            control_register: 0,
            read_buffer: 0,
            current_address: 0,
        }
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) fn write_ppuaddr<Visitor: super::Visitor>(visitor: &mut Visitor, value: Visitor::U8) {
    let old_address = visitor.ppu_current_address();
    let new_address_high = visitor.low_byte(old_address);
    let new_address_low = value;
    let new_address = visitor.concatenate(new_address_high, new_address_low);
    visitor.set_ppu_current_address(new_address);
}

pub(super) fn read_ppudata<Visitor: super::Visitor>(visitor: &mut Visitor) -> Visitor::U8 {
    let address = visitor.ppu_current_address();
    increment_ppu_current_address(visitor);
    read(visitor, address)
}

pub(super) fn write_ppudata<Visitor: super::Visitor>(visitor: &mut Visitor, value: Visitor::U8) {
    let address = visitor.ppu_current_address();
    increment_ppu_current_address(visitor);
    write(visitor, address, value);
}

fn increment_ppu_current_address<Visitor: super::Visitor>(visitor: &mut Visitor) {
    let n0 = visitor.immediate_u8(0);

    let address = visitor.ppu_current_address();
    let address_increment = {
        let control_register = visitor.ppu_control_register();
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
    visitor.set_ppu_current_address(incremented_address);
}

fn read<Visitor: super::Visitor>(visitor: &mut Visitor, address: Visitor::U16) -> Visitor::U8 {
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
        0x2000..=0x3eff,
        |mut visitor, address| {
            let previous_value = visitor.ppu_read_buffer();
            let address_mask = visitor.immediate_u16(0xfff);
            let address = visitor.and_u16(address, address_mask);
            let value = visitor.ppu_ram(address);
            visitor.set_ppu_read_buffer(value);
            visitor.terminate(Some(previous_value));
        },
        value,
    );
    if_address_in_range(
        visitor,
        0x3f00..=0x3fff,
        |mut visitor, address| {
            let address_mask = visitor.immediate_u16(0x1f);
            let address = visitor.and_u16(address, address_mask);
            let value = visitor.ppu_palette_ram(address);
            visitor.terminate(Some(value));
        },
        value,
    )
}

fn write<Visitor: super::Visitor>(
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

    if_address_in_range(0x2000..=0x3eff, |mut visitor, address, value| {
        let address_mask = visitor.immediate_u16(0xfff);
        let address = visitor.and_u16(address, address_mask);
        visitor.set_ppu_ram(address, value);
        visitor.terminate(None);
    });
    if_address_in_range(0x3f00..=0x3fff, |mut visitor, address, value| {
        let address_mask = visitor.immediate_u16(0x1f);
        let address = visitor.and_u16(address, address_mask);
        visitor.set_ppu_palette_ram(address, value);
        visitor.terminate(None);
    });
}
