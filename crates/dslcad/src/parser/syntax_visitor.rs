use crate::parser::{
    Expression, If, Index, Invocation, Literal, Map, NestedScope, Property, Reduce, Reference,
    Statement, Variable,
};
use crate::resources::Resource;
use logos::Span;
use std::sync::Arc;

pub trait StatementVisitor: Sized {
    type Result;

    fn visit_statement(&mut self, e: &Statement) -> Self::Result {
        e.walk_statement(self)
    }
    fn visit_variable(&mut self, variable: &Variable, _span: &Span) -> Self::Result;
    fn visit_create_part(&mut self, expr: &Expression, _span: &Span) -> Self::Result;
}

pub trait ExpressionVisitor: Sized {
    type Result;

    fn visit_expression(&mut self, e: &Expression) -> Self::Result {
        e.walk_expression(self)
    }
    fn visit_literal(&mut self, l: &Literal, s: &Span) -> Self::Result;
    fn visit_reference(&mut self, l: &Reference, s: &Span) -> Self::Result;
    fn visit_invocation(&mut self, l: &Invocation, s: &Span) -> Self::Result;
    fn visit_property(&mut self, l: &Property, s: &Span) -> Self::Result;
    fn visit_index(&mut self, l: &Index, s: &Span) -> Self::Result;
    fn visit_map(&mut self, l: &Map, s: &Span) -> Self::Result;
    fn visit_reduce(&mut self, l: &Reduce, s: &Span) -> Self::Result;
    fn visit_if(&mut self, l: &If, s: &Span) -> Self::Result;
    fn visit_scope(&mut self, l: &NestedScope, s: &Span) -> Self::Result;
}

pub trait LiteralVisitor: Sized {
    type Result;

    fn visit_literal(&mut self, e: &Literal) -> Self::Result {
        e.walk_literal(self)
    }
    fn visit_number(&mut self, v: &f64) -> Self::Result;
    fn visit_bool(&mut self, v: &bool) -> Self::Result;
    fn visit_text(&mut self, v: &str) -> Self::Result;
    fn visit_list(&mut self, v: &[Expression]) -> Self::Result;
    fn visit_resource(&mut self, v: &Arc<dyn Resource>) -> Self::Result;
    fn visit_function(&mut self, v: &[Statement]) -> Self::Result;
}
