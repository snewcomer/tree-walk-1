use crate::expression::{Expression, ExpressionVisitor, Value};
use crate::statement::{Statement, StatementVisitor};
use crate::token::Token;

pub struct AstPrinter;

impl AstPrinter {
    pub fn parenthesize(&mut self, name: String, expressions: &[&Expression]) -> String {
        let mut parts: Vec<String> = Vec::new();
        parts.push(name);

        for e in expressions {
            parts.push(e.accept(self));
        }

        format!("({})", parts.join(" "))
    }
}

impl ExpressionVisitor<String> for AstPrinter {
    fn visit_assign(&mut self, name: &Token, value: &Expression) -> String {
        format!("(= {} {})", name.identifier(), value.accept(self))
    }

    fn visit_binary(&mut self, left: &Expression, operator: &Token, right: &Expression) -> String {
        self.parenthesize(operator.to_string(), &[left, right])
    }

    fn visit_grouping(&mut self, expression: &Expression) -> String {
        self.parenthesize("group".to_owned(), &[expression])
    }

    fn visit_literal(&mut self, value: &Value) -> String {
        value.to_string()
    }

    fn visit_unary(&mut self, operator: &Token, expression: &Expression) -> String {
        self.parenthesize(operator.to_string(), &[expression])
    }

    fn visit_variable(&mut self, name: &Token) -> String {
        name.identifier()
    }
}

impl StatementVisitor<String> for AstPrinter {
    fn visit_block(&mut self, statements: &Vec<Statement>) -> String {
        let statements_string = statements
            .into_iter()
            .map(|e| e.accept(self))
            .collect::<Vec<String>>()
            .join(" ");

        format!("(block {})", statements_string)
    }

    fn visit_expression(&mut self, expression: &Expression) -> String {
        self.parenthesize(";".to_owned(), &[expression])
    }

    fn visit_print(&mut self, expression: &Expression) -> String {
        self.parenthesize("print".to_owned(), &[expression])
    }

    fn visit_var(&mut self, name: &Token, initializer: Option<&Expression>) -> String {
        match initializer {
            Some(expression) => {
                format!("(var {} = {})", name.identifier(), expression.accept(self))
            }
            None => format!("(var {})", name.identifier()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression::{Expression, Value};
    use crate::statement::Statement;
    use crate::token::{Lexeme, Token};

    #[test]
    fn it_works() {
        let statement = Statement::Var {
            name: Token::new(Lexeme::Identifier("foo".to_owned()), 1),
            initializer: Some(Box::new(Expression::Binary {
                left: Box::new(Expression::Unary {
                    operator: Token::new(Lexeme::Minus, 0),
                    expression: Box::new(Expression::Literal(Value::Number(123.0))),
                }),
                operator: Token::new(Lexeme::Star, 0),
                right: Box::new(Expression::Grouping(Box::new(Expression::Literal(
                    Value::Number(45.67),
                )))),
            })),
        };

        assert_eq!(
            statement.accept(&mut AstPrinter),
            "(var foo = (* (- 123) (group 45.67)))"
        );
    }
}
