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

    fn cpu_c(&mut self) -> Self::U1;

    fn set_cpu_c(&mut self, value: Self::U1);

    fn cpu_z(&mut self) -> Self::U1;

    fn set_cpu_z(&mut self, value: Self::U1);

    fn set_cpu_i(&mut self, value: Self::U1);

    fn set_cpu_d(&mut self, value: Self::U1);

    fn cpu_b(&mut self) -> Self::U1;

    fn set_cpu_b(&mut self, value: Self::U1);

    fn cpu_unused_flag(&mut self) -> Self::U1;

    fn set_cpu_unused_flag(&mut self, value: Self::U1);

    fn cpu_v(&mut self) -> Self::U1;

    fn set_cpu_v(&mut self, value: Self::U1);

    fn cpu_n(&mut self) -> Self::U1;

    fn set_cpu_n(&mut self, value: Self::U1);

    fn cpu_a(&mut self) -> Self::U8;

    fn set_cpu_a(&mut self, value: Self::U8);

    fn cpu_x(&mut self) -> Self::U8;

    fn set_cpu_x(&mut self, value: Self::U8);

    fn cpu_y(&mut self) -> Self::U8;

    fn set_cpu_y(&mut self, value: Self::U8);

    fn cpu_s(&mut self) -> Self::U8;

    fn set_cpu_s(&mut self, value: Self::U8);

    fn cpu_p(&mut self) -> Self::U8;

    fn set_cpu_p(&mut self, value: Self::U8);

    fn cpu_pc(&mut self) -> Self::U16;

    fn set_cpu_pc(&mut self, value: Self::U16);

    fn ppu_control_register(&mut self) -> Self::U8;

    fn set_ppu_control_register(&mut self, value: Self::U8);

    fn ppu_read_buffer(&mut self) -> Self::U8;

    fn set_ppu_read_buffer(&mut self, value: Self::U8);

    fn ppu_current_address(&mut self) -> Self::U16;

    fn set_ppu_current_address(&mut self, value: Self::U16);

    fn cpu_ram(&mut self, address: Self::U16) -> Self::U8;

    fn set_cpu_ram(&mut self, address: Self::U16, value: Self::U8);

    fn prg_ram(&mut self, address: Self::U16) -> Self::U8;

    fn set_prg_ram(&mut self, address: Self::U16, value: Self::U8);

    fn ppu_ram(&mut self, address: Self::U16) -> Self::U8;

    fn set_ppu_ram(&mut self, address: Self::U16, value: Self::U8);

    fn ppu_palette_ram(&mut self, address: Self::U16) -> Self::U8;

    fn set_ppu_palette_ram(&mut self, address: Self::U16, value: Self::U8);

    fn rom(&mut self, address: Self::U16) -> Self::U8;

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

    fn r#if(&mut self, condition: Self::U1, visit_true: impl Fn(Self)) {
        self.if_else(condition, visit_true, |visitor| {
            visitor.terminate(None);
        });
    }

    fn if_else(
        &mut self,
        condition: Self::U1,
        visit_true: impl Fn(Self),
        visit_false: impl Fn(Self),
    );

    fn if_else_with_result(
        &mut self,
        condition: Self::U1,
        visit_true: impl Fn(Self),
        visit_false: impl Fn(Self),
    ) -> Self::U8;

    fn terminate(self, argument: Option<Self::U8>);
}
