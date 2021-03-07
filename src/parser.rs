pub(crate) mod expression;

use crate::lexer::{LexemeKind, Token};
pub use expression::{Expr, Value};

pub(crate) struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

pub(crate) fn debug_tree(ast: &Expr) -> String {
    let mut st = String::new();
    st.push_str("(");
    if let Expr::Binary {
        left,
        operator,
        right,
    } = ast
    {
        let op = operator.to_string();
        st.push_str(&op);
        st.push_str(" ");

        let l = &(*left).debug();
        st.push_str(l);
        st.push_str(" ");

        let r = &(*right).debug();
        st.push_str(r);
    } else {
        println!("Not an expression");
    }

    st.push_str(")");
    st
}

impl Parser {
    pub(crate) fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, cursor: 0 }
    }

    pub(crate) fn parse(&mut self) -> Option<Expr> {
        self.expression()
    }

    fn at_end(&self) -> bool {
        self.peek_kind() == Some(LexemeKind::EOF)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    fn peek_kind(&self) -> Option<LexemeKind> {
        self.peek()
            .and_then(|Token { lexeme, .. }| Some(lexeme.clone()))
    }

    fn expect(&mut self, kind: LexemeKind) {
        if self.at(kind) {
            self.cursor += 1;
        } else {
            todo!();
        }
    }

    fn at(&self, kind: LexemeKind) -> bool {
        if self.at_end() {
            return false;
        };
        self.peek_kind() == Some(kind)
    }

    fn eat_whitespace(&mut self) {
        if self.peek_kind() == Some(LexemeKind::Whitespace) {
            self.cursor += 1;
        }
    }

    fn error(&self, line: usize, msg: &str) -> Option<Expr> {
        Some(Expr::Error { line, message: msg.to_string() })
    }

    fn is_equal(&self, kinds: Vec<LexemeKind>) -> bool {
        let res = kinds.iter().find(|&k| self.at(k.clone()));
        res.is_some()
    }

    fn expression(&mut self) -> Option<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Option<Expr> {
        let mut expr = self.comparison();

        self.eat_whitespace();

        while self.is_equal(vec![LexemeKind::BangEqual, LexemeKind::EqualEqual]) {
            let operator = self.peek_kind().unwrap();
            self.cursor += 1;
            let right = self.comparison();
            expr = Some(Expr::Binary {
                left: Box::new(expr.unwrap()),
                operator,
                right: Box::new(right.unwrap()),
            })
        }

        expr
    }

    fn comparison(&mut self) -> Option<Expr> {
        let mut expr = self.term();

        self.eat_whitespace();

        while self.is_equal(vec![
            LexemeKind::Greater,
            LexemeKind::GreaterEqual,
            LexemeKind::Less,
            LexemeKind::LessEqual,
        ]) {
            let operator = self.peek_kind().unwrap();

            self.cursor += 1;

            let right = self.term();
            expr = Some(Expr::Binary {
                left: Box::new(expr.unwrap()),
                operator,
                right: Box::new(right.unwrap()),
            })
        }

        expr
    }

    fn term(&mut self) -> Option<Expr> {
        let mut expr = self.factor();

        self.eat_whitespace();

        while self.is_equal(vec![LexemeKind::Minus, LexemeKind::Plus]) {
            let operator = self.peek_kind().unwrap();

            self.cursor += 1;

            let right = self.factor();
            expr = Some(Expr::Binary {
                left: Box::new(expr.unwrap()), // 1
                operator, // +
                right: Box::new(right.unwrap()), // 1
            })
        }

        expr
    }

    fn factor(&mut self) -> Option<Expr> {
        let mut expr = self.unary();

        self.eat_whitespace();

        while self.is_equal(vec![LexemeKind::Slash, LexemeKind::Star]) {
            let operator = self.peek_kind().unwrap();
            self.cursor += 1;
            let right = self.unary();
            expr = Some(Expr::Binary {
                left: Box::new(expr.unwrap()),
                operator,
                right: Box::new(right.unwrap()),
            })
        }

        expr
    }

    fn unary(&mut self) -> Option<Expr> {
        let mut res = None;

        self.eat_whitespace();

        while self.is_equal(vec![LexemeKind::Bang, LexemeKind::Minus, LexemeKind::Plus]) {
            let operator = self.peek_kind().unwrap();

            self.cursor += 1;

            let new = self.unary();
            match res {
                Some(Expr::Unary { operator, right }) => {
                    res = Some(Expr::Binary {
                        left: right,
                        operator: operator.clone(),
                        right: Box::new(new.unwrap()),
                    });
                },
                _ => {
                    res = Some(Expr::Unary {
                        operator,
                        right: Box::new(new.unwrap()),
                    });
                }
            }
        }

        if res.is_some() {
            res
        } else {
            let res = self.primary();
            let token = self.peek();
            if let Some(Token { lexeme: LexemeKind::UNEXPECTED(l), line }) = token {
                self.error(*line, &format!("Parsing error at {}", l))
            } else {
                res
            }
        }
    }

    fn primary(&mut self) -> Option<Expr> {
        let token = self.tokens.get(self.cursor).unwrap();
        match &token.lexeme {
            LexemeKind::FALSE => {
                self.cursor += 1;
                Some(Expr::Literal(Value::BOOLEAN(false)))
            }
            LexemeKind::TRUE => {
                self.cursor += 1;
                Some(Expr::Literal(Value::BOOLEAN(true)))
            }
            LexemeKind::STRING(st) => {
                self.cursor += 1;
                Some(Expr::Literal(Value::STRING(st.to_string())))
            }
            LexemeKind::NUMBER(num) => {
                self.cursor += 1;
                Some(Expr::Literal(Value::NUMBER(*num)))
            }
            LexemeKind::LeftParen => {
                self.cursor += 1;
                let expr = self.expression();

                self.expect(LexemeKind::RightParen);

                Some(Expr::Grouping(
                    Box::new(expr.unwrap()),
                ))
            }
            m => self.error(token.line, &format!("Parsing error at {}", m)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::Scanner;

    #[test]
    fn it_handles_binary() {
        let tokens = Scanner::new("1+1".to_owned()).collect();
        let ast = Parser::new(tokens).parse().unwrap();
        assert_eq!(
            ast,
            Expr::Binary {
                left: Box::new(Expr::Literal(Value::NUMBER(1.0))),
                operator: LexemeKind::Plus,
                right: Box::new(Expr::Literal(Value::NUMBER(1.0))),
            }
        );

        let tokens = Scanner::new("1 == 1".to_owned()).collect();
        let ast = Parser::new(tokens).parse().unwrap();
        assert_eq!(
            ast,
            Expr::Binary {
                left: Box::new(Expr::Literal(Value::NUMBER(1.0))),
                operator: LexemeKind::EqualEqual,
                right: Box::new(Expr::Literal(Value::NUMBER(1.0))),
            }
        );
    }

    #[test]
    fn it_handles_co() {
        let tokens = Scanner::new("1 >= 2".to_owned()).collect();
        let ast = Parser::new(tokens).parse().unwrap();
        assert_eq!(
            ast,
            Expr::Binary {
                left: Box::new(Expr::Literal(Value::NUMBER(1.0))),
                operator: LexemeKind::GreaterEqual,
                right: Box::new(Expr::Literal(Value::NUMBER(2.0))),
            }
        );

        let tokens = Scanner::new("1 <= 2".to_owned()).collect();
        let ast = Parser::new(tokens).parse().unwrap();
        assert_eq!(
            ast,
            Expr::Binary {
                left: Box::new(Expr::Literal(Value::NUMBER(1.0))),
                operator: LexemeKind::LessEqual,
                right: Box::new(Expr::Literal(Value::NUMBER(2.0))),
            }
        );
    }

    #[test]
    fn it_handles_unary() {
        let tokens = Scanner::new("-1".to_owned()).collect();
        let ast = Parser::new(tokens).parse().unwrap();
        assert_eq!(
            ast,
            Expr::Unary {
                operator: LexemeKind::Minus,
                right: Box::new(Expr::Literal(Value::NUMBER(1.0))),
            }
        );

        let tokens = Scanner::new("+1".to_owned()).collect();
        let ast = Parser::new(tokens).parse().unwrap();
        assert_eq!(
            ast,
            Expr::Unary {
                operator: LexemeKind::Plus,
                right: Box::new(Expr::Literal(Value::NUMBER(1.0))),
            }
        );
    }

    #[test]
    fn it_errors_keyword() {
        let tokens = Scanner::new("and".to_owned()).collect();
        let ast = Parser::new(tokens).parse().unwrap();
        assert_eq!(
            ast,
            Expr::Error { line: 0, message: "Parsing error at AND".to_string() }
        );
    }

    #[test]
    fn not_expression() {
        let tokens = Scanner::new("a".to_owned()).collect();
        let ast = Parser::new(tokens).parse().unwrap();
        assert_eq!(
            ast,
            Expr::Error { line: 0, message: "Parsing error at IDENTIFIER(\"a\")".to_string() }
        );
    }

    #[test]
    fn it_works_parenthesized_expression() {
        let tokens = Scanner::new("(1+1)".to_owned()).collect();
        let ast = Parser::new(tokens).parse().unwrap();
        assert_eq!(
            ast,
            Expr::Grouping(
                Box::new(Expr::Binary {
                    left: Box::new(Expr::Literal(Value::NUMBER(1.0))),
                    operator: LexemeKind::Plus,
                    right: Box::new(Expr::Literal(Value::NUMBER(1.0))),
                }),
            )
        );
    }

    #[test]
    fn it_works_plus_plus() {
        let tokens = Scanner::new("+1+1".to_owned()).collect();
        let ast = Parser::new(tokens).parse().unwrap();
        assert_eq!(
            ast,
            Expr::Binary {
                left: Box::new(Expr::Literal(Value::NUMBER(1.0))),
                operator: LexemeKind::Plus,
                right: Box::new(Expr::Literal(Value::NUMBER(1.0))),
            }
        );
    }
}
