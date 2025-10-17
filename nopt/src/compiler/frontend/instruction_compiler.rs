use crate::{
    compiler::{
        frontend::memory_compiler,
        ir::{
            BasicBlock, CpuFlag, CpuRegister, Definition1, Definition8, Definition16, Destination1,
            Destination8, Function, Instruction, Variable1, Variable8, Variable16,
        },
    },
    nes_assembly,
};

pub(super) fn compile(instruction: nes_assembly::Instruction) -> Function {
    let basic_block = InstructionCompiler {
        cpu_instruction: instruction,
        basic_block: BasicBlock::new(),
    }
    .transpile();
    Function { basic_block }
}

struct InstructionCompiler {
    cpu_instruction: nes_assembly::Instruction,
    basic_block: BasicBlock,
}

impl InstructionCompiler {
    #[expect(clippy::too_many_lines)]
    pub(crate) fn transpile(mut self) -> BasicBlock {
        self.basic_block.jump_target = self.define_16(self.cpu_instruction.address_end());

        match self.cpu_instruction.operation().mnemonic() {
            nes_assembly::Mnemonic::Adc => {
                let operand_0 = self.define_8(CpuRegister::A);
                let operand_1 = self.read_operand_u8();
                let operand_carry = self.define_1(CpuFlag::C);

                let result = self.define_8(Definition8::Sum {
                    operand_0,
                    operand_1,
                    operand_carry,
                });
                let result_carry = self.define_1(Definition1::SumCarry {
                    operand_0,
                    operand_1,
                    operand_carry,
                });
                let result_overflow = self.define_1(Definition1::SumOverflow {
                    operand_0,
                    operand_1,
                    operand_carry,
                });

                self.store_8(CpuRegister::A, result);
                self.store_1(CpuFlag::C, result_carry);
                self.store_1(CpuFlag::V, result_overflow);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::And => {
                let operand_0 = self.define_8(CpuRegister::A);
                let operand_1 = self.read_operand_u8();

                let result = self.define_8(operand_0 & operand_1);

                self.store_8(CpuRegister::A, result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Asl => {
                let operand = self.read_operand_u8();
                let operand_carry = self.define_1(false);

                let result = self.define_8(Definition8::RotateLeft {
                    operand,
                    operand_carry,
                });
                let result_carry = self.define_1(Definition1::U8Bit { operand, index: 7 });

                self.write_operand_u8(result);
                self.store_1(CpuFlag::C, result_carry);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Bcc => {
                let c = self.define_1(CpuFlag::C);
                let not_c = self.define_1(!c);
                let address_if_true = self.read_operand_u16();
                let address_if_false = self.define_16(self.cpu_instruction.address_end());
                let jump_target = self.define_16(Definition16::Select {
                    condition: not_c,
                    result_if_true: address_if_true,
                    result_if_false: address_if_false,
                });

                self.jump(jump_target);
            }
            nes_assembly::Mnemonic::Bcs => {
                let c = self.define_1(CpuFlag::C);
                let address_if_true = self.read_operand_u16();
                let address_if_false = self.define_16(self.cpu_instruction.address_end());

                let jump_target = self.define_16(Definition16::Select {
                    condition: c,
                    result_if_true: address_if_true,
                    result_if_false: address_if_false,
                });

                self.jump(jump_target);
            }
            nes_assembly::Mnemonic::Beq => {
                let z = self.define_1(CpuFlag::Z);
                let address_if_true = self.read_operand_u16();
                let address_if_false = self.define_16(self.cpu_instruction.address_end());
                let jump_target = self.define_16(Definition16::Select {
                    condition: z,
                    result_if_true: address_if_true,
                    result_if_false: address_if_false,
                });

                self.jump(jump_target);
            }
            nes_assembly::Mnemonic::Bit => {
                let operand = self.read_operand_u8();
                let n = self.define_1(Definition1::U8Bit { operand, index: 7 });
                let v = self.define_1(Definition1::U8Bit { operand, index: 6 });
                let a = self.define_8(CpuRegister::A);
                let result = self.define_8(a & operand);
                let z = self.define_1(Definition1::EqualToZero(result));

                self.store_1(CpuFlag::N, n);
                self.store_1(CpuFlag::V, v);
                self.store_1(CpuFlag::Z, z);
            }
            nes_assembly::Mnemonic::Bmi => {
                let n = self.define_1(CpuFlag::N);
                let address_if_true = self.read_operand_u16();
                let address_if_false = self.define_16(self.cpu_instruction.address_end());
                let jump_target = self.define_16(Definition16::Select {
                    condition: n,
                    result_if_true: address_if_true,
                    result_if_false: address_if_false,
                });

                self.jump(jump_target);
            }
            nes_assembly::Mnemonic::Bne => {
                let z = self.define_1(CpuFlag::Z);
                let not_z = self.define_1(!z);
                let address_if_true = self.read_operand_u16();
                let address_if_false = self.define_16(self.cpu_instruction.address_end());
                let jump_target = self.define_16(Definition16::Select {
                    condition: not_z,
                    result_if_true: address_if_true,
                    result_if_false: address_if_false,
                });

                self.jump(jump_target);
            }
            nes_assembly::Mnemonic::Bpl => {
                let n = self.define_1(CpuFlag::N);
                let not_n = self.define_1(!n);
                let address_if_true = self.read_operand_u16();
                let address_if_false = self.define_16(self.cpu_instruction.address_end());
                let jump_target = self.define_16(Definition16::Select {
                    condition: not_n,
                    result_if_true: address_if_true,
                    result_if_false: address_if_false,
                });

                self.jump(jump_target);
            }
            nes_assembly::Mnemonic::Brk => {
                let r#true = self.define_1(true);

                let pc = self.define_16(Definition16::Pc);
                let irq_handler = self.define_16(0xfffe);

                self.store_1(CpuFlag::I, r#true);

                let p = self.define_8(CpuRegister::P);

                self.push_u8(p);
                self.push_u16(pc);
                self.jump(irq_handler);
            }
            nes_assembly::Mnemonic::Bvc => {
                let v = self.define_1(CpuFlag::V);
                let not_v = self.define_1(!v);
                let address_if_true = self.read_operand_u16();
                let address_if_false = self.define_16(self.cpu_instruction.address_end());
                let jump_target = self.define_16(Definition16::Select {
                    condition: not_v,
                    result_if_true: address_if_true,
                    result_if_false: address_if_false,
                });

                self.jump(jump_target);
            }
            nes_assembly::Mnemonic::Bvs => {
                let v = self.define_1(CpuFlag::V);
                let address_if_true = self.read_operand_u16();
                let address_if_false = self.define_16(self.cpu_instruction.address_end());
                let jump_target = self.define_16(Definition16::Select {
                    condition: v,
                    result_if_true: address_if_true,
                    result_if_false: address_if_false,
                });

                self.jump(jump_target);
            }
            nes_assembly::Mnemonic::Clc => {
                let r#false = self.define_1(false);

                self.store_1(CpuFlag::C, r#false);
            }
            nes_assembly::Mnemonic::Cld => {
                let r#false = self.define_1(false);

                self.store_1(CpuFlag::D, r#false);
            }
            nes_assembly::Mnemonic::Cli => {
                let r#false = self.define_1(false);

                self.store_1(CpuFlag::I, r#false);
            }
            nes_assembly::Mnemonic::Clv => {
                let r#false = self.define_1(false);

                self.store_1(CpuFlag::V, r#false);
            }
            nes_assembly::Mnemonic::Cmp => {
                let operand_0 = self.define_8(CpuRegister::A);
                let operand_1 = self.read_operand_u8();
                let operand_borrow = self.define_1(false);

                let result = self.define_8(Definition8::Difference {
                    operand_0,
                    operand_1,
                    operand_borrow,
                });
                let result_borrow = self.define_1(Definition1::DifferenceBorrow {
                    operand_0,
                    operand_1,
                    operand_borrow,
                });
                let result_carry = self.define_1(!result_borrow);

                self.store_1(CpuFlag::C, result_carry);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Cpx => {
                let operand_0 = self.define_8(CpuRegister::X);
                let operand_1 = self.read_operand_u8();
                let operand_borrow = self.define_1(false);

                let result = self.define_8(Definition8::Difference {
                    operand_0,
                    operand_1,
                    operand_borrow,
                });
                let result_borrow = self.define_1(Definition1::DifferenceBorrow {
                    operand_0,
                    operand_1,
                    operand_borrow,
                });
                let result_carry = self.define_1(!result_borrow);

                self.store_1(CpuFlag::C, result_carry);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Cpy => {
                let operand_0 = self.define_8(CpuRegister::Y);
                let operand_1 = self.read_operand_u8();
                let operand_borrow = self.define_1(false);

                let result = self.define_8(Definition8::Difference {
                    operand_0,
                    operand_1,
                    operand_borrow,
                });
                let result_borrow = self.define_1(Definition1::DifferenceBorrow {
                    operand_0,
                    operand_1,
                    operand_borrow,
                });
                let result_carry = self.define_1(!result_borrow);

                self.store_1(CpuFlag::C, result_carry);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Dec => {
                let operand_0 = self.read_operand_u8();
                let operand_1 = self.define_8(1);
                let operand_borrow = self.define_1(false);

                let result = self.define_8(Definition8::Difference {
                    operand_0,
                    operand_1,
                    operand_borrow,
                });

                self.write_operand_u8(result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Dex => {
                let operand_0 = self.define_8(CpuRegister::X);
                let operand_1 = self.define_8(1);
                let operand_borrow = self.define_1(false);

                let result = self.define_8(Definition8::Difference {
                    operand_0,
                    operand_1,
                    operand_borrow,
                });

                self.store_8(CpuRegister::X, result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Dey => {
                let operand_0 = self.define_8(CpuRegister::Y);
                let operand_1 = self.define_8(1);
                let operand_borrow = self.define_1(false);

                let result = self.define_8(Definition8::Difference {
                    operand_0,
                    operand_1,
                    operand_borrow,
                });

                self.store_8(CpuRegister::Y, result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Eor => {
                let operand_0 = self.define_8(CpuRegister::A);
                let operand_1 = self.read_operand_u8();

                let result = self.define_8(operand_0 ^ operand_1);

                self.store_8(CpuRegister::A, result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Inc => {
                let operand_0 = self.read_operand_u8();
                let operand_1 = self.define_8(1);
                let operand_carry = self.define_1(false);

                let result = self.define_8(Definition8::Sum {
                    operand_0,
                    operand_1,
                    operand_carry,
                });

                self.write_operand_u8(result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Inx => {
                let operand_0 = self.define_8(CpuRegister::X);
                let operand_1 = self.define_8(1);
                let operand_carry = self.define_1(false);

                let result = self.define_8(Definition8::Sum {
                    operand_0,
                    operand_1,
                    operand_carry,
                });

                self.store_8(CpuRegister::X, result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Iny => {
                let operand_0 = self.define_8(CpuRegister::Y);
                let operand_1 = self.define_8(1);
                let operand_carry = self.define_1(false);

                let result = self.define_8(Definition8::Sum {
                    operand_0,
                    operand_1,
                    operand_carry,
                });

                self.store_8(CpuRegister::Y, result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Jmp => {
                let address = self.read_operand_u16();

                self.jump(address);
            }
            nes_assembly::Mnemonic::Jsr => {
                let n2 = self.define_16(2);

                let pc = self.define_16(Definition16::Pc);
                let pc_plus_2 = self.define_16(Definition16::Sum {
                    operand_0: pc,
                    operand_1: n2,
                });
                let address = self.read_operand_u16();

                self.push_u16(pc_plus_2);
                self.jump(address);
            }
            nes_assembly::Mnemonic::Lda => {
                let result = self.read_operand_u8();

                self.store_8(CpuRegister::A, result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Ldx => {
                let result = self.read_operand_u8();

                self.store_8(CpuRegister::X, result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Ldy => {
                let result = self.read_operand_u8();

                self.store_8(CpuRegister::Y, result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Lsr => {
                let operand = self.read_operand_u8();
                let operand_carry = self.define_1(false);

                let result = self.define_8(Definition8::RotateRight {
                    operand,
                    operand_carry,
                });

                let result_carry = self.define_1(Definition1::U8Bit { operand, index: 0 });

                self.write_operand_u8(result);
                self.store_1(CpuFlag::C, result_carry);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Nop => {}
            nes_assembly::Mnemonic::Ora => {
                let operand_0 = self.define_8(CpuRegister::A);
                let operand_1 = self.read_operand_u8();

                let result = self.define_8(operand_0 | operand_1);

                self.store_8(CpuRegister::A, result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Pha => {
                let value = self.define_8(CpuRegister::A);

                self.push_u8(value);
            }
            nes_assembly::Mnemonic::Php => {
                let set_flags_mask =
                    self.define_8((1 << CpuFlag::Unused.index()) | (1 << CpuFlag::B.index()));

                let value = self.define_8(CpuRegister::P);
                let value = self.define_8(value | set_flags_mask);

                self.push_u8(value);
            }
            nes_assembly::Mnemonic::Pla => {
                let value = self.pop_u8();

                self.store_8(CpuRegister::A, value);
                self.set_nz(value);
            }
            nes_assembly::Mnemonic::Plp => {
                let b = self.define_1(CpuFlag::B);
                let unused_flag = self.define_1(CpuFlag::Unused);
                let value = self.pop_u8();

                self.store_8(CpuRegister::P, value);
                self.store_1(CpuFlag::B, b);
                self.store_1(CpuFlag::Unused, unused_flag);
            }
            nes_assembly::Mnemonic::Rol => {
                let operand = self.read_operand_u8();
                let operand_carry = self.define_1(CpuFlag::C);

                let result = self.define_8(Definition8::RotateLeft {
                    operand,
                    operand_carry,
                });

                let result_carry = self.define_1(Definition1::U8Bit { operand, index: 7 });

                self.write_operand_u8(result);
                self.store_1(CpuFlag::C, result_carry);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Ror => {
                let operand = self.read_operand_u8();
                let operand_carry = self.define_1(CpuFlag::C);

                let result = self.define_8(Definition8::RotateRight {
                    operand,
                    operand_carry,
                });

                let result_carry = self.define_1(Definition1::U8Bit { operand, index: 0 });

                self.write_operand_u8(result);
                self.store_1(CpuFlag::C, result_carry);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Rti => {
                let unused_flag = self.define_1(CpuFlag::Unused);
                let p = self.pop_u8();
                let return_address = self.pop_u16();

                self.store_8(CpuRegister::P, p);
                self.store_1(CpuFlag::Unused, unused_flag);
                self.jump(return_address);
            }
            nes_assembly::Mnemonic::Rts => {
                let n1 = self.define_16(1);

                let return_address_minus_1 = self.pop_u16();
                let return_address = self.define_16(Definition16::Sum {
                    operand_0: return_address_minus_1,
                    operand_1: n1,
                });

                self.jump(return_address);
            }
            nes_assembly::Mnemonic::Sbc => {
                let operand_0 = self.define_8(CpuRegister::A);
                let operand_1 = self.read_operand_u8();
                let operand_carry = self.define_1(CpuFlag::C);
                let operand_borrow = self.define_1(!operand_carry);

                let result = self.define_8(Definition8::Difference {
                    operand_0,
                    operand_1,
                    operand_borrow,
                });
                let result_borrow = self.define_1(Definition1::DifferenceBorrow {
                    operand_0,
                    operand_1,
                    operand_borrow,
                });
                let result_carry = self.define_1(!result_borrow);
                let result_overflow = self.define_1(Definition1::DifferenceOverflow {
                    operand_0,
                    operand_1,
                    operand_borrow,
                });

                self.store_8(CpuRegister::A, result);
                self.store_1(CpuFlag::C, result_carry);
                self.store_1(CpuFlag::V, result_overflow);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Sec => {
                let r#true = self.define_1(true);
                self.store_1(CpuFlag::C, r#true);
            }
            nes_assembly::Mnemonic::Sed => {
                let r#true = self.define_1(true);
                self.store_1(CpuFlag::D, r#true);
            }
            nes_assembly::Mnemonic::Sei => {
                let r#true = self.define_1(true);
                self.store_1(CpuFlag::I, r#true);
            }
            nes_assembly::Mnemonic::Sta => {
                let result = self.define_8(CpuRegister::A);
                self.write_operand_u8(result);
            }
            nes_assembly::Mnemonic::Stx => {
                let result = self.define_8(CpuRegister::X);
                self.write_operand_u8(result);
            }
            nes_assembly::Mnemonic::Sty => {
                let result = self.define_8(CpuRegister::Y);
                self.write_operand_u8(result);
            }
            nes_assembly::Mnemonic::Tax => {
                let result = self.define_8(CpuRegister::A);
                self.store_8(CpuRegister::X, result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Tay => {
                let result = self.define_8(CpuRegister::A);
                self.store_8(CpuRegister::Y, result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Tsx => {
                let result = self.define_8(CpuRegister::S);
                self.store_8(CpuRegister::X, result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Txa => {
                let result = self.define_8(CpuRegister::X);
                self.store_8(CpuRegister::A, result);
                self.set_nz(result);
            }
            nes_assembly::Mnemonic::Txs => {
                let result = self.define_8(CpuRegister::X);
                self.store_8(CpuRegister::S, result);
            }
            nes_assembly::Mnemonic::Tya => {
                let result = self.define_8(CpuRegister::Y);
                self.store_8(CpuRegister::A, result);
                self.set_nz(result);
            }
        }

        self.basic_block
    }

    fn define_1(&mut self, definition: impl Into<Definition1>) -> Variable1 {
        self.basic_block.define_1(definition.into())
    }

    fn define_8(&mut self, definition: impl Into<Definition8>) -> Variable8 {
        self.basic_block.define_8(definition.into())
    }

    fn define_16(&mut self, definition: impl Into<Definition16>) -> Variable16 {
        self.basic_block.define_16(definition.into())
    }

    fn read_u16_deref(&mut self, cpu_address: Variable16) -> Variable16 {
        let r#false = self.define_1(false);
        let n1 = self.define_8(1);

        let low_address = cpu_address;

        // intentionally apply page wrapping to the high byte address, matching the
        // behavior of the original hardware
        let high_address_high = self.define_8(Definition8::HighByte(low_address));
        let high_address_low = self.define_8(Definition8::LowByte(low_address));
        let high_address_low = self.define_8(Definition8::Sum {
            operand_0: high_address_low,
            operand_1: n1,
            operand_carry: r#false,
        });
        let high_address = self.define_16(high_address_high % high_address_low);

        let low = memory_compiler::compile_read(&mut self.basic_block, low_address);
        let high = memory_compiler::compile_read(&mut self.basic_block, high_address);
        self.define_16(high % low)
    }

    fn store_1(&mut self, destination: impl Into<Destination1>, register: Variable1) {
        self.basic_block.instructions.push(Instruction::Store1 {
            variable: register,
            destination: destination.into(),
        });
    }

    fn store_8(&mut self, destination: impl Into<Destination8>, register: Variable8) {
        self.basic_block.instructions.push(Instruction::Store8 {
            variable: register,
            destination: destination.into(),
        });
    }

    fn set_nz(&mut self, value: Variable8) {
        let n = self.define_1(Definition1::Negative(value));
        let z = self.define_1(Definition1::EqualToZero(value));

        self.store_1(CpuFlag::N, n);
        self.store_1(CpuFlag::Z, z);
    }

    fn jump(&mut self, target: Variable16) {
        self.basic_block.jump_target = target;
    }

    fn push_u8(&mut self, value: Variable8) {
        let r#false = self.define_1(false);
        let n1 = self.define_8(1);

        let s = self.define_8(CpuRegister::S);
        let s_minus_1 = self.define_8(Definition8::Difference {
            operand_0: s,
            operand_1: n1,
            operand_borrow: r#false,
        });
        let address = self.define_16(n1 % s);

        memory_compiler::compile_write(&mut self.basic_block, address, value);
        self.store_8(CpuRegister::S, s_minus_1);
    }

    fn push_u16(&mut self, value: Variable16) {
        let low = self.define_8(Definition8::LowByte(value));
        let high = self.define_8(Definition8::HighByte(value));

        self.push_u8(high);
        self.push_u8(low);
    }

    fn pop_u8(&mut self) -> Variable8 {
        let r#false = self.define_1(false);
        let n1 = self.define_8(1);

        let s = self.define_8(CpuRegister::S);
        let s_plus_1 = self.define_8(Definition8::Sum {
            operand_0: s,
            operand_1: n1,
            operand_carry: r#false,
        });
        let result_address = self.define_16(n1 % s_plus_1);
        let result = memory_compiler::compile_read(&mut self.basic_block, result_address);

        self.store_8(CpuRegister::S, s_plus_1);
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

                let x = self.define_8(CpuRegister::X);
                let operand_0 = self.define_16(self.cpu_instruction.operand_u16());
                let operand_1 = self.define_16(n0 % x);
                self.define_16(Definition16::Sum {
                    operand_0,
                    operand_1,
                })
            }
            nes_assembly::AddressingMode::AbsoluteY => {
                let n0 = self.define_8(0);

                let y = self.define_8(CpuRegister::Y);
                let y_u16 = self.define_16(Definition16::FromU8s { low: y, high: n0 });
                let operand = self.define_16(self.cpu_instruction.operand_u16());
                self.define_16(Definition16::Sum {
                    operand_0: operand,
                    operand_1: y_u16,
                })
            }
            nes_assembly::AddressingMode::Accumulator
            | nes_assembly::AddressingMode::Immediate
            | nes_assembly::AddressingMode::Implied
            | nes_assembly::AddressingMode::Indirect
            | nes_assembly::AddressingMode::Relative => unreachable!(),
            nes_assembly::AddressingMode::IndirectY => {
                let n0 = self.define_8(0);

                let operand =
                    self.define_16(Definition16::Immediate(self.cpu_instruction.operand_u16()));
                let operand_0 = self.read_u16_deref(operand);
                let y = self.define_8(CpuRegister::Y);
                let operand_1 = self.define_16(n0 % y);
                self.define_16(Definition16::Sum {
                    operand_0,
                    operand_1,
                })
            }
            nes_assembly::AddressingMode::XIndirect => {
                let r#false = self.define_1(false);
                let n0 = self.define_8(0);

                let x = self.define_8(CpuRegister::X);
                let operand = self.define_8(self.cpu_instruction.operand_u8());
                let address = self.define_8(Definition8::Sum {
                    operand_0: operand,
                    operand_1: x,
                    operand_carry: r#false,
                });
                let address = self.define_16(n0 % address);
                self.read_u16_deref(address)
            }
            nes_assembly::AddressingMode::ZeropageX => {
                let n0 = self.define_8(0);
                let operand = self.define_8(self.cpu_instruction.operand_u8());
                let x = self.define_8(CpuRegister::X);
                let r#false = self.define_1(false);
                let address = self.define_8(Definition8::Sum {
                    operand_0: operand,
                    operand_1: x,
                    operand_carry: r#false,
                });
                self.define_16(n0 % address)
            }
            nes_assembly::AddressingMode::ZeropageY => {
                let n0 = self.define_8(0);
                let operand = self.define_8(self.cpu_instruction.operand_u8());
                let y = self.define_8(CpuRegister::Y);
                let r#false = self.define_1(false);
                let address = self.define_8(Definition8::Sum {
                    operand_0: operand,
                    operand_1: y,
                    operand_carry: r#false,
                });
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
                memory_compiler::compile_read(&mut self.basic_block, address)
            }
            nes_assembly::AddressingMode::Accumulator => self.define_8(CpuRegister::A),
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
                memory_compiler::compile_write(&mut self.basic_block, address, source);
            }
            nes_assembly::AddressingMode::Accumulator => {
                self.store_8(CpuRegister::A, source);
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
            nes_assembly::AddressingMode::Relative => self.define_16(Definition16::Immediate(
                self.cpu_instruction
                    .address_end()
                    .wrapping_add_signed(i16::from(self.cpu_instruction.operand_i8())),
            )),
            _ => unreachable!(),
        }
    }
}
