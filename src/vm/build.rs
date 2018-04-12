use vm::*;

use lexer::Literal;

pub use parser::{Declaration, Statement, Expression, Primary};

#[derive(Debug)]
pub enum VMBuildError {
  // error, pos
  InvalidExpression(String, i32),
  UnsupportedOperator(Token, i32),
  UnsupportedType(Literal, i32),

  Temp
}

pub struct VMBuild {
  program: Vec<u8>,
  is_debug: bool
}

impl VMBuild {
  pub fn new() -> Self {
    Self {
      program: Vec::new(),
      is_debug: false
    }
  }

  fn build_binary(&mut self, expr: &Expression, pos: i32) -> Result<Vec<u8>, VMBuildError> {
    match expr {
      &Expression::Binary(ref left, ref token, ref right) => {
        let left_pos = match **left {
          Expression::Primary(_, pos) => pos,
          _ => 0
        };
        let right_pos = match **right {
          Expression::Primary(_, pos) => pos,
          _ => 0
        };

        let mut left = self.build_binary(&*left, left_pos)?;
        let mut right = self.build_binary(&*right, right_pos)?;

        left.append(&mut right);

        left.push(u(match token {
          &(Token::Plus, _) => ADD,
          &(Token::Minus, _) => SUB,
          &(Token::Asterix, _) => MULTIPLY,
          &(Token::Equals, _) => ASSIGN,
          &(Token::Slash, _) => DIVIDE,
          
          &(Token::Lt, _) => LT,
          &(Token::Gt, _) => GT,
          &(Token::LtOrEquals, _) => LTOREQ,
          &(Token::GtOrEquals, _) => GTOREQ,

          &(_, pos) => return Err(VMBuildError::UnsupportedOperator(token.0, pos))
        }));

        if self.is_debug {
          let pos = match token {
            &(_, pos) => pos
          };

          left.push(pos as u8); 
        }

        Ok(left)
      },
      &Expression::Primary(ref literal, pos) => {
        match literal {
          &Primary::Identifier(ref identifier) => {
            let mut v = vec![u(PUSH_STRING)];
            let mut s: Vec<u8> = identifier.clone().into_bytes();
            v.append(&mut s);
            v.push(u(NULL));
            v.push(u(PUSH_VAR));
            
            if self.is_debug {
              v.push(pos as u8);
            }

            Ok(v)
          },
          &Primary::Literal(ref literal) => {
            match literal {
              &lexer::Literal::Num(num) => {
                let bv: [u8; 8] = unsafe {
                  mem::transmute(num)
                };
                let mut bv: Vec<u8> = bv.to_vec();

                let mut v = vec![u(PUSH_NUM)];
                v.append(&mut bv);
                Ok(v)
              },
              &lexer::Literal::Bool(b) => {
                let v = vec![u(PUSH_BOOL), if b {0x01} else {0x00}];
                Ok(v)
              },
              &lexer::Literal::String(ref s) => {
                let mut v = vec![u(PUSH_STRING)];
                let mut s: Vec<u8> = s.clone().into_bytes();
                v.append(&mut s);
                v.push(u(NULL));
                Ok(v)
              },
              &lexer::Literal::Nil => {
                Ok(vec![u(PUSH_NIL)])
              },
              _ => return Err(VMBuildError::UnsupportedType(literal.clone(), pos))
            }
          }
        }
      },
      &Expression::PrintExpr(ref expr, pos) => {
        let expr_pos = match **expr {
          Expression::Primary(_, pos) => pos,
          _ => 0
        };

        let mut v = self.build_binary(&*expr, expr_pos)?;
        v.push(u(PRINT));

        if self.is_debug {
          v.push(pos as u8)
        }

        Ok(v)
      },
      _ => return Err(VMBuildError::InvalidExpression(format!("{:?}", expr), pos))
    }
  }

  fn build_expr(&mut self, expr: &Expression, pos: i32) -> Result<Vec<u8>, VMBuildError> {
    let binary = self.build_binary(expr, pos);
    binary
  }

  fn build_stmt(&mut self, stmt: &Statement) -> Result<Vec<u8>, VMBuildError> {
    match stmt {
      &Statement::ExpressionStmt(ref expr, is_statement, pos) => {
        let mut res = self.build_expr(expr, pos)?;
        if is_statement {
          res.push(u(POP));
        }
        Ok(res)
      }
    }
  }

  fn build_decl(&mut self, decl: &Declaration) -> Result<Vec<u8>, VMBuildError> {
    match decl {
      &Declaration::Statement(ref stmt, pos) => self.build_stmt(stmt)
    }
  }

  pub fn build(&mut self, decls: Decls, query: String, options: BuildOptions) -> Result<Instructions, VMBuildError> {
    let mut program: Vec<u8> = vec![u(VERSION), 0x01];

    if options.contains(BuildOptions::DEBUG) {
      self.is_debug = true;
      program.push(u(DEBUG))
    }

    if options.contains(BuildOptions::CODE) {
      program.push(u(DEBUG_CODE));
      let mut code = query.as_bytes().to_vec();
      program.append(&mut code);
      program.push(u(DEBUG_CODE_END));
    }

    program.push(u(META_END));

    for i in decls {
      let mut built = self.build_decl(&*i)?;
      program.append(&mut built);
    }
    
    program.push(u(END));

    Ok(program)
  }
}