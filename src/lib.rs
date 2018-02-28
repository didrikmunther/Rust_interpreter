pub mod lexer;
pub mod parser;

pub fn exec(query: &str) -> Option<String> {
  let lexed = lexer::lex(query).unwrap();
  let parsed = parser::parse(&lexed);

  Some(String::from("temp"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
