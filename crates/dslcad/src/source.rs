use logos::Span;

pub trait LineColExt {
    fn line_col(&self, text: &str) -> (usize, Span);
}

impl LineColExt for Span {
    fn line_col(&self, text: &str) -> (usize, Span) {
        line_col(text, self)
    }
}

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
