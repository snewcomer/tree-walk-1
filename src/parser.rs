use core::slice;
use std::error;
use std::fmt;
use std::iter;

use crate::expression::{Expression, Value};
use crate::statement::Statement;
use crate::token::{Lexeme, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    InvalidAssignmentTarget { token: Token },
    UnexpectedToken { token: Token, message: String },
    UnexpectedEnd,
}

impl ParseError {
    fn invalid_assignment_target(token: Token) -> Self {
        Self::InvalidAssignmentTarget { token }
    }

    fn unexpected_token(token: Token, message: String) -> Self {
        Self::UnexpectedToken { token, message }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidAssignmentTarget { token } => write!(
                f,
                "[line {}] at '{}' {}",
                token.line,
                token.to_string(),
                "Invalid assignment target"
            ),
            Self::UnexpectedToken { token, message } => write!(
                f,
                "[line {}] at '{}' {}",
                token.line,
                token.to_string(),
                message
            ),
            Self::UnexpectedEnd => {
                write!(f, "Unexpected end of tokens")
            }
        }
    }
}

impl error::Error for ParseError {}

type StatementParseResult = Result<Statement, ParseError>;
type ExpressionParseResult = Result<Expression, ParseError>;

pub struct Parser<'a> {
    tokens: iter::Peekable<slice::Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Parser {
            tokens: tokens.iter().peekable(),
        }
    }

    fn declaration(&mut self) -> StatementParseResult {
        if matches!(self.peek_lexeme(), Some(&Lexeme::Var)) {
            self.advance();
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> StatementParseResult {
        let name = self.consume(
            |l| match l {
                &Lexeme::Identifier(_) => true,
                _ => false,
            },
            "Expected variable name.".to_owned(),
        )?;

        let mut initializer: Option<Box<Expression>> = None;

        if matches!(self.peek_lexeme(), Some(&Lexeme::Equal)) {
            self.advance();
            let expression = self.expression()?;
            initializer = Some(Box::new(expression));
        }

        self.consume(
            |l| l == &Lexeme::Semicolon,
            "Expected ';' after variable declaration.".to_owned(),
        )?;

        Ok(Statement::Var { name, initializer })
    }

    fn statement(&mut self) -> StatementParseResult {
        match self.peek_lexeme() {
            Some(&Lexeme::Print) => {
                self.advance();
                self.print_statement()
            }
            Some(&Lexeme::LeftBrace) => {
                self.advance();
                self.block()
            }
            _ => self.expression_statement(),
        }
    }

    fn print_statement(&mut self) -> StatementParseResult {
        let expression = self.expression()?;
        self.consume(
            |l| l == &Lexeme::Semicolon,
            "Expected ';' after value.".to_owned(),
        )?;

        Ok(Statement::Print(Box::new(expression)))
    }

    fn block(&mut self) -> StatementParseResult {
        let mut statements: Vec<Statement> = Vec::new();
        loop {
            match self.peek_lexeme() {
                None => break,
                Some(Lexeme::RightBrace) => break,
                _ => {
                    statements.push(self.declaration()?);
                }
            };
        }

        self.consume(
            |l| l == &Lexeme::RightBrace,
            "Expected '}' after block.".to_owned(),
        )?;

        Ok(Statement::Block(statements))
    }

    fn expression_statement(&mut self) -> StatementParseResult {
        let expression = self.expression()?;
        self.consume(
            |l| l == &Lexeme::Semicolon,
            "Expected ';' after value.".to_owned(),
        )?;

        Ok(Statement::Expression(Box::new(expression)))
    }

    fn expression(&mut self) -> ExpressionParseResult {
        self.assignment()
    }

    fn assignment(&mut self) -> ExpressionParseResult {
        let expr = self.equality()?;

        if matches!(self.peek_lexeme(), Some(&Lexeme::Equal)) {
            let equals = self.advance().unwrap();
            let value = self.assignment()?;

            if let Expression::Variable(name) = expr {
                return Ok(Expression::Assign {
                    name,
                    value: Box::new(value),
                });
            }

            return Err(ParseError::invalid_assignment_target(equals));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> ExpressionParseResult {
        let mut expr = self.comparision()?;

        while matches!(
            self.peek_lexeme(),
            Some(&Lexeme::BangEqual) | Some(&Lexeme::EqualEqual)
        ) {
            let operator = self.advance().unwrap();
            let right = self.comparision()?;

            expr = Expression::Binary {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparision(&mut self) -> ExpressionParseResult {
        let mut expr = self.term()?;

        while matches!(
            self.peek_lexeme(),
            Some(&Lexeme::Greater)
                | Some(&Lexeme::GreaterEqual)
                | Some(&Lexeme::Less)
                | Some(&Lexeme::LessEqual)
        ) {
            let operator = self.advance().unwrap();
            let right = self.term()?;

            expr = Expression::Binary {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> ExpressionParseResult {
        let mut expr = self.factor()?;

        while matches!(
            self.peek_lexeme(),
            Some(&Lexeme::Minus) | Some(&Lexeme::Plus)
        ) {
            let operator = self.advance().unwrap();
            let right = self.factor()?;

            expr = Expression::Binary {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ExpressionParseResult {
        let mut expr = self.unary()?;

        while matches!(
            self.peek_lexeme(),
            Some(&Lexeme::Slash) | Some(&Lexeme::Star)
        ) {
            let operator = self.advance().unwrap();
            let right = self.unary()?;

            expr = Expression::Binary {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ExpressionParseResult {
        if matches!(
            self.peek_lexeme(),
            Some(&Lexeme::Bang) | Some(&Lexeme::Minus)
        ) {
            let operator = self.advance().unwrap();
            let right = self.unary()?;

            return Ok(Expression::Unary {
                operator: operator,
                expression: Box::new(right),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> ExpressionParseResult {
        match self.advance() {
            Some(Token {
                lexeme: Lexeme::False,
                ..
            }) => Ok(Expression::Literal(Value::Bool(false))),
            Some(Token {
                lexeme: Lexeme::True,
                ..
            }) => Ok(Expression::Literal(Value::Bool(true))),
            Some(Token {
                lexeme: Lexeme::Nil,
                ..
            }) => Ok(Expression::Literal(Value::Nil)),
            Some(Token {
                lexeme: Lexeme::Number(number),
                ..
            }) => Ok(Expression::Literal(Value::Number(number))),
            Some(Token {
                lexeme: Lexeme::String(string),
                ..
            }) => Ok(Expression::Literal(Value::String(string))),
            Some(
                token
                @
                Token {
                    lexeme: Lexeme::Identifier(_),
                    ..
                },
            ) => Ok(Expression::Variable(token)),
            Some(Token {
                lexeme: Lexeme::LeftParen,
                ..
            }) => {
                let expr = self.expression()?;
                self.consume(
                    |l| l == &Lexeme::RightParen,
                    "Expected ')' after expression.".to_owned(),
                )?;
                Ok(Expression::Grouping(Box::new(expr)))
            }
            Some(token @ Token { .. }) => Err(ParseError::unexpected_token(
                token,
                "Expected expression.".to_owned(),
            )),
            _ => Err(ParseError::UnexpectedEnd),
        }
    }

    fn advance(&mut self) -> Option<Token> {
        self.tokens.next().map(|t| t.clone())
    }

    fn peek_lexeme(&mut self) -> Option<&Lexeme> {
        self.tokens.peek().map(|t| &t.lexeme)
    }

    fn consume<P>(&mut self, predicate: P, error_message: String) -> Result<Token, ParseError>
    where
        P: Fn(&Lexeme) -> bool,
    {
        let result = match self.tokens.peek() {
            Some(token) => {
                if predicate(&token.lexeme) {
                    Ok((*token).clone())
                } else {
                    Err(ParseError::unexpected_token(
                        (*token).clone(),
                        error_message,
                    ))
                }
            }
            None => Err(ParseError::UnexpectedEnd),
        };

        if result.is_ok() {
            self.advance();
        }

        result
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = StatementParseResult;

    fn next(&mut self) -> Option<Self::Item> {
        match self.peek_lexeme() {
            Some(&Lexeme::Eof) => None,
            _ => Some(self.declaration()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression::{Expression, Value};
    use crate::token::{Lexeme, Token};

    #[test]
    fn it_handles_string_literal_expressions() {
        let tokens = vec![
            Token::new(Lexeme::String("string literal".to_owned()), 0),
            Token::new(Lexeme::Semicolon, 0),
            Token::new(Lexeme::Eof, 0),
        ];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.next(),
            Some(Ok(Statement::Expression(Box::new(Expression::Literal(
                Value::String("string literal".to_owned())
            )))))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_handles_number_literal_expressions() {
        let tokens = vec![
            Token::new(Lexeme::Number(12.0), 0),
            Token::new(Lexeme::Semicolon, 0),
            Token::new(Lexeme::Eof, 0),
        ];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.next(),
            Some(Ok(Statement::Expression(Box::new(Expression::Literal(
                Value::Number(12.0)
            )))))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_handles_true_boolean_literal_expressions() {
        let tokens = vec![
            Token::new(Lexeme::True, 0),
            Token::new(Lexeme::Semicolon, 0),
            Token::new(Lexeme::Eof, 0),
        ];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.next(),
            Some(Ok(Statement::Expression(Box::new(Expression::Literal(
                Value::Bool(true)
            )))))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_handles_false_boolean_literal_expressions() {
        let tokens = vec![
            Token::new(Lexeme::False, 0),
            Token::new(Lexeme::Semicolon, 0),
            Token::new(Lexeme::Eof, 0),
        ];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.next(),
            Some(Ok(Statement::Expression(Box::new(Expression::Literal(
                Value::Bool(false)
            )))))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_handles_nil_literal_expressions() {
        let tokens = vec![
            Token::new(Lexeme::Nil, 0),
            Token::new(Lexeme::Semicolon, 0),
            Token::new(Lexeme::Eof, 0),
        ];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.next(),
            Some(Ok(Statement::Expression(Box::new(Expression::Literal(
                Value::Nil
            )))))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_handles_unary_expressions() {
        let tokens = vec![
            Token::new(Lexeme::Minus, 0),
            Token::new(Lexeme::Number(12.0), 0),
            Token::new(Lexeme::Semicolon, 0),
            Token::new(Lexeme::Eof, 0),
        ];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.next(),
            Some(Ok(Statement::Expression(Box::new(Expression::Unary {
                operator: Token::new(Lexeme::Minus, 0),
                expression: Box::new(Expression::Literal(Value::Number(12.0)))
            }))))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_handles_binary_expressions() {
        let tokens = vec![
            Token::new(Lexeme::Number(2.0), 0),
            Token::new(Lexeme::Minus, 0),
            Token::new(Lexeme::Number(12.0), 0),
            Token::new(Lexeme::Semicolon, 0),
            Token::new(Lexeme::Eof, 0),
        ];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.next(),
            Some(Ok(Statement::Expression(Box::new(Expression::Binary {
                left: Box::new(Expression::Literal(Value::Number(2.0))),
                operator: Token::new(Lexeme::Minus, 0),
                right: Box::new(Expression::Literal(Value::Number(12.0)))
            }))))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_handles_grouping_expressions() {
        let tokens = vec![
            Token::new(Lexeme::LeftParen, 0),
            Token::new(Lexeme::Number(2.0), 0),
            Token::new(Lexeme::RightParen, 0),
            Token::new(Lexeme::Semicolon, 0),
            Token::new(Lexeme::Eof, 0),
        ];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.next(),
            Some(Ok(Statement::Expression(Box::new(Expression::Grouping(
                Box::new(Expression::Literal(Value::Number(2.0)))
            )))))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_handles_unclosed_grouping_expressions() {
        let tokens = vec![
            Token::new(Lexeme::LeftParen, 0),
            Token::new(Lexeme::Number(2.0), 0),
            Token::new(Lexeme::Eof, 0),
        ];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.next(),
            Some(Err(ParseError::UnexpectedToken {
                token: Token::new(Lexeme::Eof, 0),
                message: "Expected ')' after expression.".to_owned()
            }))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_handles_variable_declarations() {
        let tokens = vec![
            Token::new(Lexeme::Var, 0),
            Token::new(Lexeme::Identifier("foo".to_owned()), 0),
            Token::new(Lexeme::Semicolon, 0),
            Token::new(Lexeme::Eof, 0),
        ];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.next(),
            Some(Ok(Statement::Var {
                name: Token::new(Lexeme::Identifier("foo".to_owned()), 0),
                initializer: None
            }))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_handles_variable_declarations_with_initializers() {
        let tokens = vec![
            Token::new(Lexeme::Var, 0),
            Token::new(Lexeme::Identifier("foo".to_owned()), 0),
            Token::new(Lexeme::Equal, 0),
            Token::new(Lexeme::Number(5.0), 0),
            Token::new(Lexeme::Semicolon, 0),
            Token::new(Lexeme::Eof, 0),
        ];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.next(),
            Some(Ok(Statement::Var {
                name: Token::new(Lexeme::Identifier("foo".to_owned()), 0),
                initializer: Some(Box::new(Expression::Literal(Value::Number(5.0))))
            }))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_handles_incomplete_variable_declarations() {
        let tokens = vec![Token::new(Lexeme::Var, 0), Token::new(Lexeme::Eof, 0)];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.next(),
            Some(Err(ParseError::UnexpectedToken {
                token: Token::new(Lexeme::Eof, 0),
                message: "Expected variable name.".to_owned()
            }))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_handles_variable_declarations_without_semicolons() {
        let tokens = vec![
            Token::new(Lexeme::Var, 0),
            Token::new(Lexeme::Identifier("foo".to_owned()), 0),
            Token::new(Lexeme::Eof, 0),
        ];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.next(),
            Some(Err(ParseError::UnexpectedToken {
                token: Token::new(Lexeme::Eof, 0),
                message: "Expected ';' after variable declaration.".to_owned()
            }))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_handles_assignment() {
        let tokens = vec![
            Token::new(Lexeme::Identifier("foo".to_owned()), 0),
            Token::new(Lexeme::Equal, 0),
            Token::new(Lexeme::Number(5.0), 0),
            Token::new(Lexeme::Semicolon, 0),
            Token::new(Lexeme::Eof, 0),
        ];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.next(),
            Some(Ok(Statement::Expression(Box::new(Expression::Assign {
                name: Token::new(Lexeme::Identifier("foo".to_owned()), 0),
                value: Box::new(Expression::Literal(Value::Number(5.0)))
            }))))
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn it_handles_incomplete_expressions() {
        let tokens = vec![Token::new(Lexeme::Minus, 0), Token::new(Lexeme::Eof, 0)];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.next(),
            Some(Err(ParseError::UnexpectedToken {
                token: Token::new(Lexeme::Eof, 0),
                message: "Expected expression.".to_owned()
            }))
        );
    }

    #[test]
    fn it_handles_a_premature_end_of_tokens() {
        let tokens = vec![Token::new(Lexeme::Minus, 0)];
        let mut parser = Parser::new(&tokens);

        assert_eq!(parser.next(), Some(Err(ParseError::UnexpectedEnd)));
    }
}
