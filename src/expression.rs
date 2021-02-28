use std::fmt;

use crate::token::Token;
use crate::visitor::Visitor;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            &Self::Nil => false,
            &Self::Bool(b) => b,
            _ => true,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Self::Nil => write!(f, "nil"),
            &Self::Bool(b) => write!(f, "{}", b),
            &Self::Number(n) => write!(f, "{}", n),
            &Self::String(ref s) => write!(f, "\"{}\"", s),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Binary {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Grouping(Box<Expression>),
    Literal(Value),
    Unary {
        operator: Token,
        expression: Box<Expression>,
    },
}

impl Expression {
    pub fn accept<T>(&self, visitor: &mut dyn Visitor<T>) -> T {
        match *self {
            Expression::Binary {
                ref left,
                ref operator,
                ref right,
            } => visitor.visit_binary(left, operator, right),
            Expression::Grouping(ref e) => visitor.visit_grouping(e),
            Expression::Literal(ref value) => visitor.visit_literal(value),
            Expression::Unary {
                ref operator,
                ref expression,
            } => visitor.visit_unary(operator, expression),
        }
    }
}
