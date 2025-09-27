use crate::{Nes, compiler::ir};
use cranelift_codegen::{
    Context,
    control::ControlPlane,
    ir::{Function, InstBuilder, MemFlags, Type, Value, condcodes::IntCC},
    settings::{self, Configurable as _},
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use memmap2::Mmap;
use std::collections::HashMap;
use target_lexicon::Triple;

pub(super) fn compile(ir: ir::Function, nes: *mut Nes, optimize: bool) -> Mmap {
    CraneliftBackend::new(optimize).compile(ir, nes)
}

struct CraneliftBackend {
    optimize: bool,
    variable_1_mapping: HashMap<usize, Value>,
    variable_8_mapping: HashMap<usize, Value>,
    variable_16_mapping: HashMap<usize, Value>,
}

impl CraneliftBackend {
    pub(crate) fn new(optimize: bool) -> Self {
        Self {
            optimize,
            variable_1_mapping: HashMap::new(),
            variable_8_mapping: HashMap::new(),
            variable_16_mapping: HashMap::new(),
        }
    }

    #[expect(clippy::too_many_lines)]
    pub(crate) fn compile(mut self, ir: ir::Function, nes: *mut Nes) -> Mmap {
        let type_u8 = Type::int(8).unwrap();
        let type_u16 = Type::int(16).unwrap();

        let mut function = Function::new();
        let mut function_builder_context = FunctionBuilderContext::new();
        let mut function_builder =
            FunctionBuilder::new(&mut function, &mut function_builder_context);

        let block = function_builder.create_block();
        function_builder.seal_block(block);
        function_builder.switch_to_block(block);

        let mut flags_builder = settings::builder();
        flags_builder
            .set("opt_level", if self.optimize { "speed" } else { "none" })
            .unwrap();

        let isa_builder = cranelift_codegen::isa::lookup(Triple::host()).unwrap();
        let isa = isa_builder
            .finish(settings::Flags::new(flags_builder))
            .unwrap();

        let nes_cpu_ram_address = function_builder.ins().iconst(isa.pointer_type(), unsafe {
            (*nes).cpu.ram.as_ptr() as i64
        });
        let nes_cpu_prg_rom_address = function_builder.ins().iconst(isa.pointer_type(), unsafe {
            (*nes).rom.prg_rom().as_ptr() as i64
        });
        let nes_cpu_a_address = function_builder
            .ins()
            .iconst(isa.pointer_type(), unsafe { &raw mut (*nes).cpu.a as i64 });
        let nes_cpu_x_address = function_builder
            .ins()
            .iconst(isa.pointer_type(), unsafe { &raw mut (*nes).cpu.x as i64 });
        let nes_cpu_y_address = function_builder
            .ins()
            .iconst(isa.pointer_type(), unsafe { &raw mut (*nes).cpu.y as i64 });
        let nes_cpu_s_address = function_builder
            .ins()
            .iconst(isa.pointer_type(), unsafe { &raw mut (*nes).cpu.s as i64 });
        let nes_cpu_p_address = function_builder
            .ins()
            .iconst(isa.pointer_type(), unsafe { &raw mut (*nes).cpu.p as i64 });
        let nes_cpu_pc_address = function_builder
            .ins()
            .iconst(isa.pointer_type(), unsafe { &raw mut (*nes).cpu.pc as i64 });

        for instruction in ir.basic_block.instructions {
            match instruction {
                ir::Instruction::Define1 {
                    variable,
                    definition,
                } => {
                    let value = match definition {
                        ir::Definition1::Immediate(immediate) => {
                            function_builder.ins().iconst(type_u8, i64::from(immediate))
                        }
                        ir::Definition1::CpuFlag(cpu_flag) => {
                            let value = function_builder.ins().load(
                                type_u8,
                                MemFlags::new(),
                                nes_cpu_p_address,
                                0,
                            );
                            let value = function_builder
                                .ins()
                                .ushr_imm(value, i64::from(cpu_flag.index()));
                            function_builder.ins().band_imm(value, 0b1)
                        }
                        ir::Definition1::Not(variable) => {
                            function_builder.ins().bxor_imm(self.value_1(variable), 1)
                        }
                        ir::Definition1::EqualToZero(variable) => {
                            function_builder
                                .ins()
                                .icmp_imm(IntCC::Equal, self.value_8(variable), 0)
                        }
                        ir::Definition1::Negative(variable) => function_builder.ins().icmp_imm(
                            IntCC::UnsignedGreaterThanOrEqual,
                            self.value_8(variable),
                            0x80,
                        ),
                        ir::Definition1::U8Bit { operand, index } => {
                            let value = function_builder
                                .ins()
                                .ushr_imm(self.value_8(operand), i64::from(index));
                            function_builder.ins().band_imm(value, 0b1)
                        }
                        ir::Definition1::LessThan16(operand_0, operand_1) => {
                            function_builder.ins().icmp(
                                IntCC::UnsignedLessThan,
                                self.value_16(operand_0),
                                self.value_16(operand_1),
                            )
                        }
                        ir::Definition1::SumCarry {
                            operand_0,
                            operand_1,
                            operand_carry,
                        } => {
                            let (result, carry_0) = function_builder
                                .ins()
                                .uadd_overflow(self.value_8(operand_0), self.value_8(operand_1));
                            let (_, carry_1) = function_builder
                                .ins()
                                .uadd_overflow(result, self.value_1(operand_carry));
                            function_builder.ins().bor(carry_0, carry_1)
                        }
                        ir::Definition1::SumOverflow {
                            operand_0,
                            operand_1,
                            operand_carry,
                        } => {
                            let sum = function_builder
                                .ins()
                                .uadd_overflow(self.value_8(operand_0), self.value_8(operand_1))
                                .0;
                            let sum = function_builder
                                .ins()
                                .uadd_overflow(sum, self.value_1(operand_carry))
                                .0;

                            let operand_0_xor_sum =
                                function_builder.ins().bxor(self.value_8(operand_0), sum);
                            let operand_1_xor_sum =
                                function_builder.ins().bxor(self.value_8(operand_1), sum);
                            let overflow = function_builder
                                .ins()
                                .band(operand_0_xor_sum, operand_1_xor_sum);
                            function_builder.ins().ushr_imm(overflow, 7)
                        }
                        ir::Definition1::DifferenceBorrow {
                            operand_0,
                            operand_1,
                            operand_borrow,
                        } => {
                            let (result, borrow_0) = function_builder
                                .ins()
                                .usub_overflow(self.value_8(operand_0), self.value_8(operand_1));
                            let (_, borrow_1) = function_builder
                                .ins()
                                .usub_overflow(result, self.value_1(operand_borrow));
                            function_builder.ins().bor(borrow_0, borrow_1)
                        }
                        ir::Definition1::DifferenceOverflow {
                            operand_0,
                            operand_1,
                            operand_borrow,
                        } => {
                            let sum = function_builder
                                .ins()
                                .usub_overflow(self.value_8(operand_0), self.value_8(operand_1))
                                .0;
                            let sum = function_builder
                                .ins()
                                .usub_overflow(sum, self.value_1(operand_borrow))
                                .0;

                            let operand_0_xor_sum =
                                function_builder.ins().bxor(self.value_8(operand_0), sum);
                            let operand_1_xor_sum =
                                function_builder.ins().bxor(self.value_8(operand_1), sum);
                            let operand_1_xnor_sum = function_builder.ins().bnot(operand_1_xor_sum);
                            let overflow = function_builder
                                .ins()
                                .band(operand_0_xor_sum, operand_1_xnor_sum);
                            function_builder.ins().ushr_imm(overflow, 7)
                        }
                    };
                    debug_assert_eq!(function_builder.func.dfg.value_type(value), type_u8);
                    self.variable_1_mapping.insert(variable.id, value);
                }
                ir::Instruction::Define8 {
                    variable,
                    definition,
                } => {
                    let value = match definition {
                        ir::Definition8::Immediate(immediate) => {
                            function_builder.ins().iconst(type_u8, i64::from(immediate))
                        }
                        ir::Definition8::CpuRegister(cpu_register) => {
                            let address = match cpu_register {
                                ir::CpuRegister::A => nes_cpu_a_address,
                                ir::CpuRegister::X => nes_cpu_x_address,
                                ir::CpuRegister::Y => nes_cpu_y_address,
                                ir::CpuRegister::S => nes_cpu_s_address,
                                ir::CpuRegister::P => nes_cpu_p_address,
                            };
                            function_builder
                                .ins()
                                .load(type_u8, MemFlags::new(), address, 0)
                        }
                        ir::Definition8::Ram(variable) => {
                            let cpu_address = function_builder
                                .ins()
                                .uextend(isa.pointer_type(), self.value_16(variable));
                            let index = function_builder.ins().band_imm(cpu_address, 0x7ff);
                            let address = function_builder
                                .ins()
                                .uadd_overflow(nes_cpu_ram_address, index)
                                .0;
                            function_builder
                                .ins()
                                .load(type_u8, MemFlags::new(), address, 0)
                        }
                        ir::Definition8::Rom(variable) => {
                            let cpu_address = function_builder
                                .ins()
                                .uextend(isa.pointer_type(), self.value_16(variable));
                            let index = function_builder.ins().band_imm(cpu_address, unsafe {
                                i64::try_from((*nes).rom.prg_rom().len() - 1).unwrap()
                            });
                            let address = function_builder
                                .ins()
                                .uadd_overflow(nes_cpu_prg_rom_address, index)
                                .0;
                            function_builder
                                .ins()
                                .load(type_u8, MemFlags::new(), address, 0)
                        }
                        ir::Definition8::LowByte(variable) => function_builder
                            .ins()
                            .ireduce(type_u8, self.value_16(variable)),
                        ir::Definition8::HighByte(variable) => {
                            let high = function_builder.ins().ushr_imm(self.value_16(variable), 8);
                            function_builder.ins().ireduce(type_u8, high)
                        }
                        ir::Definition8::Or(variable_0, variable_1) => function_builder
                            .ins()
                            .bor(self.value_8(variable_0), self.value_8(variable_1)),
                        ir::Definition8::And(variable_0, variable_1) => function_builder
                            .ins()
                            .band(self.value_8(variable_0), self.value_8(variable_1)),
                        ir::Definition8::Xor(variable_0, variable_1) => function_builder
                            .ins()
                            .bxor(self.value_8(variable_0), self.value_8(variable_1)),
                        ir::Definition8::RotateLeft {
                            operand,
                            operand_carry,
                        } => {
                            let result = function_builder.ins().ishl_imm(self.value_8(operand), 1);
                            function_builder
                                .ins()
                                .bor(result, self.value_1(operand_carry))
                        }
                        ir::Definition8::RotateRight {
                            operand,
                            operand_carry,
                        } => {
                            let operand_carry = function_builder
                                .ins()
                                .uextend(type_u16, self.value_1(operand_carry));
                            let operand_carry = function_builder.ins().ishl_imm(operand_carry, 8);
                            let operand = function_builder
                                .ins()
                                .uextend(type_u16, self.value_8(operand));
                            let operand = function_builder.ins().bor(operand_carry, operand);
                            let result = function_builder.ins().sshr_imm(operand, 1);
                            function_builder.ins().ireduce(type_u8, result)
                        }
                        ir::Definition8::Sum {
                            operand_0,
                            operand_1,
                            operand_carry,
                        } => {
                            let sum = function_builder
                                .ins()
                                .uadd_overflow(self.value_8(operand_0), self.value_8(operand_1))
                                .0;
                            function_builder
                                .ins()
                                .uadd_overflow(sum, self.value_1(operand_carry))
                                .0
                        }
                        ir::Definition8::Difference {
                            operand_0,
                            operand_1,
                            operand_borrow,
                        } => {
                            let result = function_builder
                                .ins()
                                .usub_overflow(self.value_8(operand_0), self.value_8(operand_1))
                                .0;
                            function_builder
                                .ins()
                                .usub_overflow(result, self.value_1(operand_borrow))
                                .0
                        }
                        ir::Definition8::Select {
                            condition,
                            result_if_true,
                            result_if_false,
                        } => function_builder.ins().select(
                            self.value_1(condition),
                            self.value_8(result_if_true),
                            self.value_8(result_if_false),
                        ),
                    };
                    debug_assert_eq!(function_builder.func.dfg.value_type(value), type_u8);
                    self.variable_8_mapping.insert(variable.id, value);
                }
                ir::Instruction::Define16 {
                    variable,
                    definition,
                } => {
                    let value = match definition {
                        ir::Definition16::Immediate(immediate) => function_builder
                            .ins()
                            .iconst(type_u16, i64::from(immediate)),
                        ir::Definition16::Pc => function_builder.ins().load(
                            type_u16,
                            MemFlags::new(),
                            nes_cpu_pc_address,
                            0,
                        ),
                        ir::Definition16::FromU8s { high, low } => {
                            let high = function_builder.ins().uextend(type_u16, self.value_8(high));
                            let high = function_builder.ins().ishl_imm(high, 8);
                            let low = function_builder.ins().uextend(type_u16, self.value_8(low));
                            function_builder.ins().bor(high, low)
                        }
                        ir::Definition16::Sum {
                            operand_0,
                            operand_1,
                        } => {
                            function_builder
                                .ins()
                                .uadd_overflow(self.value_16(operand_0), self.value_16(operand_1))
                                .0
                        }
                        ir::Definition16::Select {
                            condition,
                            result_if_true,
                            result_if_false,
                        } => function_builder.ins().select(
                            self.value_1(condition),
                            self.value_16(result_if_true),
                            self.value_16(result_if_false),
                        ),
                    };
                    debug_assert_eq!(function_builder.func.dfg.value_type(value), type_u16);
                    self.variable_16_mapping.insert(variable.id, value);
                }
                ir::Instruction::Store1 {
                    destination,
                    variable,
                } => match destination {
                    ir::Destination1::CpuFlag(cpu_flag) => {
                        let flag = function_builder
                            .ins()
                            .ishl_imm(self.value_1(variable), i64::from(cpu_flag.index()));
                        let p = function_builder.ins().load(
                            type_u8,
                            MemFlags::new(),
                            nes_cpu_p_address,
                            0,
                        );
                        let p = function_builder.ins().band_imm(p, !(1 << cpu_flag.index()));
                        let p = function_builder.ins().bor(p, flag);
                        function_builder
                            .ins()
                            .store(MemFlags::new(), p, nes_cpu_p_address, 0);
                    }
                },
                ir::Instruction::Store8 {
                    destination,
                    variable,
                } => match destination {
                    ir::Destination8::CpuRegister(cpu_register) => {
                        let address = match cpu_register {
                            ir::CpuRegister::A => nes_cpu_a_address,
                            ir::CpuRegister::X => nes_cpu_x_address,
                            ir::CpuRegister::Y => nes_cpu_y_address,
                            ir::CpuRegister::S => nes_cpu_s_address,
                            ir::CpuRegister::P => nes_cpu_p_address,
                        };
                        function_builder.ins().store(
                            MemFlags::new(),
                            self.value_8(variable),
                            address,
                            0,
                        );
                    }
                    ir::Destination8::Ram(address) => {
                        let address = function_builder
                            .ins()
                            .uextend(isa.pointer_type(), self.value_16(address));
                        let index = function_builder.ins().band_imm(address, 0x7ff);
                        let address = function_builder.ins().iadd(nes_cpu_ram_address, index);
                        function_builder.ins().store(
                            MemFlags::new(),
                            self.value_8(variable),
                            address,
                            0,
                        );
                    }
                },
            }
        }

        function_builder.ins().store(
            MemFlags::new(),
            self.value_16(ir.basic_block.jump_target),
            nes_cpu_pc_address,
            0,
        );
        function_builder.ins().return_(&[]);

        function_builder.finalize();

        let mut context = Context::for_function(function);
        let buffer = &context
            .compile(&*isa, &mut ControlPlane::default())
            .unwrap()
            .buffer;

        let mut allocated_buffer = memmap2::MmapMut::map_anon(buffer.data().len()).unwrap();

        unsafe {
            allocated_buffer
                .as_mut_ptr()
                .copy_from(buffer.data().as_ptr(), buffer.data().len());
        }
        allocated_buffer.make_exec().unwrap()
    }

    fn value_1(&mut self, variable: ir::Variable1) -> Value {
        *self.variable_1_mapping.get(&variable.id).unwrap()
    }

    fn value_8(&mut self, variable: ir::Variable8) -> Value {
        *self.variable_8_mapping.get(&variable.id).unwrap()
    }

    fn value_16(&mut self, variable: ir::Variable16) -> Value {
        *self.variable_16_mapping.get(&variable.id).unwrap()
    }
}
