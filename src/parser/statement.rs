use crate::lexer::LexemeKind;
use super::expression::Expr;
use super::Parser;
use crate::visitor::StatementVisitor;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Block(Box<Vec<Stmt>>),
    VariableDef {
        ident: String,
        expr: Option<Expr>,
    },
    Print(Option<Expr>),
    Expr(Expr),
    Error {
        line: usize,
        message: String,
    }
}

impl Stmt {
    pub(crate) fn accept<T>(&self, visitor: &mut dyn StatementVisitor<T>) -> T {
        match self {
            Stmt::Block(stmts) => {
                visitor.visit_block(stmts)
            }
            Stmt::VariableDef { ident, expr } => {
                visitor.visit_variable_def(ident, expr)
            }
            Stmt::Print(expr) => {
                visitor.visit_print(expr)
            }
            Stmt::Expr(expr) => {
                visitor.visit_expr(expr)
            }
            Stmt::Error { line, message } => {
                visitor.visit_error(line, message)
            }
        }
    }
}

pub(crate) fn parse(p: &mut Parser) -> Option<Stmt> {
    p.eat_whitespace();

    if p.at(LexemeKind::VAR) {
        p.cursor += 1;
        // ultimately, this is what our program is made up of
        declaration_stmt(p)
    } else if p.at(LexemeKind::LeftBrace) {
        p.cursor += 1;

        block(p)
    } else {
        statement(p)
    }
}

fn block(p: &mut Parser) -> Option<Stmt> {
    let mut v: Vec<Stmt> = vec![];

    p.eat_whitespace();

    while p.at(LexemeKind::RightBrace) == false {
        let res = parse(p);
        v.push(res.unwrap());

        p.eat_whitespace();
    }

    p.eat_whitespace();

    p.cursor += 1; // RightBrace

    Some(Stmt::Block(Box::new(v)))
}

pub(crate) fn statement(p: &mut Parser) -> Option<Stmt> {
    if p.at(LexemeKind::PRINT) {
        p.cursor += 1; // PRINT
        print_stmt(p)
    } else {
        // fallthrough to expression
        let expr = p.expression()?;
        Some(Stmt::Expr(expr))
    }
}

fn declaration_stmt(p: &mut Parser) -> Option<Stmt> {
    // var x = 1+1;
    p.eat_whitespace();

    match p.expression() {
        Some(Expr::Assign { name, expr }) => {
            let stmt = Some(Stmt::VariableDef { ident: name, expr: Some(*expr) });
            // assert!(p.at(LexemeKind::Semicolon));
            p.cursor += 1;
            stmt
        }
        Some(Expr::Variable(name)) => {
            let stmt = Some(Stmt::VariableDef { ident: name, expr: None });
            // assert!(p.at(LexemeKind::Semicolon));
            p.cursor += 1;
            stmt
        }
        _ => Some(Stmt::Error { line: 0, message: "Unfinished right hand assignment".to_string() })
    }
}

