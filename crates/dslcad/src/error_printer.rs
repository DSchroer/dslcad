use crate::parser::{ParseError, Reader};
use crate::runtime::{RuntimeError, WithStack};
use crate::source::LineColExt;
use std::error::Error;
use std::io::{Result, Write};

pub struct ErrorPrinter<R: Reader> {
    reader: R,
}

impl<R: Reader> ErrorPrinter<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn print_parse_error(&self, mut w: impl Write, parse_error: &ParseError) -> Result<()> {
        let source = self.reader.read(parse_error.file.to_path());
        write!(w, "error in {}", parse_error.file.to_str())?;
        if let Ok(source) = source {
            let (line, col) = parse_error.error.line_col(&source);
            write!(w, "[{}:{}-{}]", line, col.start, col.end)?;
        }
        writeln!(w, ":")?;

        writeln!(w, "{}", parse_error.error)?;
        Ok(())
    }

    pub fn print_runtime_error(
        &self,
        mut w: impl Write,
        runtime_error: &WithStack<RuntimeError>,
    ) -> Result<()> {
        writeln!(w, "error:")?;
        writeln!(w, "{}", runtime_error.error)?;
        writeln!(w)?;
        writeln!(w, "stacktrace:")?;

        for frame in runtime_error.stack.iter().rev() {
            let source = self.reader.read(frame.document.to_path());

            write!(w, "{}", frame.document.to_str())?;
            if let Ok(source) = source {
                let (line, col) = frame.span.line_col(&source);
                write!(w, "[{}:{}-{}]", line, col.start, col.end)?;
            }
            writeln!(w)?;
        }
        Ok(())
    }

    pub fn print_error(&self, mut w: impl Write, error: &impl Error) -> Result<()> {
        writeln!(w, "error:")?;
        writeln!(w, "{error}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{DocId, DocumentParseError};
    use crate::tests::TestReader;

    #[test]
    fn it_prints_standard_errors() {
        let mut buf = Vec::new();
        let printer = ErrorPrinter::new(TestReader("hello world"));
        printer.print_error(&mut buf, &std::fmt::Error).unwrap();

        assert_eq!(
            String::from_utf8(buf).unwrap(),
            r"error:
an error occurred when formatting an argument
"
        );
    }

    #[test]
    fn it_prints_parse_errors() {
        let mut buf = Vec::new();
        let printer = ErrorPrinter::new(TestReader("hello world"));
        printer
            .print_parse_error(
                &mut buf,
                &ParseError {
                    file: DocId::new("test.ds".to_string()),
                    error: DocumentParseError::Expected("test", "foo".to_string(), 5..10),
                },
            )
            .unwrap();

        assert_eq!(
            String::from_utf8(buf).unwrap(),
            r"error in test.ds[1:5-10]:
expected test but found foo
"
        );
    }
}
