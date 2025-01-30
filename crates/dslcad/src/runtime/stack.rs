use crate::parser::{DocId, Statement};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Range;

pub type Stack = Vec<StackFrame>;
type Span = Range<usize>;

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub document: DocId,
    pub span: Span,
}

impl StackFrame {
    pub fn from_statement(document: &DocId, statement: &Statement) -> Self {
        StackFrame {
            document: document.clone(),
            span: statement.span().clone(),
        }
    }
}

#[derive(Debug)]
pub struct WithStack<T: Error> {
    pub error: T,
    pub stack: Vec<StackFrame>,
}

impl<T: Error> WithStack<T> {
    pub fn from_err(error: T, stack: &Stack) -> Self {
        WithStack {
            error,
            stack: stack.clone(),
        }
    }
}

impl<T: Error> Display for WithStack<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.error, f)
    }
}

impl<T: Error> Error for WithStack<T> {}
