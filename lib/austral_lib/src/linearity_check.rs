use crate::{
    common::Identifier,
    r#type::Universe,
    stages::{TExpr, TStmt},
    type_system::type_universe,
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Appearances {
    pub consumed: i32,
    pub read: i32,
    pub write: i32,
    pub path: i32,
}

#[derive(Default, Clone, PartialEq)]
pub enum VarState {
    #[default]
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

pub type StateTable = HashMap<Identifier, (i32, VarState)>;

pub fn check_statement(
    stmt_name: &str,
    state_table: &mut StateTable,
    stmt: &TStmt,
    depth: i32,
) -> bool {
    match stmt {
        TStmt::TSkip(_) => true,
        TStmt::TLet(_, name, expr, _, ty, body) => {
            // This is an internal error because the compiler is expected to catch redefinitions before this.
            debug_assert!(!state_table.contains_key(name));
            check_expression(state_table, depth, expr);
            if type_universe(ty) == Universe::LinearUniverse {
                state_table.insert(name.clone(), (depth, VarState::Unconsumed));
                let result = check_statement(stmt_name, state_table, body, depth);
                // the body extends until the end of the block (scope)
                state_table.remove(name);
                result
            } else {
                check_statement(stmt_name, state_table, body, depth)
            }
        }
        TStmt::TAssign(_, lvalue, rvalue) => {
            check_expression(state_table, depth, lvalue)
                && check_expression(state_table, depth, rvalue)
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
            //check_expression(state_table, depth, expr) &&
            state_table.is_empty()
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
        TStmt::TIf(_, _cond, then_stmt, else_stmt) => {
            let cond_result = true; //check_expression(state_table, depth, cond);
            let mut then_table = state_table.clone();
            let mut else_table = state_table.clone();
            let then_result = check_statement(stmt_name, &mut then_table, then_stmt, depth);
            let else_result = check_statement(stmt_name, &mut else_table, else_stmt, depth);
            *state_table = then_table;
            cond_result && then_result && else_result && *state_table == else_table
        }
        TStmt::TCase(_, _, _, _) => {
            todo!()
        }
        TStmt::TWhile(_, _cond, body) => {
            //check_expression(state_table, depth, cond) &&
            check_statement(stmt_name, state_table, body, depth + 1)
        }
        TStmt::TFor(_, _, start, end, body) => {
            check_expression(state_table, depth, start)
                && check_expression(state_table, depth, end)
                && check_statement(stmt_name, state_table, body, depth + 1)
        }
    }
}

fn check_expression(_state_table: &mut StateTable, _depth: i32, _texpr: &TExpr) -> bool {
    true
}

#[cfg(test)]
mod test {
    use crate::span::Span;
    use crate::{
        common::{Identifier, Mutability},
        linearity_check::check_statement,
        r#type::Ty,
        stages::{TExpr, TStmt},
    };

    use std::collections::HashMap;
    #[test]
    fn test_let() {
        let mut state_table = HashMap::new();
        let stmt = TStmt::TLet(
            Span::default(),
            Identifier::new("x"),
            Box::new(TExpr::TIntConstant("1".to_string())),
            Mutability::Immutable,
            Ty::SpanMut(Box::new(Ty::Boolean), Box::new(Ty::Boolean)),
            Box::new(TStmt::TSkip(Span::default())),
        );
        let result = check_statement("test_let", &mut state_table, &stmt, 0);
        assert!(result);
    }
}
