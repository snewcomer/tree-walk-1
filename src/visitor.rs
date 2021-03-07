use crate::lexer::LexemeKind;
use crate::parser::{Expr, Value};

// Dynamic dispatch
// This has a higher runtime cost due to vtable lookups.
// This is compared to static dispatch, which will monomorphize each function that expects a
// generic type T.
pub trait Visitor<T> {
    fn visit_binary(&mut self, left: &Expr, operator: &LexemeKind, right: &Expr) -> T;
    fn visit_literal(&mut self, val: &Value) -> T;
    fn visit_unary(&mut self, operator: &LexemeKind, right: &Expr) -> T;
    fn visit_grouping(&mut self, val: &Expr) -> T;
    fn visit_error(&mut self, line: &usize, message: &str) -> T;
}
