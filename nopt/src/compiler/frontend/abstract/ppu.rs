pub(super) fn write_ppuaddr<Visitor: super::Visitor>(visitor: &mut Visitor, value: Visitor::U8) {
    let old_address = visitor.ppu_current_address();
    let new_address_high = visitor.low_byte(old_address);
    let new_address_low = value;
    let new_address = visitor.concatenate(new_address_high, new_address_low);
    visitor.set_ppu_current_address(new_address);
}

pub(super) fn read_ppudata<Visitor: super::Visitor>(visitor: &mut Visitor) -> Visitor::U8 {
    let address = visitor.ppu_current_address();
    visitor.ppu_ram(address)
}

pub(super) fn write_ppudata<Visitor: super::Visitor>(visitor: &mut Visitor, value: Visitor::U8) {
    let address = visitor.ppu_current_address();
    visitor.set_ppu_ram(address, value);
}
