use std::{
    fmt::Debug,
    ops::{BitAnd, BitOr, BitXor, Not, Rem},
};

pub(super) struct BasicBlock {
    pub instructions: Vec<Instruction>,
    pub jump_target: Variable16,
}

impl BasicBlock {
    pub(super) fn new() -> Self {
        Self {
            instructions: vec![],
            jump_target: Variable16 { id: usize::MAX }, // TODO: don't use dummy value for id
        }
    }

    pub(super) fn define_1(&mut self, definition: Definition1) -> Variable1 {
        let variable = Variable1 {
            id: self.instructions.len(),
        };
        self.instructions.push(Instruction::Define1 {
            variable,
            definition,
        });
        variable
    }

    pub(super) fn define_8(&mut self, definition: Definition8) -> Variable8 {
        let variable = Variable8 {
            id: self.instructions.len(),
        };
        self.instructions.push(Instruction::Define8 {
            variable,
            definition,
        });
        variable
    }

    pub(super) fn define_16(&mut self, definition: Definition16) -> Variable16 {
        let variable = Variable16 {
            id: self.instructions.len(),
        };
        self.instructions.push(Instruction::Define16 {
            variable,
            definition,
        });
        variable
    }
}

pub(super) enum Instruction {
    Define1 {
        variable: Variable1,
        definition: Definition1,
    },
    Define8 {
        variable: Variable8,
        definition: Definition8,
    },
    Define16 {
        variable: Variable16,
        definition: Definition16,
    },
    Store1 {
        destination: Destination1,
        variable: Variable1,
    },
    Store8 {
        destination: Destination8,
        variable: Variable8,
    },
}

#[derive(Clone)]
pub(super) enum CpuFlag {
    C,
    Z,
    I,
    D,
    B,
    Unused,
    V,
    N,
}

impl Debug for CpuFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::C => write!(f, "c"),
            Self::Z => write!(f, "z"),
            Self::I => write!(f, "i"),
            Self::D => write!(f, "d"),
            Self::B => write!(f, "b"),
            Self::Unused => write!(f, "unused_flag"),
            Self::V => write!(f, "v"),
            Self::N => write!(f, "n"),
        }
    }
}

impl CpuFlag {
    pub(super) fn index(&self) -> u8 {
        match self {
            CpuFlag::C => 0,
            CpuFlag::Z => 1,
            CpuFlag::I => 2,
            CpuFlag::D => 3,
            CpuFlag::B => 4,
            CpuFlag::Unused => 5,
            CpuFlag::V => 6,
            CpuFlag::N => 7,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Variable1 {
    pub id: usize,
}

impl Debug for Variable1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "var_{}", self.id)
    }
}

pub(super) enum Definition1 {
    Immediate(bool),
    CpuFlag(CpuFlag),
    Not(Variable1),
    EqualToZero(Variable8),
    Negative(Variable8),
    U8Bit {
        operand: Variable8,
        index: u8,
    },
    LessThan16(Variable16, Variable16),
    SumCarry {
        operand_0: Variable8,
        operand_1: Variable8,
        operand_carry: Variable1,
    },
    SumOverflow {
        operand_0: Variable8,
        operand_1: Variable8,
        operand_carry: Variable1,
    },
    DifferenceBorrow {
        operand_0: Variable8,
        operand_1: Variable8,
        operand_borrow: Variable1,
    },
    DifferenceOverflow {
        operand_0: Variable8,
        operand_1: Variable8,
        operand_borrow: Variable1,
    },
}

impl From<bool> for Definition1 {
    fn from(value: bool) -> Self {
        Self::Immediate(value)
    }
}

impl From<CpuFlag> for Definition1 {
    fn from(value: CpuFlag) -> Self {
        Self::CpuFlag(value)
    }
}

impl Not for Variable1 {
    type Output = Definition1;

    fn not(self) -> Self::Output {
        Self::Output::Not(self)
    }
}

impl Debug for Definition1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Immediate(immediate) => write!(f, "{}", u8::from(*immediate)),
            Self::CpuFlag(flag) => write!(f, "{flag:?}"),
            Self::Not(register_u1) => write!(f, "!{register_u1:?}"),
            Self::EqualToZero(u8) => write!(f, "({u8:?} == 0)"),
            Self::Negative(u8) => write!(f, "({u8:?} >= 0x80)"),
            Self::U8Bit { operand, index } => write!(f, "{operand:?}.bit({index})"),
            Self::SumCarry {
                operand_0,
                operand_1,
                operand_carry,
            } => write!(
                f,
                "({operand_0:?} + {operand_1:?} + {operand_carry:?}).carry"
            ),
            Self::SumOverflow {
                operand_0,
                operand_1,
                operand_carry,
            } => write!(
                f,
                "({operand_0:?} + {operand_1:?} + {operand_carry:?}).overflow"
            ),
            Self::DifferenceBorrow {
                operand_0,
                operand_1,
                operand_borrow,
            } => write!(
                f,
                "({operand_0:?} - {operand_1:?} - {operand_borrow:?}).borrow"
            ),
            Self::DifferenceOverflow {
                operand_0,
                operand_1,
                operand_borrow,
            } => write!(
                f,
                "({operand_0:?} - {operand_1:?} - {operand_borrow:?}).overflow"
            ),
            Self::LessThan16(operand_0, operand_1) => write!(f, "({operand_0:?} < {operand_1:?})"),
        }
    }
}

#[derive(Clone)]
pub(super) enum Destination1 {
    CpuFlag(CpuFlag),
}

impl From<CpuFlag> for Destination1 {
    fn from(value: CpuFlag) -> Self {
        Self::CpuFlag(value)
    }
}

