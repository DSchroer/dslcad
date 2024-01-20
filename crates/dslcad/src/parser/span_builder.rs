use crate::parser::lexer::Lexer;
use logos::Span;

pub struct SpanBuilder {
    start: usize,
}

impl SpanBuilder {
    pub fn from(lexer: &Lexer) -> Self {
        SpanBuilder {
            start: lexer.span().start,
        }
    }

    pub fn to(self, lexer: &Lexer) -> Span {
        self.start..lexer.span().end
    }
}
