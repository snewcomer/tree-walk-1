use crate::lexer::LexemeKind;
use crate::visitor::Visitor;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: LexemeKind,
        right: Box<Expr>,
    },
    Literal(Value),
    Unary {
        operator: LexemeKind,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Error {
        line: usize,
        message: String,
    }
}

// a single element tuple struct over a generic type will not work.
// arms in parser will return different types for T
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    BOOLEAN(bool),
    STRING(String),
    NUMBER(f64),
}

impl Expr {
    pub(crate) fn accept<T>(&self, visitor: &mut dyn Visitor<T>) -> T {
        match self {
            Expr::Binary { operator, ref left, ref right } => {
                visitor.visit_binary(left, operator, right)
            }
            Expr::Unary { operator, ref right } => {
                visitor.visit_unary(operator, right)
            }
            Expr::Grouping(ref val) => {
                visitor.visit_grouping(val)
            }
            Expr::Literal(v) => {
                visitor.visit_literal(v)
            }
            Expr::Error { line, message } => {
                visitor.visit_error(line, message)
            }
        }
    }

    pub(crate) fn debug(&self) -> String {
        match self {
            Expr::Binary { operator, left, right } => {
                let mut st = String::new();
                st.push_str("(");

                let op = operator.to_string();
                st.push_str(&op);
                st.push_str(" ");

                let l = &left.debug();
                st.push_str(l);
                st.push_str(" ");

                let r = &right.debug();
                st.push_str(r);

                st
            },
            Expr::Literal(v) => {
                match v {
                    Value::BOOLEAN(true) => "true".to_string(),
                    Value::BOOLEAN(false) => "true".to_string(),
                    Value::STRING(st) => st.to_string(),
                    Value::NUMBER(n) => n.to_string(),
                }
            }
            Expr::Unary { operator, right } => {
                let mut st = String::new();
                st.push_str("( ");

                let op = operator.to_string();
                st.push_str(&op);
                st.push_str(" ");

                let r = &right.debug();
                st.push_str(r);
                st.push_str(" ");

                st
            },
            Expr::Grouping(value) => {
                value.debug()
            },
            Expr::Error { message, .. } => message.to_string()
        }
    }
}