impl Debug for Destination1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CpuFlag(flag) => write!(f, "{flag:?}"),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Variable8 {
    pub id: usize,
}

impl Debug for Variable8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "var_{}", self.id)
    }
}

#[derive(Clone, Copy)]
pub(super) enum CpuRegister {
    A,
    X,
    Y,
    S,
    P,
}

impl Debug for CpuRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A => write!(f, "a"),
            Self::X => write!(f, "x"),
            Self::Y => write!(f, "y"),
            Self::S => write!(f, "s"),
            Self::P => write!(f, "p"),
        }
    }
}

#[derive(Clone)]
pub(super) enum Definition8 {
    Immediate(u8),
    CpuRegister(CpuRegister),
    Ram(Variable16),
    Rom(Variable16),
    LowByte(Variable16),
    HighByte(Variable16),
    Or(Variable8, Variable8),
    And(Variable8, Variable8),
    Xor(Variable8, Variable8),
    RotateLeft {
        operand: Variable8,
        operand_carry: Variable1,
    },
    RotateRight {
        operand: Variable8,
        operand_carry: Variable1,
    },
    Sum {
        operand_0: Variable8,
        operand_1: Variable8,
        operand_carry: Variable1,
    },
    Difference {
        operand_0: Variable8,
        operand_1: Variable8,
        operand_borrow: Variable1,
    },
    Select {
        condition: Variable1,
        result_if_true: Variable8,
        result_if_false: Variable8,
    },
}

impl From<u8> for Definition8 {
    fn from(value: u8) -> Self {
        Self::Immediate(value)
    }
}

impl From<CpuRegister> for Definition8 {
    fn from(value: CpuRegister) -> Self {
        Self::CpuRegister(value)
    }
}

impl BitOr for Variable8 {
    type Output = Definition8;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::Output::Or(self, rhs)
    }
}

impl BitAnd for Variable8 {
    type Output = Definition8;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::Output::And(self, rhs)
    }
}

impl BitXor for Variable8 {
    type Output = Definition8;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::Output::Xor(self, rhs)
    }
}

impl Debug for Definition8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Immediate(immediate) => write!(f, "0x{immediate:02x}"),
            Self::CpuRegister(cpu_register) => write!(f, "{cpu_register:?}"),
            Self::Ram(variable) => write!(f, "ram[{variable:?}]"),
            Self::Rom(variable) => write!(f, "rom[{variable:?}]"),
            Self::LowByte(variable) => write!(f, "<{variable:?}"),
            Self::HighByte(variable) => write!(f, ">{variable:?}"),
            Self::Or(u8_0, u8_1) => write!(f, "({u8_0:?} | {u8_1:?}"),
            Self::And(u8_0, u8_1) => write!(f, "({u8_0:?} & {u8_1:?}"),
            Self::Xor(u8_0, u8_1) => write!(f, "({u8_0:?} ^ {u8_1:?}"),
            Self::RotateLeft {
                operand,
                operand_carry,
            } => write!(f, "(({operand:?}, {operand_carry:?}) << 1)"),
            Self::RotateRight {
                operand,
                operand_carry,
            } => write!(f, "(({operand:?}, {operand_carry:?}) >> 1)"),
            Self::Sum {
                operand_0,
                operand_1,
                operand_carry,
            } => write!(f, "({operand_0:?} + {operand_1:?} + {operand_carry:?})"),
            Self::Difference {
                operand_0,
                operand_1,
                operand_borrow,
            } => write!(f, "({operand_0:?} - {operand_1:?} - {operand_borrow:?})"),
            Self::Select {
                condition,
                result_if_true,
                result_if_false,
            } => write!(
                f,
                "(if {condition:?} then {result_if_true:?} else {result_if_false:?})"
            ),
        }
    }
}

#[derive(Clone)]
pub(super) enum Destination8 {
    CpuRegister(CpuRegister),
    Ram(Variable16),
}

impl From<CpuRegister> for Destination8 {
    fn from(value: CpuRegister) -> Self {
        Self::CpuRegister(value)
    }
}

impl Debug for Destination8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CpuRegister(u8) => write!(f, "{u8:?}"),
            Self::Ram(variable) => write!(f, "ram[{variable:?}]"),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Variable16 {
    pub id: usize,
}

impl Debug for Variable16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "var_{}", self.id)
    }
}

#[derive(Clone)]
pub(super) enum Definition16 {
    Immediate(u16),
    Pc,
    FromU8s {
        high: Variable8,
        low: Variable8,
    },
    Sum {
        operand_0: Variable16,
        operand_1: Variable16,
    },
    Select {
        condition: Variable1,
        result_if_true: Variable16,
        result_if_false: Variable16,
    },
}

impl From<u16> for Definition16 {
    fn from(value: u16) -> Self {
        Self::Immediate(value)
    }
}

impl Rem for Variable8 {
    type Output = Definition16;

    fn rem(self, rhs: Self) -> Self::Output {
        Self::Output::FromU8s {
            low: rhs,
            high: self,
        }
    }
}

impl Debug for Definition16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Immediate(immediate) => write!(f, "0x{immediate:04x}"),
            Self::Pc => write!(f, "pc"),
            Self::FromU8s { low, high } => write!(f, "({high:?} % {low:?})"),
            Self::Sum {
                operand_0,
                operand_1,
            } => write!(f, "({operand_0:?} + {operand_1:?})"),
            Self::Select {
                condition,
                result_if_true,
                result_if_false,
            } => write!(
                f,
                "(if {condition:?} then {result_if_true:?} else {result_if_false:?})"
            ),
        }
    }
}
