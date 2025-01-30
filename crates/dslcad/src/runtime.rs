mod access;
mod output;
mod runtime_error;
mod scope;
mod script_instance;
mod stack;
mod types;
mod value;

use crate::library::{ArgValue, CallSignature, Library};
use crate::parser::*;
use crate::runtime::scope::Scope;
use log::trace;
use logos::Span;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use std::time::Instant;

pub use access::Access;
pub use stack::WithStack;
pub use types::Type;
pub use value::Value;

use crate::resources::Resource;
use crate::runtime::stack::{Stack, StackFrame};
use crate::runtime::value::Function;
pub use runtime_error::RuntimeError;
pub use script_instance::ScriptInstance;

const MAX_STACK_SIZE: usize = 255;

pub struct Engine<'a> {
    library: &'a Library,
    ast: &'a Ast,
    stack: Stack,
    scope: Scope,
    current_document: Option<DocId>,
}

impl<'a> Engine<'a> {
    pub fn new(library: &'a Library, ast: &'a Ast) -> Self {
        Engine {
            library,
            ast,
            stack: Stack::new(),
            scope: Scope::default(),
            current_document: None,
        }
    }

    pub fn eval_root(
        &mut self,
        arguments: HashMap<&'a str, Literal>,
    ) -> Result<Value, WithStack<RuntimeError>> {
        let root = self.ast.root().clone();
        let arguments = arguments
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    self.visit_expression(&Expression::Literal(v, Span::default()))
                        .unwrap(),
                )
            })
            .collect();
        Ok(self
            .with_scope(Scope::new(arguments), |e| e.eval(root))?
            .into())
    }

    fn eval(&mut self, id: DocId) -> Result<ScriptInstance, WithStack<RuntimeError>> {
        self.current_document = Some(id.clone());
        let statements = self.ast.documents.get(&id).ok_or_else(|| {
            WithStack::from_err(RuntimeError::UnknownIdentifier(id.to_string()), &self.stack)
        })?;

        self.eval_statements(id, statements)
    }

    fn with_scope<T>(&mut self, scope: Scope, f: impl FnOnce(&mut Self) -> T) -> T {
        let tmp = self.scope.clone();
        self.scope = scope;
        let ret = f(self);
        self.scope = tmp;
        ret
    }

    fn eval_statements(
        &mut self,
        id: DocId,
        statements: &[Statement],
    ) -> Result<ScriptInstance, WithStack<RuntimeError>> {
        let mut ret = Vec::new();

        for statement in statements {
            self.stack.push(StackFrame::from_statement(&id, statement));

            if self.stack.len() >= MAX_STACK_SIZE {
                return Err(WithStack::from_err(
                    RuntimeError::StackOverflow(),
                    &self.stack,
                ));
            }

            if let Some(v) = self.visit_statement(statement)? {
                ret.push(v);
            }

            self.stack.pop();
        }

        ScriptInstance::from_scope(ret, self.scope.clone())
            .map_err(|e| WithStack::from_err(e, &self.stack))
    }

    fn named_argument_values(
        argument_values: Vec<ArgValue>,
    ) -> Result<HashMap<&str, Value>, RuntimeError> {
        argument_values
            .into_iter()
            .try_fold(HashMap::new(), |mut acc, v| {
                match v {
                    ArgValue::Named(name, val) => {
                        acc.insert(name, val);
                    }
                    ArgValue::Unnamed(_) => {
                        return Err(RuntimeError::UnknownDefaultArgument {
                            name: "".to_string(),
                        })
                    }
                }
                Ok(acc)
            })
    }
}

