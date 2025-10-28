use std::{
    cell::RefCell,
    fmt::Debug,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

pub(super) struct Function {
    pub basic_block: Rc<RefCell<BasicBlock>>,
}

pub(super) struct BasicBlock {
    pub variable_id_counter: Rc<AtomicUsize>,
    pub has_argument: bool,
    pub instructions: Vec<Instruction>,
    pub jump: Jump,
}

impl BasicBlock {
    pub(super) fn new(variable_id_counter: Rc<AtomicUsize>) -> Self {
        Self {
            variable_id_counter,
            has_argument: false,
            instructions: vec![],
            jump: Jump::Return,
        }
    }

    pub(super) fn set_has_argument(&mut self, has_argument: bool) {
        self.has_argument = has_argument;
    }

    pub(super) fn define_1(&mut self, definition: Definition1) -> Variable1 {
        let variable = Variable1 {
            id: self.variable_id_counter.fetch_add(1, Ordering::Relaxed),
        };
        self.instructions.push(Instruction::Define1 {
            variable,
            definition,
        });
        variable
    }

    pub(super) fn define_8(&mut self, definition: Definition8) -> Variable8 {
        let variable = Variable8 {
            id: self.variable_id_counter.fetch_add(1, Ordering::Relaxed),
        };
        self.instructions.push(Instruction::Define8 {
            variable,
            definition,
        });
        variable
    }

    pub(super) fn define_16(&mut self, definition: Definition16) -> Variable16 {
        let variable = Variable16 {
            id: self.variable_id_counter.fetch_add(1, Ordering::Relaxed),
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
    Store8 {
        destination: Destination8,
        variable: Variable8,
    },
    Store16 {
        destination: Destination16,
        variable: Variable16,
    },
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Define1 {
                variable,
                definition,
            } => write!(f, "{variable:?} = {definition:?}"),
            Self::Define8 {
                variable,
                definition,
            } => write!(f, "{variable:?} = {definition:?}"),
            Self::Define16 {
                variable,
                definition,
            } => write!(f, "{variable:?} = {definition:?}"),
            Self::Store8 {
                destination,
                variable,
            } => write!(f, "{destination:?} = {variable:?}"),
            Self::Store16 {
                destination,
                variable,
            } => write!(f, "{destination:?} = {variable:?}"),
        }
    }
}

#[derive(Clone)]
pub(crate) enum Jump {
    BasicBlock {
        condition: Variable1,
        target_if_true: Rc<RefCell<BasicBlock>>,
        target_if_true_argument: Option<Variable8>,
        target_if_false: Rc<RefCell<BasicBlock>>,
        target_if_false_argument: Option<Variable8>,
    },
    Return,
}

impl Debug for Jump {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Return => write!(f, "return"),
            Self::BasicBlock {
                condition,
                target_if_true: _,
                target_if_true_argument: _,
                target_if_false: _,
                target_if_false_argument: _,
            } => {
                write!(
                    f,
                    "jump to (if {condition:?} basic block TODO else basic block TODO)"
                )
            }
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Variable1 {
    pub id: usize,
}

impl Debug for Variable1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "var_{}", self.id)
    }
}

pub(crate) enum Definition1 {
    Not(Variable1),
    And(Variable1, Variable1),
    EqualToZero(Variable8),
    U8Bit {
        operand: Variable8,
        index: u8,
    },
    LessThanOrEqual16(Variable16, Variable16),
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

impl Debug for Definition1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Not(register_u1) => write!(f, "!{register_u1:?}"),
            Self::And(operand_0, operand_1) => write!(f, "({operand_0:?} & {operand_1:?})"),
            Self::EqualToZero(u8) => write!(f, "({u8:?} == 0)"),
            Self::U8Bit { operand, index } => write!(f, "{operand:?}.bit({index})"),
            Self::LessThanOrEqual16(operand_0, operand_1) => {
                write!(f, "({operand_0:?} <= {operand_1:?})")
            }
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
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Variable8 {
    pub id: usize,
}

impl Debug for Variable8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "var_{}", self.id)
    }
}

#[derive(Clone)]
pub(crate) enum Definition8 {
    BasicBlockArgument,
    NativeMemory {
        address: *const u8,
        offset: Variable16,
    },
    Immediate(u8),
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
}

impl Debug for Definition8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BasicBlockArgument => write!(f, "arg"),
            Self::NativeMemory { address, offset } => {
                write!(f, "{address:?}[{offset:?}]")
            }
            Self::Immediate(immediate) => write!(f, "0x{immediate:02x}"),
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
        }
    }
}

#[derive(Clone)]
pub(super) enum Destination8 {
    NativeMemory {
        address: *mut u8,
        offset: Variable16,
    },
}

impl Debug for Destination8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NativeMemory { address, offset } => write!(f, "{address:?}[{offset:?}"),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Variable16 {
    pub id: usize,
}

impl Debug for Variable16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "var_{}", self.id)
    }
}

#[derive(Clone)]
pub(crate) enum Definition16 {
    NativeMemory {
        address: *const u16,
    },
    FromU8s {
        high: Variable8,
        low: Variable8,
    },
    Select {
        condition: Variable1,
        result_if_true: Variable16,
        result_if_false: Variable16,
    },
}

impl Debug for Definition16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NativeMemory { address } => write!(f, "{address:?}[0]"),
            Self::FromU8s { low, high } => write!(f, "({high:?} % {low:?})"),
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
pub(super) enum Destination16 {
    NativeMemory { address: *mut u16 },
}

impl Debug for Destination16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NativeMemory { address } => write!(f, "{address:?}[0]"),
        }
    }
}
