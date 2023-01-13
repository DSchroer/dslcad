use crate::parser::lexer::Lexer;
use crate::parser::Expression;
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

    pub fn from_expr(expr: &Expression) -> Self {
        SpanBuilder {
            start: expr.span().start,
        }
    }

    pub fn to(self, lexer: &Lexer) -> Span {
        self.start..lexer.span().end
    }
}
