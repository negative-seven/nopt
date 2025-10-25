pub(crate) trait Visitor {
    type U1: Copy;
    type U8: Copy;
    type U16: Copy;

    fn immediate_u1(&mut self, value: bool) -> Self::U1;

    fn immediate_u8(&mut self, value: u8) -> Self::U8;

    fn immediate_u16(&mut self, value: u16) -> Self::U16;

    fn cpu_c(&self) -> Self::U1;

    fn set_cpu_c(&self, variable: Self::U1);

    fn cpu_z(&self) -> Self::U1;

    fn set_cpu_z(&self, variable: Self::U1);

    fn set_cpu_i(&self, variable: Self::U1);

    fn set_cpu_d(&self, variable: Self::U1);

    fn cpu_b(&self) -> Self::U1;

    fn set_cpu_b(&self, variable: Self::U1);

    fn cpu_unused_flag(&self) -> Self::U1;

    fn set_cpu_unused_flag(&self, variable: Self::U1);

    fn cpu_v(&self) -> Self::U1;

    fn set_cpu_v(&self, variable: Self::U1);

    fn cpu_n(&self) -> Self::U1;

    fn set_cpu_n(&self, variable: Self::U1);

    fn cpu_a(&self) -> Self::U8;

    fn set_cpu_a(&self, variable: Self::U8);

    fn cpu_x(&self) -> Self::U8;

    fn set_cpu_x(&self, variable: Self::U8);

    fn cpu_y(&self) -> Self::U8;

    fn set_cpu_y(&self, variable: Self::U8);

    fn cpu_s(&self) -> Self::U8;

    fn set_cpu_s(&self, variable: Self::U8);

    fn cpu_p(&self) -> Self::U8;

    fn set_cpu_p(&self, variable: Self::U8);

    fn cpu_pc(&mut self) -> Self::U16;

    fn ppu_current_address(&mut self) -> Self::U16;

    fn set_ppu_current_address(&self, variable: Self::U16);

    fn cpu_ram(&mut self, address: Self::U16) -> Self::U8;

    fn set_cpu_ram(&self, address: Self::U16, variable: Self::U8);

    fn prg_ram(&mut self, address: Self::U16) -> Self::U8;

    fn set_prg_ram(&self, address: Self::U16, variable: Self::U8);

    fn ppu_ram(&mut self, address: Self::U16) -> Self::U8;

    fn set_ppu_ram(&self, address: Self::U16, variable: Self::U8);

    fn rom(&mut self, address: Self::U16) -> Self::U8;

    fn get_bit(&mut self, operand: Self::U8, index: u8) -> Self::U1;

    fn not(&mut self, operand: Self::U1) -> Self::U1;

    fn is_zero(&mut self, operand: Self::U8) -> Self::U1;

    fn is_negative(&mut self, operand: Self::U8) -> Self::U1;

    fn rotate_left(&mut self, operand: Self::U8, operand_carry: Self::U1) -> Self::U8;

    fn rotate_right(&mut self, operand: Self::U8, operand_carry: Self::U1) -> Self::U8;

    fn low_byte(&mut self, operand: Self::U16) -> Self::U8;

    fn high_byte(&mut self, operand: Self::U16) -> Self::U8;

    fn less_than_or_equal(&mut self, operand_0: Self::U16, operand_1: Self::U16) -> Self::U1;

    fn select(
        &mut self,
        condition: Self::U1,
        result_if_true: Self::U16,
        result_if_false: Self::U16,
    ) -> Self::U16;

    fn concatenate(&mut self, high: Self::U8, low: Self::U8) -> Self::U16;

    fn or(&mut self, operand_0: Self::U8, operand_1: Self::U8) -> Self::U8;

    fn and_u1(&mut self, operand_0: Self::U1, operand_1: Self::U1) -> Self::U1;

    fn and_u8(&mut self, operand_0: Self::U8, operand_1: Self::U8) -> Self::U8;

    fn xor(&mut self, operand_0: Self::U8, operand_1: Self::U8) -> Self::U8;

    fn add_u8(
        &mut self,
        operand_0: Self::U8,
        operand_1: Self::U8,
        operand_carry: Self::U1,
    ) -> Self::U8;

    fn add_u8_carry(
        &mut self,
        operand_0: Self::U8,
        operand_1: Self::U8,
        operand_carry: Self::U1,
    ) -> Self::U1;

    fn add_u8_overflow(
        &mut self,
        operand_0: Self::U8,
        operand_1: Self::U8,
        operand_carry: Self::U1,
    ) -> Self::U1;

    fn add_u16(&mut self, operand_0: Self::U16, operand_1: Self::U16) -> Self::U16;

    fn sub(
        &mut self,
        operand_0: Self::U8,
        operand_1: Self::U8,
        operand_borrow: Self::U1,
    ) -> Self::U8;

    fn sub_borrow(
        &mut self,
        operand_0: Self::U8,
        operand_1: Self::U8,
        operand_borrow: Self::U1,
    ) -> Self::U1;

    fn sub_overflow(
        &mut self,
        operand_0: Self::U8,
        operand_1: Self::U8,
        operand_borrow: Self::U1,
    ) -> Self::U1;

    fn if_else(
        &mut self,
        condition: Self::U1,
        populate_true_block: impl Fn(&mut Self),
        populate_false_block: impl Fn(&mut Self),
    );

    fn if_else_with_result(
        &mut self,
        condition: Self::U1,
        populate_true_block: impl Fn(&mut Self) -> Self::U8,
        populate_false_block: impl Fn(&mut Self) -> Self::U8,
    ) -> Self::U8;

    fn jump(&self, address: Self::U16);
}
