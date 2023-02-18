use crate::parser::{Document, Statement};
use logos::{Source, Span};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Write};

pub type Stack<'a> = Vec<StackFrame<'a>>;

#[derive(Debug)]
pub struct StackFrame<'a> {
    document: &'a Document<'a>,
    span: Span,
}

impl<'a> StackFrame<'a> {
    pub fn from_statement(document: &'a Document, statement: &'a Statement) -> Self {
        StackFrame {
            document,
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
            let (line, _) = line_col(frame.document.source(), &frame.span);
            writeln!(
                trace,
                "{}[{}]: {}",
                frame.document.id(),
                line,
                frame.document.source().slice(frame.span.clone()).unwrap()
            )
            .unwrap();
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

fn line_col(text: &str, span: &Span) -> (usize, Span) {
    let mut target = span.clone();
    for (i, line) in text.split('\n').enumerate() {
        let len = line.len();
        if target.start > len {
            target.start -= len + 1;
            target.end -= len + 1;
        } else {
            return (i + 1, target);
        }
    }
    (1, target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tests::TestReader;
    use crate::parser::SourceStore;
    use crate::runtime::RuntimeError;
    use std::collections::HashSet;

    #[test]
    fn it_can_print_stacks() {
        let mut stack = Stack::new();
        let store = SourceStore::new(Box::new(TestReader("")));
        let id = store.forge_id("test".to_string()).unwrap();
        let doc = Document::new(
            id,
            "var foo = bar;",
            HashSet::new(),
            vec![Statement::Variable {
                name: "",
                value: None,
                span: 0..3,
            }],
        );
        stack.push(StackFrame::from_statement(
            &doc,
            doc.statements().next().unwrap(),
        ));
        println!(
            "{}",
            WithStack::from_err(RuntimeError::NoReturnValue(), &stack)
        )
    }
}