impl StatementVisitor for Engine<'_> {
    type Result = Result<Option<Value>, WithStack<RuntimeError>>;

    fn visit_variable(
        &mut self,
        Variable { value, name }: &Variable,
        _span: &Span,
    ) -> Self::Result {
        match value {
            Some(value) => {
                let value = if let Some(value) = self.scope.get(name.as_str()).cloned() {
                    value
                } else {
                    value.walk_expression(self)?
                };
                self.scope.set(name.to_string(), value);
            }
            None => {
                let value = if let Some(value) = self.scope.get(name.as_str()).cloned() {
                    value
                } else {
                    return Err(WithStack::from_err(
                        RuntimeError::UnsetParameter(name.to_string()),
                        &self.stack,
                    ));
                };
                self.scope.set(name.to_string(), value);
            }
        }
        Ok(None)
    }

    fn visit_create_part(&mut self, expr: &Expression, _span: &Span) -> Self::Result {
        Ok(Some(self.visit_expression(expr)?))
    }
}

impl ExpressionVisitor for Engine<'_> {
    type Result = Result<Value, WithStack<RuntimeError>>;

    fn visit_literal(&mut self, l: &Literal, _s: &Span) -> Self::Result {
        l.walk_literal(self)
    }

    fn visit_reference(&mut self, l: &Reference, _s: &Span) -> Self::Result {
        let scope = self.scope.clone();

        if let Some(value) = scope.get(&l.name) {
            Ok(value.clone())
        } else if self.library.contains(&l.name) {
            Ok(Value::Function(Rc::new(Function::Builtin {
                name: l.name.to_string(),
            })))
        } else {
            Err(WithStack::from_err(
                RuntimeError::UnknownIdentifier(l.name.to_string()),
                &self.stack,
            ))
        }
    }

    fn visit_invocation(
        &mut self,
        Invocation { arguments, path }: &Invocation,
        _s: &Span,
    ) -> Self::Result {
        let argument_values = arguments.iter().try_fold(Vec::new(), |mut acc, argument| {
            match argument {
                Argument::Named(name, expr) => {
                    let value = self.visit_expression(expr.deref())?;
                    acc.push(ArgValue::Named(name, value))
                }
                Argument::Unnamed(expr) => {
                    let value = self.visit_expression(expr.deref())?;
                    acc.push(ArgValue::Unnamed(value));
                }
            }
            Ok(acc)
        })?;

        match path {
            CallPath::Function(path) => {
                let timer = Instant::now();

                let value = self.visit_expression(path)?;
                let func = value
                    .to_function()
                    .map_err(|e| WithStack::from_err(e, &self.stack))?;

                let res = match func.as_ref() {
                    Function::Builtin { name } => {
                        let (f, a) = self
                            .library
                            .find(CallSignature::new(name, argument_values))
                            .map_err(|e| WithStack::from_err(e, &self.stack))?;
                        f(&a).map_err(|e| WithStack::from_err(e, &self.stack))?
                    }
                    Function::Defined {
                        clojure,
                        statements,
                    } => {
                        let named_argument_values = Engine::named_argument_values(argument_values)
                            .map_err(|e| WithStack::from_err(e, &self.stack))?;

                        let document = self.current_document.as_ref().map(|d| d.to_string());
                        let mut scope = clojure.clone();
                        scope.set_arguments(named_argument_values);
                        self.with_scope(scope, |e| {
                            e.eval_statements(DocId::new_with_path("fn", document), statements)
                        })?
                        .into()
                    }
                };

                if timer.elapsed().as_millis() != 0 {
                    trace!(
                        "{:?}(..) executed in {}ms",
                        path,
                        timer.elapsed().as_millis()
                    );
                }
                Ok(res)
            }
            CallPath::Document(id) => {
                let named_argument_values = Engine::named_argument_values(argument_values)
                    .map_err(|e| WithStack::from_err(e, &self.stack))?;
                let v =
                    self.with_scope(Scope::new(named_argument_values), |e| e.eval(id.clone()))?;
                Ok(Value::Script(Rc::new(v)))
            }
        }
    }

    fn visit_property(&mut self, Property { name, target }: &Property, _s: &Span) -> Self::Result {
        let l = target.walk_expression(self)?;

        let script = l
            .to_accessible()
            .map_err(|e| WithStack::from_err(e, &self.stack))?;
        match script.get(name) {
            None => Err(WithStack::from_err(
                RuntimeError::MissingProperty(name.to_owned()),
                &self.stack,
            )),
            Some(v) => Ok(v),
        }
    }

    fn visit_index(&mut self, l: &Index, _s: &Span) -> Self::Result {
        let target_value = self.visit_expression(&l.target)?;
        let index_value = self.visit_expression(&l.index)?;

        let list = target_value
            .to_list()
            .map_err(|e| WithStack::from_err(e, &self.stack))?;
        let index = index_value
            .to_number()
            .map_err(|e| WithStack::from_err(e, &self.stack))?;
        Ok(list[index.round() as usize].clone())
    }

    fn visit_map(&mut self, l: &Map, _s: &Span) -> Self::Result {
        let scope = self.scope.clone();

        let range_value = self.visit_expression(&l.range)?;
        let range_value = range_value
            .to_list()
            .map_err(|e| WithStack::from_err(e, &self.stack))?;

        let mut loop_scope = scope.clone();
        let mut results = Vec::new();
        for v in range_value {
            loop_scope.set(l.identifier.to_string(), v.clone());
            let eval = self.with_scope(loop_scope.clone(), |e| e.visit_expression(&l.action))?;
            results.push(eval);
        }

        Ok(Value::List(results))
    }

    fn visit_reduce(&mut self, l: &Reduce, _s: &Span) -> Self::Result {
        let scope = self.scope.clone();

        let range_value = self.visit_expression(&l.range)?;
        let mut range_value = range_value
            .to_list()
            .map_err(|e| WithStack::from_err(e, &self.stack))?
            .clone();
        range_value.reverse();

        let mut loop_scope = scope.clone();
        let mut result = if let Some(expr) = &l.root {
            self.visit_expression(expr)?
        } else {
            range_value
                .pop()
                .ok_or(RuntimeError::EmptyReduce())
                .map_err(|e| WithStack::from_err(e, &self.stack))?
        };
        for v in range_value.into_iter().rev() {
            loop_scope.set(l.left.to_string(), result.clone());
            loop_scope.set(l.right.to_string(), v.clone());
            result = self.with_scope(loop_scope.clone(), |e| e.visit_expression(&l.action))?;
        }

        Ok(result)
    }

    fn visit_if(&mut self, l: &If, _s: &Span) -> Self::Result {
        let condition_value = self.visit_expression(&l.condition)?;
        if condition_value
            .to_bool()
            .map_err(|e| WithStack::from_err(e, &self.stack))?
        {
            Ok(self.visit_expression(&l.if_true)?)
        } else {
            Ok(self.visit_expression(&l.if_false)?)
        }
    }

    fn visit_scope(&mut self, l: &NestedScope, _s: &Span) -> Self::Result {
        let document = self.current_document.as_ref().map(|d| d.to_string());
        let inst = self.eval_statements(DocId::new_with_path("scope", document), &l.statements)?;
        Ok(Value::Script(Rc::new(inst)))
    }
}

impl LiteralVisitor for Engine<'_> {
    type Result = Result<Value, WithStack<RuntimeError>>;

    fn visit_number(&mut self, v: &f64) -> Self::Result {
        Ok(Value::Number(*v))
    }

    fn visit_bool(&mut self, v: &bool) -> Self::Result {
        Ok(Value::Bool(*v))
    }

    fn visit_text(&mut self, v: &str) -> Self::Result {
        Ok(Value::Text(v.to_string()))
    }

    fn visit_list(&mut self, v: &[Expression]) -> Self::Result {
        Ok(Value::List(
            v.iter()
                .map(|v| self.visit_expression(v))
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }

    fn visit_resource(&mut self, v: &dyn Resource) -> Self::Result {
        v.to_instance()
            .map_err(|e| WithStack::from_err(e, &self.stack))
    }

    fn visit_function(&mut self, v: &Rc<Vec<Statement>>) -> Self::Result {
        Ok(Value::Function(Rc::new(Function::Defined {
            clojure: self.scope.clone(),
            statements: v.clone(),
        })))
    }
}
