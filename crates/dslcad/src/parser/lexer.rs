use logos::Logos;

pub type Lexer<'a> = logos::Lexer<'a, Token>;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
pub enum Token {
    #[token("=")]
    Equal,
    #[token("(")]
    OpenBracket,
    #[token(")")]
    CloseBracket,
    #[token("[")]
    OpenList,
    #[token("]")]
    CloseList,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token(".")]
    Period,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Multiply,
    #[token("/")]
    Divide,
    #[token("^")]
    Power,
    #[token("%")]
    Modulo,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEquals,
    #[token("==")]
    Equals,
    #[token("!=")]
    NotEquals,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEquals,

    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("not")]
    Not,

    #[token("->")]
    Inject,

    #[regex(r"true|false")]
    Bool,
    #[regex(r"\d+(\.\d*)?")]
    Number,
    #[regex("\"[^\"]*\"")]
    String,

    #[token("var")]
    Var,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex(r"(\.\.?)(/[a-zA-Z0-9_]+)+")]
    Path,

    #[error]
    #[regex(r"([ \t\n\r\f]+|//.*)", logos::skip)]
    Error,
}

#[cfg(test)]
mod tests {
    use super::Token::*;
    use super::*;

    #[test]
    fn it_can_lex_variable() {
        assert_eq!(vec![Var, Identifier, Equal, Number], tokens("var x = 5"));
        assert_eq!(vec![Var, Identifier], tokens("var x"));
        assert_eq!(vec![Var, Identifier, Equal, Bool], tokens("var x = true"));
    }

    #[test]
    fn it_can_lex_with_crlf() {
        assert_eq!(vec![Var, Identifier], tokens("var\r\nx"));
    }

    #[test]
    fn it_can_lex_calls() {
        assert_eq!(
            vec![
                Identifier,
                OpenBracket,
                Identifier,
                Equal,
                Number,
                CloseBracket
            ],
            tokens("cube(x=10)")
        );
        assert_eq!(
            vec![Identifier, OpenBracket, CloseBracket],
            tokens("cube()")
        );
        assert_eq!(
            vec![Path, OpenBracket, CloseBracket],
            tokens("./test/cube()")
        );
        assert_eq!(
            vec![Path, OpenBracket, CloseBracket],
            tokens("../test/cube()")
        );
    }

    #[test]
    fn it_can_lex_ref_div() {
        assert_eq!(vec![Identifier, Divide, Number], tokens("test/4"));
    }

    #[test]
    fn it_can_lex_lists() {
        assert_eq!(vec![OpenList, Identifier, CloseList], tokens("[test]"));
    }

    fn tokens(input: &str) -> Vec<Token> {
        Token::lexer(input).into_iter().collect()
    }
}