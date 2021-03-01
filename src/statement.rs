use crate::expression::Expression;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Expression(Box<Expression>),
    Print(Box<Expression>),
}

impl Statement {
    pub fn accept<T>(&self, visitor: &mut dyn StatementVisitor<T>) -> T {
        match *self {
            Self::Expression(ref e) => visitor.visit_expression(e),
            Self::Print(ref e) => visitor.visit_print(e),
        }
    }
}

pub trait StatementVisitor<T> {
    fn visit_expression(&mut self, expression: &Expression) -> T;
    fn visit_print(&mut self, expression: &Expression) -> T;
}
