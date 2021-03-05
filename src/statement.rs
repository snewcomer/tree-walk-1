use crate::expression::Expression;
use crate::token::Token;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Block(Vec<Statement>),
    Expression(Box<Expression>),
    Print(Box<Expression>),
    Var {
        name: Token,
        initializer: Option<Box<Expression>>,
    },
}

impl Statement {
    pub fn accept<T>(&self, visitor: &mut dyn StatementVisitor<T>) -> T {
        match &self {
            &Self::Block(ss) => visitor.visit_block(ss),
            &Self::Expression(ref e) => visitor.visit_expression(e),
            &Self::Print(ref e) => visitor.visit_print(e),
            &Self::Var {
                ref name,
                initializer,
            } => visitor.visit_var(name, initializer.as_deref()),
        }
    }
}

pub trait StatementVisitor<T> {
    fn visit_block(&mut self, statements: &Vec<Statement>) -> T;
    fn visit_expression(&mut self, expression: &Expression) -> T;
    fn visit_print(&mut self, expression: &Expression) -> T;
    fn visit_var(&mut self, name: &Token, initializer: Option<&Expression>) -> T;
}
