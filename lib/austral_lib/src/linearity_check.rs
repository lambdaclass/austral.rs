use crate::{r#type::Ty, span::Span, stages::TStmt};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Appearances {
    pub consumed: i32,
    pub read: i32,
    pub write: i32,
    pub path: i32,
}

pub enum VarState {
    Unconsumed,
    BorrowedRead,
    BorrowedWrite,
    Consumed,
}

impl std::fmt::Debug for VarState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            VarState::Unconsumed => write!(f, "not yet consumed"),
            VarState::BorrowedRead => write!(f, "borrowed (read-only)"),
            VarState::BorrowedWrite => write!(f, "borrowed (read-write)"),
            VarState::Consumed => write!(f, "consumed"),
        }
    }
}

pub enum Partitions {
    Zero,
    One,
    MoreThanOne,
}

impl Partitions {
    pub fn partition(n: usize) -> Self {
        if n > 1 {
            Partitions::MoreThanOne
        } else if n == 1 {
            Partitions::One
        } else if n == 0 {
            Partitions::Zero
        } else {
            panic!("Impossible")
        }
    }
}

pub fn partitions_are_consistent(stmt_name: &str, a: Partitions, b: Partitions) -> bool {
    match (a, b) {
        (Partitions::Zero, Partitions::Zero) => true,
        (Partitions::One, Partitions::One) => true,
        (Partitions::MoreThanOne, Partitions::MoreThanOne) => true,
        _ => false,
    }
}

pub type StateTable = HashMap<String, (i32, VarState)>;

fn check_statement(stmt_name: &str, state_table: &mut StateTable, stmt: &TStmt) -> bool {
    match stmt {
        TStmt::TSkip(_) => true,
        TStmt::TLet(_, name, _, _, _) => {
            // adds the new entry to the hash
            // if the variable is already in the hash, then it's a redefinition
            // if the variable is not in the hash, then it's a definition
            false
        }
        TStmt::TAssign(..) => {
            panic!("TODO: Implement check_statement for TAssign")
        }
        TStmt::TAssignTmp(..) => {
            panic!("TODO: Implement check_statement for TAssignTmp")
        }
        TStmt::TBlock(..) => {
            panic!("TODO: Implement check_statement for TBlock")
        }
        TStmt::TDiscarding(..) => {
            panic!("TODO: Implement check_statement for TDiscarding")
        }
        TStmt::TReturn(..) => {
            panic!("TODO: Implement check_statement for TReturn")
        }
        TStmt::TLetTmp(..) => {
            panic!("TODO: Implement check_statement for TLetTmp")
        }
        TStmt::TBorrow { .. } => {
            panic!("TODO: Implement check_statement for TBorrow")
        }
        TStmt::TDestructure(_, _, _, _, _) => todo!(),
        TStmt::TAssignVar(_, _, _) => todo!(),
        TStmt::TInitialAssign(_, _) => todo!(),
        TStmt::TIf(_, _, _, _) => todo!(),
        TStmt::TCase(_, _, _, _) => todo!(),
        TStmt::TWhile(_, _, _) => todo!(),
        TStmt::TFor(_, _, _, _, _) => todo!(),
    }
}
