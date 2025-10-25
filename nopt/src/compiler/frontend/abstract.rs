mod cpu_memory;

use crate::{
    compiler::{
        frontend::CompilerVisitor,
        ir::{
            CpuFlag, CpuRegister, Definition1, Definition8, Definition16, Destination1,
            Destination8, Jump, Variable1, Variable8, Variable16,
        },
    },
    nes_assembly,
};
use tracing::warn;

pub(crate) struct Compiler {
    pub(crate) visitor: CompilerVisitor,
    pub(crate) cpu_instruction: nes_assembly::Instruction,
}

impl Compiler {
    #[expect(clippy::too_many_lines)]
    pub(crate) fn transpile(mut self) {
        let mut jump = None;

        match self.cpu_instruction.operation().mnemonic() {
            nes_assembly::Mnemonic::Adc => {
                let operand_0 = self.define_8(self.visitor.cpu_a());
                let operand_1 = self.read_operand_u8();
                let operand_carry = self.define_1(self.visitor.cpu_c());

                let result = self.visitor.add_u8(operand_0, operand_1, operand_carry);
                let result_carry = self
                    .visitor
                    .add_u8_carry(operand_0, operand_1, operand_carry);
                let result_overflow =
                    self.visitor
                        .add_u8_overflow(operand_0, operand_1, operand_carry);

                self.store_8(self.visitor.cpu_a(), result);
                self.store_1(self.visitor.cpu_c(), result_carry);
                self.store_1(self.visitor.cpu_v(), result_overflow);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::And => {
                let operand_0 = self.define_8(self.visitor.cpu_a());
                let operand_1 = self.read_operand_u8();

                let result = self.visitor.and(operand_0, operand_1);

                self.store_8(self.visitor.cpu_a(), result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Asl => {
                let operand = self.read_operand_u8();
                let operand_carry = self.define_1(false);

                let result = self.visitor.rotate_left(operand, operand_carry);
                let result_carry = self.visitor.get_bit(operand, 7);

                self.write_operand_u8(result);
                self.store_1(self.visitor.cpu_c(), result_carry);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Bcc => {
                let c = self.define_1(self.visitor.cpu_c());
                let not_c = self.define_1(!c);
                let address_if_true = self.read_operand_u16();
                let address_if_false = self.define_16(self.cpu_instruction.address_end());
                let jump_target = self
                    .visitor
                    .select(not_c, address_if_true, address_if_false);

                jump = Some(Jump::CpuAddress(jump_target));
            }
            nes_assembly::Mnemonic::Bcs => {
                let c = self.define_1(self.visitor.cpu_c());
                let address_if_true = self.read_operand_u16();
                let address_if_false = self.define_16(self.cpu_instruction.address_end());

                let jump_target = self.visitor.select(c, address_if_true, address_if_false);

                jump = Some(Jump::CpuAddress(jump_target));
            }
            nes_assembly::Mnemonic::Beq => {
                let z = self.define_1(self.visitor.cpu_z());
                let address_if_true = self.read_operand_u16();
                let address_if_false = self.define_16(self.cpu_instruction.address_end());
                let jump_target = self.visitor.select(z, address_if_true, address_if_false);

                jump = Some(Jump::CpuAddress(jump_target));
            }
            nes_assembly::Mnemonic::Bit => {
                let operand = self.read_operand_u8();
                let n = self.visitor.get_bit(operand, 7);
                let v = self.visitor.get_bit(operand, 6);
                let a = self.define_8(self.visitor.cpu_a());
                let result = self.define_8(a & operand);
                let z = self.visitor.is_zero(result);

                self.store_1(self.visitor.cpu_n(), n);
                self.store_1(self.visitor.cpu_v(), v);
                self.store_1(self.visitor.cpu_z(), z);
            }
            nes_assembly::Mnemonic::Bmi => {
                let n = self.define_1(self.visitor.cpu_n());
                let address_if_true = self.read_operand_u16();
                let address_if_false = self.define_16(self.cpu_instruction.address_end());
                let jump_target = self.visitor.select(n, address_if_true, address_if_false);

                jump = Some(Jump::CpuAddress(jump_target));
            }
            nes_assembly::Mnemonic::Bne => {
                let z = self.define_1(self.visitor.cpu_z());
                let not_z = self.define_1(!z);
                let address_if_true = self.read_operand_u16();
                let address_if_false = self.define_16(self.cpu_instruction.address_end());
                let jump_target = self
                    .visitor
                    .select(not_z, address_if_true, address_if_false);

                jump = Some(Jump::CpuAddress(jump_target));
            }
            nes_assembly::Mnemonic::Bpl => {
                let n = self.define_1(self.visitor.cpu_n());
                let not_n = self.define_1(!n);
                let address_if_true = self.read_operand_u16();
                let address_if_false = self.define_16(self.cpu_instruction.address_end());
                let jump_target = self
                    .visitor
                    .select(not_n, address_if_true, address_if_false);

                jump = Some(Jump::CpuAddress(jump_target));
            }
            nes_assembly::Mnemonic::Brk => {
                let r#true = self.define_1(true);
                let n2 = self.define_16(2);

                let pc = self.visitor.cpu_pc();
                let pc_plus_two = self.visitor.add_u16(pc, n2);
                let irq_handler_address = self.define_16(0xfffe);
                let irq_handler = self.read_u16_deref(irq_handler_address);

                self.store_1(self.visitor.cpu_unused_flag(), r#true);
                self.store_1(self.visitor.cpu_b(), r#true);

                let p = self.define_8(self.visitor.cpu_p());

                self.store_1(self.visitor.cpu_i(), r#true);
                self.push_u16(pc_plus_two);
                self.push_u8(p);
                jump = Some(Jump::CpuAddress(irq_handler));
            }
            nes_assembly::Mnemonic::Bvc => {
                let v = self.define_1(self.visitor.cpu_v());
                let not_v = self.define_1(!v);
                let address_if_true = self.read_operand_u16();
                let address_if_false = self.define_16(self.cpu_instruction.address_end());
                let jump_target = self
                    .visitor
                    .select(not_v, address_if_true, address_if_false);

                jump = Some(Jump::CpuAddress(jump_target));
            }
            nes_assembly::Mnemonic::Bvs => {
                let v = self.define_1(self.visitor.cpu_v());
                let address_if_true = self.read_operand_u16();
                let address_if_false = self.define_16(self.cpu_instruction.address_end());
                let jump_target = self.visitor.select(v, address_if_true, address_if_false);

                jump = Some(Jump::CpuAddress(jump_target));
            }
            nes_assembly::Mnemonic::Clc => {
                let r#false = self.define_1(false);

                self.store_1(self.visitor.cpu_c(), r#false);
            }
            nes_assembly::Mnemonic::Cld => {
                let r#false = self.define_1(false);

                self.store_1(self.visitor.cpu_d(), r#false);
            }
            nes_assembly::Mnemonic::Cli => {
                let r#false = self.define_1(false);

                self.store_1(self.visitor.cpu_i(), r#false);
            }
            nes_assembly::Mnemonic::Clv => {
                let r#false = self.define_1(false);

                self.store_1(self.visitor.cpu_v(), r#false);
            }
            nes_assembly::Mnemonic::Cmp => {
                let operand_0 = self.define_8(self.visitor.cpu_a());
                let operand_1 = self.read_operand_u8();
                let operand_borrow = self.define_1(false);

                let result = self.visitor.sub(operand_0, operand_1, operand_borrow);
                let result_borrow = self
                    .visitor
                    .sub_borrow(operand_0, operand_1, operand_borrow);
                let result_carry = self.visitor.not(result_borrow);

                self.store_1(self.visitor.cpu_c(), result_carry);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Cpx => {
                let operand_0 = self.define_8(self.visitor.cpu_x());
                let operand_1 = self.read_operand_u8();
                let operand_borrow = self.define_1(false);

                let result = self.visitor.sub(operand_0, operand_1, operand_borrow);
                let result_borrow = self
                    .visitor
                    .sub_borrow(operand_0, operand_1, operand_borrow);
                let result_carry = self.define_1(!result_borrow);

                self.store_1(self.visitor.cpu_c(), result_carry);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Cpy => {
                let operand_0 = self.define_8(self.visitor.cpu_y());
                let operand_1 = self.read_operand_u8();
                let operand_borrow = self.define_1(false);

                let result = self.visitor.sub(operand_0, operand_1, operand_borrow);
                let result_borrow = self
                    .visitor
                    .sub_borrow(operand_0, operand_1, operand_borrow);
                let result_carry = self.define_1(!result_borrow);

                self.store_1(self.visitor.cpu_c(), result_carry);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Dec => {
                let operand_0 = self.read_operand_u8();
                let operand_1 = self.define_8(1);
                let operand_borrow = self.define_1(false);

                let result = self.visitor.sub(operand_0, operand_1, operand_borrow);

                self.write_operand_u8(result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Dex => {
                let operand_0 = self.define_8(self.visitor.cpu_x());
                let operand_1 = self.define_8(1);
                let operand_borrow = self.define_1(false);

                let result = self.visitor.sub(operand_0, operand_1, operand_borrow);

                self.store_8(self.visitor.cpu_x(), result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Dey => {
                let operand_0 = self.define_8(self.visitor.cpu_y());
                let operand_1 = self.define_8(1);
                let operand_borrow = self.define_1(false);

                let result = self.visitor.sub(operand_0, operand_1, operand_borrow);

                self.store_8(self.visitor.cpu_y(), result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Eor => {
                let operand_0 = self.define_8(self.visitor.cpu_a());
                let operand_1 = self.read_operand_u8();

                let result = self.define_8(operand_0 ^ operand_1);

                self.store_8(self.visitor.cpu_a(), result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Inc => {
                let operand_0 = self.read_operand_u8();
                let operand_1 = self.define_8(1);
                let operand_carry = self.define_1(false);

                let result = self.visitor.add_u8(operand_0, operand_1, operand_carry);

                self.write_operand_u8(result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Inx => {
                let operand_0 = self.define_8(self.visitor.cpu_x());
                let operand_1 = self.define_8(1);
                let operand_carry = self.define_1(false);

                let result = self.visitor.add_u8(operand_0, operand_1, operand_carry);

                self.store_8(self.visitor.cpu_x(), result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Iny => {
                let operand_0 = self.define_8(self.visitor.cpu_y());
                let operand_1 = self.define_8(1);
                let operand_carry = self.define_1(false);

                let result = self.visitor.add_u8(operand_0, operand_1, operand_carry);

                self.store_8(self.visitor.cpu_y(), result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Jmp => {
                let address = self.read_operand_u16();

                jump = Some(Jump::CpuAddress(address));
            }
            nes_assembly::Mnemonic::Jsr => {
                let n2 = self.define_16(2);

                let pc = self.visitor.cpu_pc();
                let pc_plus_2 = self.visitor.add_u16(pc, n2);
                let address = self.read_operand_u16();

                self.push_u16(pc_plus_2);
                jump = Some(Jump::CpuAddress(address));
            }
            nes_assembly::Mnemonic::Lda => {
                let result = self.read_operand_u8();

                self.store_8(self.visitor.cpu_a(), result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Ldx => {
                let result = self.read_operand_u8();

                self.store_8(self.visitor.cpu_x(), result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Ldy => {
                let result = self.read_operand_u8();

                self.store_8(self.visitor.cpu_y(), result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Lsr => {
                let operand = self.read_operand_u8();
                let operand_carry = self.define_1(false);

                let result = self.visitor.rotate_right(operand, operand_carry);
                let result_carry = self.visitor.get_bit(operand, 0);

                self.write_operand_u8(result);
                self.store_1(self.visitor.cpu_c(), result_carry);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Nop => {}
            nes_assembly::Mnemonic::Ora => {
                let operand_0 = self.define_8(self.visitor.cpu_a());
                let operand_1 = self.read_operand_u8();

                let result = self.define_8(operand_0 | operand_1);

                self.store_8(self.visitor.cpu_a(), result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Pha => {
                let value = self.define_8(self.visitor.cpu_a());

                self.push_u8(value);
            }
            nes_assembly::Mnemonic::Php => {
                let set_flags_mask =
                    self.define_8((1 << CpuFlag::Unused.index()) | (1 << CpuFlag::B.index()));

                let value = self.define_8(self.visitor.cpu_p());
                let value = self.define_8(value | set_flags_mask);

                self.push_u8(value);
            }
            nes_assembly::Mnemonic::Pla => {
                let value = self.pop_u8();

                self.store_8(self.visitor.cpu_a(), value);
                self.set_nz(value);
            }
            nes_assembly::Mnemonic::Plp => {
                let b = self.define_1(self.visitor.cpu_b());
                let unused_flag = self.define_1(self.visitor.cpu_unused_flag());
                let value = self.pop_u8();

                self.store_8(self.visitor.cpu_p(), value);
                self.store_1(self.visitor.cpu_b(), b);
                self.store_1(self.visitor.cpu_unused_flag(), unused_flag);
            }
            nes_assembly::Mnemonic::Rol => {
                let operand = self.read_operand_u8();
                let operand_carry = self.define_1(self.visitor.cpu_c());

                let result = self.visitor.rotate_left(operand, operand_carry);
                let result_carry = self.visitor.get_bit(operand, 7);

                self.write_operand_u8(result);
                self.store_1(self.visitor.cpu_c(), result_carry);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Ror => {
                let operand = self.read_operand_u8();
                let operand_carry = self.define_1(self.visitor.cpu_c());

                let result = self.visitor.rotate_right(operand, operand_carry);
                let result_carry = self.visitor.get_bit(operand, 0);

                self.write_operand_u8(result);
                self.store_1(self.visitor.cpu_c(), result_carry);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Rti => {
                let unused_flag = self.define_1(self.visitor.cpu_unused_flag());
                let p = self.pop_u8();
                let return_address = self.pop_u16();

                self.store_8(self.visitor.cpu_p(), p);
                self.store_1(self.visitor.cpu_unused_flag(), unused_flag);
                jump = Some(Jump::CpuAddress(return_address));
            }
            nes_assembly::Mnemonic::Rts => {
                let n1 = self.define_16(1);

                let return_address_minus_1 = self.pop_u16();
                let return_address = self.visitor.add_u16(return_address_minus_1, n1);

                jump = Some(Jump::CpuAddress(return_address));
            }
            nes_assembly::Mnemonic::Sbc => {
                let operand_0 = self.define_8(self.visitor.cpu_a());
                let operand_1 = self.read_operand_u8();
                let operand_carry = self.define_1(self.visitor.cpu_c());
                let operand_borrow = self.define_1(!operand_carry);

                let result = self.visitor.sub(operand_0, operand_1, operand_borrow);
                let result_borrow = self
                    .visitor
                    .sub_borrow(operand_0, operand_1, operand_borrow);
                let result_carry = self.visitor.not(result_borrow);
                let result_overflow =
                    self.visitor
                        .sub_overflow(operand_0, operand_1, operand_borrow);

                self.store_8(self.visitor.cpu_a(), result);
                self.store_1(self.visitor.cpu_c(), result_carry);
                self.store_1(self.visitor.cpu_v(), result_overflow);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Sec => {
                let r#true = self.define_1(true);
                self.store_1(self.visitor.cpu_c(), r#true);
            }
            nes_assembly::Mnemonic::Sed => {
                let r#true = self.define_1(true);
                self.store_1(self.visitor.cpu_d(), r#true);
            }
            nes_assembly::Mnemonic::Sei => {
                let r#true = self.define_1(true);
                self.store_1(self.visitor.cpu_i(), r#true);
            }
            nes_assembly::Mnemonic::Sta => {
                let result = self.define_8(self.visitor.cpu_a());
                self.write_operand_u8(result);
            }
            nes_assembly::Mnemonic::Stx => {
                let result = self.define_8(self.visitor.cpu_x());
                self.write_operand_u8(result);
            }
            nes_assembly::Mnemonic::Sty => {
                let result = self.define_8(self.visitor.cpu_y());
                self.write_operand_u8(result);
            }
            nes_assembly::Mnemonic::Tax => {
                let result = self.define_8(self.visitor.cpu_a());
                self.store_8(self.visitor.cpu_x(), result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Tay => {
                let result = self.define_8(self.visitor.cpu_a());
                self.store_8(self.visitor.cpu_y(), result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Tsx => {
                let result = self.define_8(self.visitor.cpu_s());
                self.store_8(self.visitor.cpu_x(), result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Txa => {
                let result = self.define_8(self.visitor.cpu_x());
                self.store_8(self.visitor.cpu_a(), result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Txs => {
                let result = self.define_8(self.visitor.cpu_x());
                self.store_8(self.visitor.cpu_s(), result);
            }
            nes_assembly::Mnemonic::Tya => {
                let result = self.define_8(self.visitor.cpu_y());
                self.store_8(self.visitor.cpu_a(), result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Unimplemented => {
                // unimplemented instructions are treated as a no-op as a
                // fallback
                warn!("compiling unimplemented instruction");
            }
        }

        let jump = jump.unwrap_or(Jump::CpuAddress(
            self.define_16(self.cpu_instruction.address_end()),
        ));
        self.visitor.jump(jump);
    }

    fn define_1(&mut self, definition: impl Into<Definition1>) -> Variable1 {
        self.visitor.define_1(definition)
    }

    fn define_8(&mut self, definition: impl Into<Definition8>) -> Variable8 {
        self.visitor.define_8(definition)
    }

    fn define_16(&mut self, definition: impl Into<Definition16>) -> Variable16 {
        self.visitor.define_16(definition)
    }

    fn read_u16_deref(&mut self, cpu_address: Variable16) -> Variable16 {
        let r#false = self.define_1(false);
        let n1 = self.define_8(1);

        let low_address = cpu_address;

        // intentionally apply page wrapping to the high byte address, matching the
        // behavior of the original hardware
        let high_address_high = self.visitor.high_byte(low_address);
        let high_address_low = self.visitor.low_byte(low_address);
        let high_address_low = self.visitor.add_u8(high_address_low, n1, r#false);
        let high_address = self.define_16(high_address_high % high_address_low);

        let low = cpu_memory::read(&mut self.visitor, low_address);
        let high = cpu_memory::read(&mut self.visitor, high_address);
        self.define_16(high % low)
    }

    fn store_1(&mut self, destination: impl Into<Destination1>, variable: Variable1) {
        match destination.into() {
            Destination1::CpuFlag(CpuFlag::C) => self.visitor.set_cpu_c(variable),
            Destination1::CpuFlag(CpuFlag::Z) => self.visitor.set_cpu_z(variable),
            Destination1::CpuFlag(CpuFlag::I) => self.visitor.set_cpu_i(variable),
            Destination1::CpuFlag(CpuFlag::D) => self.visitor.set_cpu_d(variable),
            Destination1::CpuFlag(CpuFlag::B) => self.visitor.set_cpu_b(variable),
            Destination1::CpuFlag(CpuFlag::Unused) => self.visitor.set_cpu_unused_flag(variable),
            Destination1::CpuFlag(CpuFlag::V) => self.visitor.set_cpu_v(variable),
            Destination1::CpuFlag(CpuFlag::N) => self.visitor.set_cpu_n(variable),
        }
    }

    fn store_8(&mut self, destination: impl Into<Destination8>, variable: Variable8) {
        match destination.into() {
            Destination8::CpuRam(address) => self.visitor.set_cpu_ram(address, variable),
            Destination8::PpuRam(address) => self.visitor.set_ppu_ram(address, variable),
            Destination8::PrgRam(address) => self.visitor.set_prg_ram(address, variable),
            Destination8::CpuRegister(CpuRegister::A) => self.visitor.set_cpu_a(variable),
            Destination8::CpuRegister(CpuRegister::X) => self.visitor.set_cpu_x(variable),
            Destination8::CpuRegister(CpuRegister::Y) => self.visitor.set_cpu_y(variable),
            Destination8::CpuRegister(CpuRegister::S) => self.visitor.set_cpu_s(variable),
            Destination8::CpuRegister(CpuRegister::P) => self.visitor.set_cpu_p(variable),
        }
    }

    fn set_nz(&mut self, value: Variable8) {
        let n = self.visitor.is_negative(value);
        let z = self.visitor.is_zero(value);

        self.store_1(self.visitor.cpu_n(), n);
        self.store_1(self.visitor.cpu_z(), z);
    }

    fn push_u8(&mut self, value: Variable8) {
        let r#false = self.define_1(false);
        let n1 = self.define_8(1);

        let s = self.define_8(self.visitor.cpu_s());
        let s_minus_1 = self.visitor.sub(s, n1, r#false);
        let address = self.define_16(n1 % s);

        cpu_memory::write(&mut self.visitor, address, value);
        self.store_8(self.visitor.cpu_s(), s_minus_1);
    }

    fn push_u16(&mut self, value: Variable16) {
        let low = self.visitor.low_byte(value);
        let high = self.visitor.high_byte(value);

        self.push_u8(high);
        self.push_u8(low);
    }

    fn pop_u8(&mut self) -> Variable8 {
        let r#false = self.define_1(false);
        let n1 = self.define_8(1);

        let s = self.define_8(self.visitor.cpu_s());
        let s_plus_1 = self.visitor.add_u8(s, n1, r#false);
        let result_address = self.define_16(n1 % s_plus_1);
        let result = cpu_memory::read(&mut self.visitor, result_address);

        self.store_8(self.visitor.cpu_s(), s_plus_1);
        result
    }

    fn pop_u16(&mut self) -> Variable16 {
        let low = self.pop_u8();
        let high = self.pop_u8();

        self.define_16(high % low)
    }

    fn get_operand_address(&mut self) -> Variable16 {
        match self.cpu_instruction.operation().addressing_mode() {
            nes_assembly::AddressingMode::Absolute | nes_assembly::AddressingMode::Zeropage => {
                self.define_16(self.cpu_instruction.operand_u16())
            }
            nes_assembly::AddressingMode::AbsoluteX => {
                let n0 = self.define_8(0);

                let x = self.define_8(self.visitor.cpu_x());
                let operand_0 = self.define_16(self.cpu_instruction.operand_u16());
                let operand_1 = self.define_16(n0 % x);
                self.visitor.add_u16(operand_0, operand_1)
            }
            nes_assembly::AddressingMode::AbsoluteY => {
                let n0 = self.define_8(0);

                let y = self.define_8(self.visitor.cpu_y());
                let y_u16 = self.visitor.concatenate(n0, y);
                let operand = self.define_16(self.cpu_instruction.operand_u16());
                self.visitor.add_u16(operand, y_u16)
            }
            nes_assembly::AddressingMode::Accumulator
            | nes_assembly::AddressingMode::Immediate
            | nes_assembly::AddressingMode::Implied
            | nes_assembly::AddressingMode::Indirect
            | nes_assembly::AddressingMode::Relative => unreachable!(),
            nes_assembly::AddressingMode::IndirectY => {
                let n0 = self.define_8(0);

                let operand = self.define_16(self.cpu_instruction.operand_u16());
                let operand_0 = self.read_u16_deref(operand);
                let y = self.define_8(self.visitor.cpu_y());
                let operand_1 = self.define_16(n0 % y);
                self.visitor.add_u16(operand_0, operand_1)
            }
            nes_assembly::AddressingMode::XIndirect => {
                let r#false = self.define_1(false);
                let n0 = self.define_8(0);

                let x = self.define_8(self.visitor.cpu_x());
                let operand = self.define_8(self.cpu_instruction.operand_u8());
                let address = self.visitor.add_u8(operand, x, r#false);
                let address = self.define_16(n0 % address);
                self.read_u16_deref(address)
            }
            nes_assembly::AddressingMode::ZeropageX => {
                let n0 = self.define_8(0);
                let operand = self.define_8(self.cpu_instruction.operand_u8());
                let x = self.define_8(self.visitor.cpu_x());
                let r#false = self.define_1(false);
                let address = self.visitor.add_u8(operand, x, r#false);
                self.define_16(n0 % address)
            }
            nes_assembly::AddressingMode::ZeropageY => {
                let n0 = self.define_8(0);
                let operand = self.define_8(self.cpu_instruction.operand_u8());
                let y = self.define_8(self.visitor.cpu_y());
                let r#false = self.define_1(false);
                let address = self.visitor.add_u8(operand, y, r#false);
                self.define_16(n0 % address)
            }
        }
    }

    fn read_operand_u8(&mut self) -> Variable8 {
        match self.cpu_instruction.operation().addressing_mode() {
            nes_assembly::AddressingMode::Absolute
            | nes_assembly::AddressingMode::Zeropage
            | nes_assembly::AddressingMode::AbsoluteX
            | nes_assembly::AddressingMode::AbsoluteY
            | nes_assembly::AddressingMode::IndirectY
            | nes_assembly::AddressingMode::XIndirect
            | nes_assembly::AddressingMode::ZeropageX
            | nes_assembly::AddressingMode::ZeropageY => {
                let address = self.get_operand_address();
                cpu_memory::read(&mut self.visitor, address)
            }
            nes_assembly::AddressingMode::Accumulator => {
                let a = self.visitor.cpu_a();
                self.define_8(a)
            }
            nes_assembly::AddressingMode::Immediate => {
                self.define_8(self.cpu_instruction.operand_u8())
            }
            nes_assembly::AddressingMode::Implied
            | nes_assembly::AddressingMode::Indirect
            | nes_assembly::AddressingMode::Relative => {
                unreachable!()
            }
        }
    }

    fn write_operand_u8(&mut self, source: Variable8) {
        match self.cpu_instruction.operation().addressing_mode() {
            nes_assembly::AddressingMode::Absolute
            | nes_assembly::AddressingMode::Zeropage
            | nes_assembly::AddressingMode::AbsoluteX
            | nes_assembly::AddressingMode::AbsoluteY
            | nes_assembly::AddressingMode::IndirectY
            | nes_assembly::AddressingMode::XIndirect
            | nes_assembly::AddressingMode::ZeropageX
            | nes_assembly::AddressingMode::ZeropageY => {
                let address = self.get_operand_address();
                cpu_memory::write(&mut self.visitor, address, source);
            }
            nes_assembly::AddressingMode::Accumulator => {
                let a = self.visitor.cpu_a();
                self.store_8(a, source);
            }
            nes_assembly::AddressingMode::Immediate
            | nes_assembly::AddressingMode::Implied
            | nes_assembly::AddressingMode::Indirect
            | nes_assembly::AddressingMode::Relative => {
                unreachable!()
            }
        }
    }

    fn read_operand_u16(&mut self) -> Variable16 {
        match self.cpu_instruction.operation().addressing_mode() {
            nes_assembly::AddressingMode::Absolute => {
                self.define_16(self.cpu_instruction.operand_u16())
            }
            nes_assembly::AddressingMode::Indirect => {
                let address = self.define_16(self.cpu_instruction.operand_u16());
                self.read_u16_deref(address)
            }
            nes_assembly::AddressingMode::Relative => self.define_16(
                self.cpu_instruction
                    .address_end()
                    .wrapping_add_signed(i16::from(self.cpu_instruction.operand_i8())),
            ),
            _ => unreachable!(),
        }
    }
}