fn print_stmt(p: &mut Parser) -> Option<Stmt> {
    p.cursor += 1; // LeftParen

    match p.peek_kind() {
        Some(LexemeKind::RightParen) => {
            p.cursor += 1; // RightParen
            // print();
            Some(Stmt::Print(None))
        }
        _ => {
            let expr = p.expression();

            if let Ok(()) = p.expect(LexemeKind::RightParen) {
                p.cursor += 1; // RightParen

                // semicolon optional
                if let Ok(_) = p.expect(LexemeKind::Semicolon) {
                   p.cursor += 1;
                }

                Some(Stmt::Print(expr))
            } else {
                Some(Stmt::Error { line: 0, message: "Unfinished print statement".to_string() })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Scanner;
    use crate::parser::{Parser, Value};

    #[test]
    fn it_stmt_works() {
        let tokens = Scanner::new("print(1)".to_owned()).collect();
        let mut p = Parser::new(tokens);
        let res = parse(&mut p);
        assert_eq!(
            res,
            Some(Stmt::Print(Some(Expr::Literal(Value::NUMBER(1.0)))))
        );
    }

    #[test]
    fn it_stmt_works_strings() {
        let tokens = Scanner::new("print(\"foo\")".to_owned()).collect();
        let mut p = Parser::new(tokens);
        let res = parse(&mut p);
        assert_eq!(
            res,
            Some(Stmt::Print(Some(Expr::Literal(Value::STRING("foo".to_string())))))
        );
    }

    #[test]
    fn it_accepts_nothing() {
        let tokens = Scanner::new("print()".to_owned()).collect();
        let mut p = Parser::new(tokens);
        let res = parse(&mut p);
        assert_eq!(
            res,
            Some(Stmt::Print(None))
        );
    }

    #[test]
    fn it_accepts_expressions() {
        let tokens = Scanner::new("print(8*8)".to_owned()).collect();
        let mut p = Parser::new(tokens);
        let res = parse(&mut p);
        assert_eq!(
            res,
            Some(Stmt::Print(Some(Expr::Binary {
                left: Box::new(Expr::Literal(Value::NUMBER(8.0))),
                operator: LexemeKind::Star,
                right: Box::new(Expr::Literal(Value::NUMBER(8.0))),
            })))
        );

        let tokens = Scanner::new("print(8 * 8)".to_owned()).collect();
        let mut p = Parser::new(tokens);
        let res = parse(&mut p);
        assert_eq!(
            res,
            Some(Stmt::Print(Some(Expr::Binary {
                left: Box::new(Expr::Literal(Value::NUMBER(8.0))),
                operator: LexemeKind::Star,
                right: Box::new(Expr::Literal(Value::NUMBER(8.0))),
            })))
        );

        let tokens = Scanner::new("print(8 *  8)".to_owned()).collect();
        let mut p = Parser::new(tokens);
        let res = parse(&mut p);
        assert_eq!(
            res,
            Some(Stmt::Print(Some(Expr::Binary {
                left: Box::new(Expr::Literal(Value::NUMBER(8.0))),
                operator: LexemeKind::Star,
                right: Box::new(Expr::Literal(Value::NUMBER(8.0))),
            })))
        );
    }

    #[test]
    fn it_errors() {
        let tokens = Scanner::new("print".to_owned()).collect();
        let mut p = Parser::new(tokens);
        let res = parse(&mut p);
        assert_eq!(res, Some(Stmt::Error { line: 0, message: "Unfinished print statement".to_string() }));
    }

    #[test]
    fn it_doesnt_panick_unfinished() {
        let tokens = Scanner::new("print(".to_owned()).collect();
        let mut p = Parser::new(tokens);
        let res = parse(&mut p);
        assert_eq!(res, Some(Stmt::Error { line: 0, message: "Unfinished print statement".to_string() }));
    }

    #[test]
    fn it_works_partial_stmts() {
        let tokens = Scanner::new("var a;".to_owned()).collect();
        let mut p = Parser::new(tokens);
        let res = parse(&mut p);
        assert_eq!(res, Some(Stmt::VariableDef { ident: "a".to_string(), expr: None }));

        let tokens = Scanner::new("var  a;".to_owned()).collect();
        let mut p = Parser::new(tokens);
        let res = parse(&mut p);
        assert_eq!(res, Some(Stmt::VariableDef { ident: "a".to_string(), expr: None }));
    }

    #[test]
    fn it_works_stmts() {
        let tokens = Scanner::new("var a = \"foo\";".to_owned()).collect();
        let mut p = Parser::new(tokens);
        let res = parse(&mut p);
        assert_eq!(res, Some(Stmt::VariableDef { ident: "a".to_string(), expr: Some(Expr::Literal(Value::STRING("foo".to_string()))) }));

        let tokens = Scanner::new("var a  =  \"foo\";".to_owned()).collect();
        let mut p = Parser::new(tokens);
        let res = parse(&mut p);
        assert_eq!(res, Some(Stmt::VariableDef { ident: "a".to_string(), expr: Some(Expr::Literal(Value::STRING("foo".to_string()))) }));

        let tokens = Scanner::new("var a  = 2*8;".to_owned()).collect();
        let mut p = Parser::new(tokens);
        let res = parse(&mut p);
        assert_eq!(
            res,
            Some(Stmt::VariableDef {
                ident: "a".to_string(),
                expr: Some(Expr::Binary {
                    left: Box::new(Expr::Literal(Value::NUMBER(2.0))),
                    operator: LexemeKind::Star,
                    right: Box::new(Expr::Literal(Value::NUMBER(8.0))),
                })
            })
        );
    }

    #[test]
    fn it_works_multiline() {
        let tokens = Scanner::new("var a = 2;
print(a);".to_owned()).collect();
        let mut p = Parser::new(tokens);
        let res = parse(&mut p);
        assert_eq!(res, Some(Stmt::VariableDef { ident: "a".to_string(), expr: Some(Expr::Literal(Value::NUMBER(2.0)))}));
    }

    #[test]
    fn it_errors_expression_l_value() {
        let tokens = Scanner::new("a + b = 2".to_owned()).collect();
        let mut p = Parser::new(tokens);
        let res = parse(&mut p);
        // error in parser expr
        assert_eq!(res, Some(Stmt::Expr(Expr::Error { line: 0, message: "Invalid left hand assignment expression".to_string() })));
    }

    #[test]
    fn it_errors_stmt() {
        let tokens = Scanner::new("var a =".to_owned()).collect();
        let mut p = Parser::new(tokens);
        let res = parse(&mut p);
        assert_eq!(res, Some(Stmt::Error { line: 0, message: "Unfinished right hand assignment".to_string() }));
    }

    #[test]
    fn it_works_block_no_spaces() {
        let tokens = Scanner::new("{var a = 2; print(a);}".to_owned()).collect();
        let mut p = Parser::new(tokens);
        let res = parse(&mut p);
        assert_eq!(
            res,
            Some(
                Stmt::Block(
                    Box::new(
                        vec![
                            Stmt::VariableDef { ident: "a".to_string(), expr: Some(Expr::Literal(Value::NUMBER(2.0))) },
                            Stmt::Print(Some(Expr::Variable("a".to_string()))),
                        ]
                    )
                )
            )
        );
    }

    #[test]
    fn it_works_block_spaces() {
        let tokens = Scanner::new("{
            var a = 2;
            print(a); }".to_owned()).collect();
        let mut p = Parser::new(tokens);
        let res = parse(&mut p);
        assert_eq!(
            res,
            Some(
                Stmt::Block(
                    Box::new(
                        vec![
                            Stmt::VariableDef { ident: "a".to_string(), expr: Some(Expr::Literal(Value::NUMBER(2.0))) },
                            Stmt::Print(Some(Expr::Variable("a".to_string()))),
                        ]
                    )
                )
            )
        );
    }
}
