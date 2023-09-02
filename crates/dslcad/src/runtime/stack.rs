use crate::parser::{DocId, Statement};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Write};
use std::ops::Range;

pub type Stack = Vec<StackFrame>;
type Span = Range<usize>;

#[derive(Debug)]
pub struct StackFrame {
    document: DocId,
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
    error: T,
    trace: String,
}

impl<T: Error> WithStack<T> {
    pub fn from_err(error: T, stack: &Stack) -> Self {
        let mut trace = String::new();

        for frame in stack.iter().rev() {
            // let (line, _) = line_col(frame.document.source(), &frame.span);
            writeln!(trace, "{}[{}]:", frame.document, 0,).unwrap();
        }

        WithStack { error, trace }
    }
}

impl<T: Error> Display for WithStack<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.error)?;
        writeln!(f, "--- STACKTRACE ---")?;
        write!(f, "{}", self.trace)?;
        Ok(())
    }
}

impl<T: Error> Error for WithStack<T> {}
