use crate::{
    compiler::frontend::nes::{Nes, cpu_memory},
    nes_assembly,
};
use tracing::warn;

pub struct Cpu {
    pub ram: [u8; 0x800],
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
    pub s: u8,
    pub pc: u16,
}

impl Cpu {
    pub fn new(pc: u16) -> Self {
        Cpu {
            ram: [0; 0x800],
            a: 0,
            x: 0,
            y: 0,
            p: 0,
            s: 0,
            pc,
        }
    }

    #[expect(clippy::too_many_lines)]
    pub(crate) fn compile<Visitor: super::Visitor>(
        nes: &mut Nes,
        visitor: Visitor,
        cpu_instruction: &nes_assembly::Instruction,
    ) {
        let mut owned_visitor = visitor;
        let visitor = &mut owned_visitor;

        let mut jump_target = None;

        match cpu_instruction.operation().mnemonic() {
            nes_assembly::Mnemonic::Adc => {
                let operand_0 = visitor.memory_u8(&raw const nes.cpu.a);
                let operand_1 = Self::read_operand_u8(nes, visitor, cpu_instruction);
                let operand_carry = Self::cpu_c(nes, visitor);

                let result = visitor.add_with_carry_u8(operand_0, operand_1, operand_carry);
                let result_carry =
                    visitor.add_with_carry_u8_carry(operand_0, operand_1, operand_carry);
                let result_overflow =
                    visitor.add_with_carry_u8_overflow(operand_0, operand_1, operand_carry);

                visitor.set_memory_u8(&raw mut nes.cpu.a, result);
                Self::set_cpu_c(nes, visitor, result_carry);
                Self::set_cpu_v(nes, visitor, result_overflow);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::And => {
                let operand_0 = visitor.memory_u8(&raw const nes.cpu.a);
                let operand_1 = Self::read_operand_u8(nes, visitor, cpu_instruction);

                let result = visitor.and_u8(operand_0, operand_1);

                visitor.set_memory_u8(&raw mut nes.cpu.a, result);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Asl => {
                let operand = Self::read_operand_u8(nes, visitor, cpu_instruction);
                let operand_carry = visitor.immediate_u1(false);

                let result = visitor.rotate_left(operand, operand_carry);
                let result_carry = visitor.get_bit(operand, 7);

                Self::write_operand_u8(nes, visitor, cpu_instruction, result);
                Self::set_cpu_c(nes, visitor, result_carry);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Bcc => {
                let c = Self::cpu_c(nes, visitor);
                let not_c = visitor.not(c);
                let address_if_true = Self::read_operand_u16(nes, visitor, cpu_instruction);
                let address_if_false = visitor.immediate_u16(cpu_instruction.address_end());
                jump_target = Some(visitor.select(not_c, address_if_true, address_if_false));
            }
            nes_assembly::Mnemonic::Bcs => {
                let c = Self::cpu_c(nes, visitor);
                let address_if_true = Self::read_operand_u16(nes, visitor, cpu_instruction);
                let address_if_false = visitor.immediate_u16(cpu_instruction.address_end());
                jump_target = Some(visitor.select(c, address_if_true, address_if_false));
            }
            nes_assembly::Mnemonic::Beq => {
                let z = Self::cpu_z(nes, visitor);
                let address_if_true = Self::read_operand_u16(nes, visitor, cpu_instruction);
                let address_if_false = visitor.immediate_u16(cpu_instruction.address_end());
                jump_target = Some(visitor.select(z, address_if_true, address_if_false));
            }
            nes_assembly::Mnemonic::Bit => {
                let operand = Self::read_operand_u8(nes, visitor, cpu_instruction);
                let n = visitor.get_bit(operand, 7);
                let v = visitor.get_bit(operand, 6);
                let a = visitor.memory_u8(&raw const nes.cpu.a);
                let result = visitor.and_u8(a, operand);
                let z = visitor.is_zero(result);

                Self::set_cpu_n(nes, visitor, n);
                Self::set_cpu_v(nes, visitor, v);
                Self::set_cpu_z(nes, visitor, z);
            }
            nes_assembly::Mnemonic::Bmi => {
                let n = Self::cpu_n(nes, visitor);
                let address_if_true = Self::read_operand_u16(nes, visitor, cpu_instruction);
                let address_if_false = visitor.immediate_u16(cpu_instruction.address_end());
                jump_target = Some(visitor.select(n, address_if_true, address_if_false));
            }
            nes_assembly::Mnemonic::Bne => {
                let z = Self::cpu_z(nes, visitor);
                let not_z = visitor.not(z);
                let address_if_true = Self::read_operand_u16(nes, visitor, cpu_instruction);
                let address_if_false = visitor.immediate_u16(cpu_instruction.address_end());
                jump_target = Some(visitor.select(not_z, address_if_true, address_if_false));
            }
            nes_assembly::Mnemonic::Bpl => {
                let n = Self::cpu_n(nes, visitor);
                let not_n = visitor.not(n);
                let address_if_true = Self::read_operand_u16(nes, visitor, cpu_instruction);
                let address_if_false = visitor.immediate_u16(cpu_instruction.address_end());
                jump_target = Some(visitor.select(not_n, address_if_true, address_if_false));
            }
            nes_assembly::Mnemonic::Brk => {
                let r#true = visitor.immediate_u1(true);
                let n2 = visitor.immediate_u16(2);

                let pc = visitor.memory_u16(&raw const nes.cpu.pc);
                let pc_plus_two = visitor.add_u16(pc, n2);
                let irq_handler_address = visitor.immediate_u16(0xfffe);
                let irq_handler = Self::read_u16_deref(nes, visitor, irq_handler_address);

                Self::set_cpu_unused_flag(nes, visitor, r#true);
                Self::set_cpu_b(nes, visitor, r#true);

                let p = visitor.memory_u8(&raw const nes.cpu.p);

                Self::set_cpu_i(nes, visitor, r#true);
                Self::push_u16(nes, visitor, pc_plus_two);
                Self::push_u8(nes, visitor, p);
                jump_target = Some(irq_handler);
            }
            nes_assembly::Mnemonic::Bvc => {
                let v = Self::cpu_v(nes, visitor);
                let not_v = visitor.not(v);
                let address_if_true = Self::read_operand_u16(nes, visitor, cpu_instruction);
                let address_if_false = visitor.immediate_u16(cpu_instruction.address_end());
                jump_target = Some(visitor.select(not_v, address_if_true, address_if_false));
            }
            nes_assembly::Mnemonic::Bvs => {
                let v = Self::cpu_v(nes, visitor);
                let address_if_true = Self::read_operand_u16(nes, visitor, cpu_instruction);
                let address_if_false = visitor.immediate_u16(cpu_instruction.address_end());
                jump_target = Some(visitor.select(v, address_if_true, address_if_false));
            }
            nes_assembly::Mnemonic::Clc => {
                let r#false = visitor.immediate_u1(false);

                Self::set_cpu_c(nes, visitor, r#false);
            }
            nes_assembly::Mnemonic::Cld => {
                let r#false = visitor.immediate_u1(false);

                Self::set_cpu_d(nes, visitor, r#false);
            }
            nes_assembly::Mnemonic::Cli => {
                let r#false = visitor.immediate_u1(false);

                Self::set_cpu_i(nes, visitor, r#false);
            }
            nes_assembly::Mnemonic::Clv => {
                let r#false = visitor.immediate_u1(false);

                Self::set_cpu_v(nes, visitor, r#false);
            }
            nes_assembly::Mnemonic::Cmp => {
                let operand_0 = visitor.memory_u8(&raw const nes.cpu.a);
                let operand_1 = Self::read_operand_u8(nes, visitor, cpu_instruction);

                let result = visitor.sub(operand_0, operand_1);
                let result_borrow = visitor.sub_borrow(operand_0, operand_1);
                let result_carry = visitor.not(result_borrow);

                Self::set_cpu_c(nes, visitor, result_carry);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Cpx => {
                let operand_0 = visitor.memory_u8(&raw const nes.cpu.x);
                let operand_1 = Self::read_operand_u8(nes, visitor, cpu_instruction);

                let result = visitor.sub(operand_0, operand_1);
                let result_borrow = visitor.sub_borrow(operand_0, operand_1);
                let result_carry = visitor.not(result_borrow);

                Self::set_cpu_c(nes, visitor, result_carry);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Cpy => {
                let operand_0 = visitor.memory_u8(&raw const nes.cpu.y);
                let operand_1 = Self::read_operand_u8(nes, visitor, cpu_instruction);

                let result = visitor.sub(operand_0, operand_1);
                let result_borrow = visitor.sub_borrow(operand_0, operand_1);
                let result_carry = visitor.not(result_borrow);

                Self::set_cpu_c(nes, visitor, result_carry);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Dec => {
                let operand_0 = Self::read_operand_u8(nes, visitor, cpu_instruction);
                let operand_1 = visitor.immediate_u8(1);

                let result = visitor.sub(operand_0, operand_1);

                Self::write_operand_u8(nes, visitor, cpu_instruction, result);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Dex => {
                let operand_0 = visitor.memory_u8(&raw const nes.cpu.x);
                let operand_1 = visitor.immediate_u8(1);

                let result = visitor.sub(operand_0, operand_1);

                visitor.set_memory_u8(&raw mut nes.cpu.x, result);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Dey => {
                let operand_0 = visitor.memory_u8(&raw const nes.cpu.y);
                let operand_1 = visitor.immediate_u8(1);

                let result = visitor.sub(operand_0, operand_1);

                visitor.set_memory_u8(&raw mut nes.cpu.y, result);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Eor => {
                let operand_0 = visitor.memory_u8(&raw const nes.cpu.a);
                let operand_1 = Self::read_operand_u8(nes, visitor, cpu_instruction);

                let result = visitor.xor(operand_0, operand_1);

                visitor.set_memory_u8(&raw mut nes.cpu.a, result);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Inc => {
                let operand_0 = Self::read_operand_u8(nes, visitor, cpu_instruction);
                let operand_1 = visitor.immediate_u8(1);

                let result = visitor.add_u8(operand_0, operand_1);

                Self::write_operand_u8(nes, visitor, cpu_instruction, result);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Inx => {
                let operand_0 = visitor.memory_u8(&raw const nes.cpu.x);
                let operand_1 = visitor.immediate_u8(1);

                let result = visitor.add_u8(operand_0, operand_1);

                visitor.set_memory_u8(&raw mut nes.cpu.x, result);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Iny => {
                let operand_0 = visitor.memory_u8(&raw const nes.cpu.y);
                let operand_1 = visitor.immediate_u8(1);

                let result = visitor.add_u8(operand_0, operand_1);

                visitor.set_memory_u8(&raw mut nes.cpu.y, result);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Jmp => {
                let address = Self::read_operand_u16(nes, visitor, cpu_instruction);

                jump_target = Some(address);
            }
            nes_assembly::Mnemonic::Jsr => {
                let n2 = visitor.immediate_u16(2);

                let pc = visitor.memory_u16(&raw const nes.cpu.pc);
                let pc_plus_2 = visitor.add_u16(pc, n2);
                let address = Self::read_operand_u16(nes, visitor, cpu_instruction);

                Self::push_u16(nes, visitor, pc_plus_2);
                jump_target = Some(address);
            }
            nes_assembly::Mnemonic::Lda => {
                let result = Self::read_operand_u8(nes, visitor, cpu_instruction);

                visitor.set_memory_u8(&raw mut nes.cpu.a, result);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Ldx => {
                let result = Self::read_operand_u8(nes, visitor, cpu_instruction);

                visitor.set_memory_u8(&raw mut nes.cpu.x, result);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Ldy => {
                let result = Self::read_operand_u8(nes, visitor, cpu_instruction);

                visitor.set_memory_u8(&raw mut nes.cpu.y, result);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Lsr => {
                let operand = Self::read_operand_u8(nes, visitor, cpu_instruction);
                let operand_carry = visitor.immediate_u1(false);

                let result = visitor.rotate_right(operand, operand_carry);
                let result_carry = visitor.get_bit(operand, 0);

                Self::write_operand_u8(nes, visitor, cpu_instruction, result);
                Self::set_cpu_c(nes, visitor, result_carry);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Nop => {}
            nes_assembly::Mnemonic::Ora => {
                let operand_0 = visitor.memory_u8(&raw const nes.cpu.a);
                let operand_1 = Self::read_operand_u8(nes, visitor, cpu_instruction);

                let result = visitor.or(operand_0, operand_1);

                visitor.set_memory_u8(&raw mut nes.cpu.a, result);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Pha => {
                let value = visitor.memory_u8(&raw const nes.cpu.a);

                Self::push_u8(nes, visitor, value);
            }
            nes_assembly::Mnemonic::Php => {
                let set_flags_mask = visitor.immediate_u8((1 << 5) | (1 << 4));

                let value = visitor.memory_u8(&raw const nes.cpu.p);
                let value = visitor.or(value, set_flags_mask);

                Self::push_u8(nes, visitor, value);
            }
            nes_assembly::Mnemonic::Pla => {
                let value = Self::pop_u8(nes, visitor);

                visitor.set_memory_u8(&raw mut nes.cpu.a, value);
                Self::set_cpu_nz(nes, visitor, value);
            }
            nes_assembly::Mnemonic::Plp => {
                let b = Self::cpu_b(nes, visitor);
                let unused_flag = Self::cpu_unused_flag(nes, visitor);
                let value = Self::pop_u8(nes, visitor);

                visitor.set_memory_u8(&raw mut nes.cpu.p, value);
                Self::set_cpu_b(nes, visitor, b);
                Self::set_cpu_unused_flag(nes, visitor, unused_flag);
            }
            nes_assembly::Mnemonic::Rol => {
                let operand = Self::read_operand_u8(nes, visitor, cpu_instruction);
                let operand_carry = Self::cpu_c(nes, visitor);

                let result = visitor.rotate_left(operand, operand_carry);
                let result_carry = visitor.get_bit(operand, 7);

                Self::write_operand_u8(nes, visitor, cpu_instruction, result);
                Self::set_cpu_c(nes, visitor, result_carry);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Ror => {
                let operand = Self::read_operand_u8(nes, visitor, cpu_instruction);
                let operand_carry = Self::cpu_c(nes, visitor);

                let result = visitor.rotate_right(operand, operand_carry);
                let result_carry = visitor.get_bit(operand, 0);

                Self::write_operand_u8(nes, visitor, cpu_instruction, result);
                Self::set_cpu_c(nes, visitor, result_carry);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Rti => {
                let unused_flag = Self::cpu_unused_flag(nes, visitor);
                let p = Self::pop_u8(nes, visitor);
                let return_address = Self::pop_u16(nes, visitor);

                visitor.set_memory_u8(&raw mut nes.cpu.p, p);
                Self::set_cpu_unused_flag(nes, visitor, unused_flag);
                jump_target = Some(return_address);
            }
            nes_assembly::Mnemonic::Rts => {
                let n1 = visitor.immediate_u16(1);

                let return_address_minus_1 = Self::pop_u16(nes, visitor);
                let return_address = visitor.add_u16(return_address_minus_1, n1);

                jump_target = Some(return_address);
            }
            nes_assembly::Mnemonic::Sbc => {
                let operand_0 = visitor.memory_u8(&raw const nes.cpu.a);
                let operand_1 = Self::read_operand_u8(nes, visitor, cpu_instruction);
                let operand_carry = Self::cpu_c(nes, visitor);
                let operand_borrow = visitor.not(operand_carry);

                let result = visitor.sub_with_borrow(operand_0, operand_1, operand_borrow);
                let result_borrow =
                    visitor.sub_with_borrow_borrow(operand_0, operand_1, operand_borrow);
                let result_carry = visitor.not(result_borrow);
                let result_overflow =
                    visitor.sub_with_borrow_overflow(operand_0, operand_1, operand_borrow);

                visitor.set_memory_u8(&raw mut nes.cpu.a, result);
                Self::set_cpu_c(nes, visitor, result_carry);
                Self::set_cpu_v(nes, visitor, result_overflow);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Sec => {
                let r#true = visitor.immediate_u1(true);
                Self::set_cpu_c(nes, visitor, r#true);
            }
            nes_assembly::Mnemonic::Sed => {
                let r#true = visitor.immediate_u1(true);
                Self::set_cpu_d(nes, visitor, r#true);
            }
            nes_assembly::Mnemonic::Sei => {
                let r#true = visitor.immediate_u1(true);
                Self::set_cpu_i(nes, visitor, r#true);
            }
            nes_assembly::Mnemonic::Sta => {
                let result = visitor.memory_u8(&raw const nes.cpu.a);
                Self::write_operand_u8(nes, visitor, cpu_instruction, result);
            }
            nes_assembly::Mnemonic::Stx => {
                let result = visitor.memory_u8(&raw const nes.cpu.x);
                Self::write_operand_u8(nes, visitor, cpu_instruction, result);
            }
            nes_assembly::Mnemonic::Sty => {
                let result = visitor.memory_u8(&raw const nes.cpu.y);
                Self::write_operand_u8(nes, visitor, cpu_instruction, result);
            }
            nes_assembly::Mnemonic::Tax => {
                let result = visitor.memory_u8(&raw const nes.cpu.a);
                visitor.set_memory_u8(&raw mut nes.cpu.x, result);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Tay => {
                let result = visitor.memory_u8(&raw const nes.cpu.a);
                visitor.set_memory_u8(&raw mut nes.cpu.y, result);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Tsx => {
                let result = visitor.memory_u8(&raw const nes.cpu.s);
                visitor.set_memory_u8(&raw mut nes.cpu.x, result);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Txa => {
                let result = visitor.memory_u8(&raw const nes.cpu.x);
                visitor.set_memory_u8(&raw mut nes.cpu.a, result);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Txs => {
                let result = visitor.memory_u8(&raw const nes.cpu.x);
                visitor.set_memory_u8(&raw mut nes.cpu.s, result);
            }
            nes_assembly::Mnemonic::Tya => {
                let result = visitor.memory_u8(&raw const nes.cpu.y);
                visitor.set_memory_u8(&raw mut nes.cpu.a, result);
                Self::set_cpu_nz(nes, visitor, result);
            }
            nes_assembly::Mnemonic::Unimplemented => {
                // unimplemented instructions are treated as a no-op as a
                // fallback
                warn!("compiling unimplemented instruction");
            }
        }

        let pc = jump_target.unwrap_or(visitor.immediate_u16(cpu_instruction.address_end()));
        visitor.set_memory_u16(&raw mut nes.cpu.pc, pc);
        owned_visitor.terminate(None);
    }

    fn read_u16_deref<Visitor: super::Visitor>(
        nes: &mut Nes,
        visitor: &mut Visitor,
        address: Visitor::U16,
    ) -> Visitor::U16 {
        let n1 = visitor.immediate_u8(1);

        let low_address = address;

        // intentionally apply page wrapping to the high byte address, matching the
        // behavior of the original hardware
        let high_address_high = visitor.high_byte(low_address);
        let high_address_low = visitor.low_byte(low_address);
        let high_address_low = visitor.add_u8(high_address_low, n1);
        let high_address = visitor.concatenate(high_address_high, high_address_low);

        let low = cpu_memory::read(nes, visitor, low_address);
        let high = cpu_memory::read(nes, visitor, high_address);
        visitor.concatenate(high, low)
    }

    fn cpu_c<Visitor: super::Visitor>(nes: &mut Nes, visitor: &mut Visitor) -> Visitor::U1 {
        Self::get_cpu_flag::<_, 0>(nes, visitor)
    }

    fn set_cpu_c<Visitor: super::Visitor>(
        nes: &mut Nes,
        visitor: &mut Visitor,
        value: Visitor::U1,
    ) {
        Self::set_cpu_flag::<_, 0>(nes, visitor, value);
    }

    fn cpu_z<Visitor: super::Visitor>(nes: &mut Nes, visitor: &mut Visitor) -> Visitor::U1 {
        Self::get_cpu_flag::<_, 1>(nes, visitor)
    }

    fn set_cpu_z<Visitor: super::Visitor>(
        nes: &mut Nes,
        visitor: &mut Visitor,
        value: Visitor::U1,
    ) {
        Self::set_cpu_flag::<_, 1>(nes, visitor, value);
    }

    fn set_cpu_i<Visitor: super::Visitor>(
        nes: &mut Nes,
        visitor: &mut Visitor,
        value: Visitor::U1,
    ) {
        Self::set_cpu_flag::<_, 2>(nes, visitor, value);
    }

    fn set_cpu_d<Visitor: super::Visitor>(
        nes: &mut Nes,
        visitor: &mut Visitor,
        value: Visitor::U1,
    ) {
        Self::set_cpu_flag::<_, 3>(nes, visitor, value);
    }

    fn cpu_b<Visitor: super::Visitor>(nes: &mut Nes, visitor: &mut Visitor) -> Visitor::U1 {
        Self::get_cpu_flag::<_, 4>(nes, visitor)
    }

    fn set_cpu_b<Visitor: super::Visitor>(
        nes: &mut Nes,
        visitor: &mut Visitor,
        value: Visitor::U1,
    ) {
        Self::set_cpu_flag::<_, 4>(nes, visitor, value);
    }

    fn cpu_unused_flag<Visitor: super::Visitor>(
        nes: &mut Nes,
        visitor: &mut Visitor,
    ) -> Visitor::U1 {
        Self::get_cpu_flag::<_, 5>(nes, visitor)
    }

    fn set_cpu_unused_flag<Visitor: super::Visitor>(
        nes: &mut Nes,
        visitor: &mut Visitor,
        value: Visitor::U1,
    ) {
        Self::set_cpu_flag::<_, 5>(nes, visitor, value);
    }

    fn cpu_v<Visitor: super::Visitor>(nes: &mut Nes, visitor: &mut Visitor) -> Visitor::U1 {
        Self::get_cpu_flag::<_, 6>(nes, visitor)
    }

    fn set_cpu_v<Visitor: super::Visitor>(
        nes: &mut Nes,
        visitor: &mut Visitor,
        value: Visitor::U1,
    ) {
        Self::set_cpu_flag::<_, 6>(nes, visitor, value);
    }

    fn cpu_n<Visitor: super::Visitor>(nes: &mut Nes, visitor: &mut Visitor) -> Visitor::U1 {
        Self::get_cpu_flag::<_, 7>(nes, visitor)
    }

    fn set_cpu_n<Visitor: super::Visitor>(
        nes: &mut Nes,
        visitor: &mut Visitor,
        value: Visitor::U1,
    ) {
        Self::set_cpu_flag::<_, 7>(nes, visitor, value);
    }

    fn get_cpu_flag<Visitor: super::Visitor, const INDEX: u8>(
        nes: &mut Nes,
        visitor: &mut Visitor,
    ) -> Visitor::U1 {
        let p = visitor.memory_u8(&raw const nes.cpu.p);
        visitor.get_bit(p, INDEX)
    }

    fn set_cpu_flag<Visitor: super::Visitor, const INDEX: u8>(
        nes: &mut Nes,
        visitor: &mut Visitor,
        value: Visitor::U1,
    ) {
        let clear_bit_mask = visitor.immediate_u8(!(1 << INDEX));

        let p = visitor.memory_u8(&raw const nes.cpu.p);
        let p = visitor.and_u8(p, clear_bit_mask);
        let p = visitor.if_else_with_result(
            value,
            |mut visitor| {
                let set_bit_mask = visitor.immediate_u8(1 << INDEX);
                let p = visitor.or(p, set_bit_mask);
                visitor.terminate(Some(p));
            },
            |visitor| {
                visitor.terminate(Some(p));
            },
        );
        visitor.set_memory_u8(&raw mut nes.cpu.p, p);
    }

    fn set_cpu_nz<Visitor: super::Visitor>(
        nes: &mut Nes,
        visitor: &mut Visitor,
        value: Visitor::U8,
    ) {
        let n = visitor.get_bit(value, 7);
        let z = visitor.is_zero(value);

        Self::set_cpu_n(nes, visitor, n);
        Self::set_cpu_z(nes, visitor, z);
    }

    fn push_u8<Visitor: super::Visitor>(nes: &mut Nes, visitor: &mut Visitor, value: Visitor::U8) {
        let n1 = visitor.immediate_u8(1);

        let s = visitor.memory_u8(&raw const nes.cpu.s);
        let s_minus_1 = visitor.sub(s, n1);
        let address = visitor.concatenate(n1, s);

        cpu_memory::write(nes, visitor, address, value);
        visitor.set_memory_u8(&raw mut nes.cpu.s, s_minus_1);
    }

    fn push_u16<Visitor: super::Visitor>(
        nes: &mut Nes,
        visitor: &mut Visitor,
        value: Visitor::U16,
    ) {
        let low = visitor.low_byte(value);
        let high = visitor.high_byte(value);

        Self::push_u8(nes, visitor, high);
        Self::push_u8(nes, visitor, low);
    }

    fn pop_u8<Visitor: super::Visitor>(nes: &mut Nes, visitor: &mut Visitor) -> Visitor::U8 {
        let n1 = visitor.immediate_u8(1);

        let s = visitor.memory_u8(&raw const nes.cpu.s);
        let s_plus_1 = visitor.add_u8(s, n1);
        let result_address = visitor.concatenate(n1, s_plus_1);
        let result = cpu_memory::read(nes, visitor, result_address);

        visitor.set_memory_u8(&raw mut nes.cpu.s, s_plus_1);
        result
    }

    fn pop_u16<Visitor: super::Visitor>(nes: &mut Nes, visitor: &mut Visitor) -> Visitor::U16 {
        let low = Self::pop_u8(nes, visitor);
        let high = Self::pop_u8(nes, visitor);

        visitor.concatenate(high, low)
    }

    fn get_operand_address<Visitor: super::Visitor>(
        nes: &mut Nes,
        visitor: &mut Visitor,
        cpu_instruction: &nes_assembly::Instruction,
    ) -> Visitor::U16 {
        match cpu_instruction.operation().addressing_mode() {
            nes_assembly::AddressingMode::Absolute | nes_assembly::AddressingMode::Zeropage => {
                visitor.immediate_u16(cpu_instruction.operand_u16())
            }
            nes_assembly::AddressingMode::AbsoluteX => {
                let n0 = visitor.immediate_u8(0);

                let x = visitor.memory_u8(&raw const nes.cpu.x);
                let operand_0 = visitor.immediate_u16(cpu_instruction.operand_u16());
                let operand_1 = visitor.concatenate(n0, x);
                visitor.add_u16(operand_0, operand_1)
            }
            nes_assembly::AddressingMode::AbsoluteY => {
                let n0 = visitor.immediate_u8(0);

                let y = visitor.memory_u8(&raw const nes.cpu.y);
                let y_u16 = visitor.concatenate(n0, y);
                let operand = visitor.immediate_u16(cpu_instruction.operand_u16());
                visitor.add_u16(operand, y_u16)
            }
            nes_assembly::AddressingMode::Accumulator
            | nes_assembly::AddressingMode::Immediate
            | nes_assembly::AddressingMode::Implied
            | nes_assembly::AddressingMode::Indirect
            | nes_assembly::AddressingMode::Relative => unreachable!(),
            nes_assembly::AddressingMode::IndirectY => {
                let n0 = visitor.immediate_u8(0);

                let operand = visitor.immediate_u16(cpu_instruction.operand_u16());
                let operand_0 = Self::read_u16_deref(nes, visitor, operand);
                let y = visitor.memory_u8(&raw const nes.cpu.y);
                let operand_1 = visitor.concatenate(n0, y);
                visitor.add_u16(operand_0, operand_1)
            }
            nes_assembly::AddressingMode::XIndirect => {
                let n0 = visitor.immediate_u8(0);

                let x = visitor.memory_u8(&raw const nes.cpu.x);
                let operand = visitor.immediate_u8(cpu_instruction.operand_u8());
                let address = visitor.add_u8(operand, x);
                let address = visitor.concatenate(n0, address);
                Self::read_u16_deref(nes, visitor, address)
            }
            nes_assembly::AddressingMode::ZeropageX => {
                let n0 = visitor.immediate_u8(0);
                let operand = visitor.immediate_u8(cpu_instruction.operand_u8());
                let x = visitor.memory_u8(&raw const nes.cpu.x);
                let address = visitor.add_u8(operand, x);
                visitor.concatenate(n0, address)
            }
            nes_assembly::AddressingMode::ZeropageY => {
                let n0 = visitor.immediate_u8(0);
                let operand = visitor.immediate_u8(cpu_instruction.operand_u8());
                let y = visitor.memory_u8(&raw const nes.cpu.y);
                let address = visitor.add_u8(operand, y);
                visitor.concatenate(n0, address)
            }
        }
    }

    fn read_operand_u8<Visitor: super::Visitor>(
        nes: &mut Nes,
        visitor: &mut Visitor,
        cpu_instruction: &nes_assembly::Instruction,
    ) -> Visitor::U8 {
        match cpu_instruction.operation().addressing_mode() {
            nes_assembly::AddressingMode::Absolute
            | nes_assembly::AddressingMode::Zeropage
            | nes_assembly::AddressingMode::AbsoluteX
            | nes_assembly::AddressingMode::AbsoluteY
            | nes_assembly::AddressingMode::IndirectY
            | nes_assembly::AddressingMode::XIndirect
            | nes_assembly::AddressingMode::ZeropageX
            | nes_assembly::AddressingMode::ZeropageY => {
                let address = Self::get_operand_address(nes, visitor, cpu_instruction);
                cpu_memory::read(nes, visitor, address)
            }
            nes_assembly::AddressingMode::Accumulator => visitor.memory_u8(&raw const nes.cpu.a),
            nes_assembly::AddressingMode::Immediate => {
                visitor.immediate_u8(cpu_instruction.operand_u8())
            }
            nes_assembly::AddressingMode::Implied
            | nes_assembly::AddressingMode::Indirect
            | nes_assembly::AddressingMode::Relative => {
                unreachable!()
            }
        }
    }

    fn write_operand_u8<Visitor: super::Visitor>(
        nes: &mut Nes,
        visitor: &mut Visitor,
        cpu_instruction: &nes_assembly::Instruction,
        value: Visitor::U8,
    ) {
        match cpu_instruction.operation().addressing_mode() {
            nes_assembly::AddressingMode::Absolute
            | nes_assembly::AddressingMode::Zeropage
            | nes_assembly::AddressingMode::AbsoluteX
            | nes_assembly::AddressingMode::AbsoluteY
            | nes_assembly::AddressingMode::IndirectY
            | nes_assembly::AddressingMode::XIndirect
            | nes_assembly::AddressingMode::ZeropageX
            | nes_assembly::AddressingMode::ZeropageY => {
                let address = Self::get_operand_address(nes, visitor, cpu_instruction);
                cpu_memory::write(nes, visitor, address, value);
            }
            nes_assembly::AddressingMode::Accumulator => {
                visitor.set_memory_u8(&raw mut nes.cpu.a, value);
            }
            nes_assembly::AddressingMode::Immediate
            | nes_assembly::AddressingMode::Implied
            | nes_assembly::AddressingMode::Indirect
            | nes_assembly::AddressingMode::Relative => {
                unreachable!()
            }
        }
    }

    fn read_operand_u16<Visitor: super::Visitor>(
        nes: &mut Nes,
        visitor: &mut Visitor,
        cpu_instruction: &nes_assembly::Instruction,
    ) -> Visitor::U16 {
        match cpu_instruction.operation().addressing_mode() {
            nes_assembly::AddressingMode::Absolute => {
                visitor.immediate_u16(cpu_instruction.operand_u16())
            }
            nes_assembly::AddressingMode::Indirect => {
                let address = visitor.immediate_u16(cpu_instruction.operand_u16());
                Self::read_u16_deref(nes, visitor, address)
            }
            nes_assembly::AddressingMode::Relative => visitor.immediate_u16(
                cpu_instruction
                    .address_end()
                    .wrapping_add_signed(i16::from(cpu_instruction.operand_i8())),
            ),
            _ => unreachable!(),
        }
    }
}
