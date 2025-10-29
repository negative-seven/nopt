pub(crate) trait Visitor: Sized {
    type U1: Copy;
    type U8: Copy;
    type U16: Copy;

    fn immediate_u1(&mut self, value: bool) -> Self::U1 {
        let zero_if_true = self.immediate_u8(u8::from(!value));
        self.is_zero(zero_if_true)
    }

    fn immediate_u8(&mut self, value: u8) -> Self::U8;

    fn immediate_u16(&mut self, value: u16) -> Self::U16 {
        let low = self.immediate_u8((value & 0xff).try_into().unwrap());
        let high = self.immediate_u8((value >> 8).try_into().unwrap());
        self.concatenate(high, low)
    }

    fn memory_u8(&mut self, address: *const u8) -> Self::U8 {
        let n0 = self.immediate_u16(0);
        self.memory_with_offset_u8(address, n0)
    }

    fn memory_with_offset_u8(&mut self, address: *const u8, offset: Self::U16) -> Self::U8;

    fn memory_u16(&mut self, address: *const u16) -> Self::U16 {
        let low = self.memory_u8(address.cast());
        let high = self.memory_u8(unsafe { address.byte_add(1).cast() });
        self.concatenate(high, low)
    }

    fn set_memory_u8(&mut self, address: *mut u8, value: Self::U8) {
        let n0 = self.immediate_u16(0);
        self.set_memory_with_offset_u8(address, n0, value);
    }

    fn set_memory_u16(&mut self, address: *mut u16, value: Self::U16) {
        let address_low = address.cast();
        let address_high = unsafe { address.byte_add(1) }.cast();
        let value_low = self.low_byte(value);
        let value_high = self.high_byte(value);
        self.set_memory_u8(address_low, value_low);
        self.set_memory_u8(address_high, value_high);
    }

    fn set_memory_with_offset_u8(&mut self, address: *mut u8, offset: Self::U16, value: Self::U8);

    fn get_bit(&mut self, value: Self::U8, bit_index: u8) -> Self::U1;

    fn not(&mut self, operand: Self::U1) -> Self::U1;

    fn is_zero(&mut self, operand: Self::U8) -> Self::U1;

    fn rotate_left(&mut self, operand: Self::U8, operand_carry: Self::U1) -> Self::U8;

    fn rotate_right(&mut self, operand: Self::U8, operand_carry: Self::U1) -> Self::U8;

    fn low_byte(&mut self, operand: Self::U16) -> Self::U8;

    fn high_byte(&mut self, operand: Self::U16) -> Self::U8;

    fn less_than_or_equal(&mut self, operand_0: Self::U16, operand_1: Self::U16) -> Self::U1;

    fn select(
        &mut self,
        condition: Self::U1,
        value_if_true: Self::U16,
        value_if_false: Self::U16,
    ) -> Self::U16;

    fn concatenate(&mut self, operand_0: Self::U8, operand_1: Self::U8) -> Self::U16;

    fn or(&mut self, operand_0: Self::U8, operand_1: Self::U8) -> Self::U8;

    fn and_u1(&mut self, operand_0: Self::U1, operand_1: Self::U1) -> Self::U1;

    fn and_u8(&mut self, operand_0: Self::U8, operand_1: Self::U8) -> Self::U8;

    fn and_u16(&mut self, operand_0: Self::U16, operand_1: Self::U16) -> Self::U16 {
        let operand_0_low = self.low_byte(operand_0);
        let operand_1_low = self.low_byte(operand_1);
        let result_low = self.and_u8(operand_0_low, operand_1_low);

        let operand_0_high = self.high_byte(operand_0);
        let operand_1_high = self.high_byte(operand_1);
        let result_high = self.and_u8(operand_0_high, operand_1_high);

        self.concatenate(result_high, result_low)
    }

    fn xor(&mut self, operand_0: Self::U8, operand_1: Self::U8) -> Self::U8;

    fn add_u8(&mut self, operand_0: Self::U8, operand_1: Self::U8) -> Self::U8 {
        let r#false = self.immediate_u1(false);
        self.add_with_carry_u8(operand_0, operand_1, r#false)
    }

    fn add_with_carry_u8(
        &mut self,
        operand_0: Self::U8,
        operand_1: Self::U8,
        operand_carry: Self::U1,
    ) -> Self::U8;

    fn add_u8_carry(&mut self, operand_0: Self::U8, operand_1: Self::U8) -> Self::U1 {
        let r#false = self.immediate_u1(false);
        self.add_with_carry_u8_carry(operand_0, operand_1, r#false)
    }

    fn add_with_carry_u8_carry(
        &mut self,
        operand_0: Self::U8,
        operand_1: Self::U8,
        operand_carry: Self::U1,
    ) -> Self::U1;

    fn add_with_carry_u8_overflow(
        &mut self,
        operand_0: Self::U8,
        operand_1: Self::U8,
        operand_carry: Self::U1,
    ) -> Self::U1;

    fn add_u16(&mut self, operand_0: Self::U16, operand_1: Self::U16) -> Self::U16 {
        let operand_0_low = self.low_byte(operand_0);
        let operand_1_low = self.low_byte(operand_1);
        let result_low = self.add_u8(operand_0_low, operand_1_low);
        let carry = self.add_u8_carry(operand_0_low, operand_1_low);

        let operand_0_high = self.high_byte(operand_0);
        let operand_1_high = self.high_byte(operand_1);
        let result_high = self.add_with_carry_u8(operand_0_high, operand_1_high, carry);

        self.concatenate(result_high, result_low)
    }

    fn sub(&mut self, operand_0: Self::U8, operand_1: Self::U8) -> Self::U8 {
        let r#false = self.immediate_u1(false);
        self.sub_with_borrow(operand_0, operand_1, r#false)
    }

    fn sub_with_borrow(
        &mut self,
        operand_0: Self::U8,
        operand_1: Self::U8,
        operand_borrow: Self::U1,
    ) -> Self::U8;

    fn sub_borrow(&mut self, operand_0: Self::U8, operand_1: Self::U8) -> Self::U1 {
        let r#false = self.immediate_u1(false);
        self.sub_with_borrow_borrow(operand_0, operand_1, r#false)
    }

    fn sub_with_borrow_borrow(
        &mut self,
        operand_0: Self::U8,
        operand_1: Self::U8,
        operand_borrow: Self::U1,
    ) -> Self::U1;

    fn sub_with_borrow_overflow(
        &mut self,
        operand_0: Self::U8,
        operand_1: Self::U8,
        operand_borrow: Self::U1,
    ) -> Self::U1;

    fn r#if(&mut self, condition: Self::U1, visit_true: impl FnMut(Self)) {
        self.if_else(condition, visit_true, |visitor| {
            visitor.terminate(None);
        });
    }

    fn if_else(
        &mut self,
        condition: Self::U1,
        visit_true: impl FnMut(Self),
        visit_false: impl FnMut(Self),
    );

    fn if_else_with_result(
        &mut self,
        condition: Self::U1,
        visit_true: impl FnMut(Self),
        visit_false: impl FnMut(Self),
    ) -> Self::U8;

    fn terminate(self, argument: Option<Self::U8>);
}
