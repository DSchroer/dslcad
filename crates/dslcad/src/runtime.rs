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

use crate::cache::Cache;
use crate::resources::ResourceFactory;
use crate::runtime::stack::{Stack, StackFrame};
use crate::runtime::value::{Function, ViewValue};
use dslcad_storage::protocol::{
    Parameter, ParameterType, ParameterValue, Projection, ShowFlags, ViewAngles,
};
pub use runtime_error::RuntimeError;
pub use script_instance::ScriptInstance;

const MAX_STACK_SIZE: usize = 255;

pub struct Engine<'a> {
    library: &'a Library,
    ast: &'a Ast,
    stack: Stack,
    scope: Scope,
    current_document: Option<DocId>,
    cache: Option<&'a mut Cache>,
    /// Parameters declared at the root document, collected during evaluation.
    parameters: Vec<Parameter>,
}

impl<'a> Engine<'a> {
    pub fn new(library: &'a Library, ast: &'a Ast) -> Self {
        Engine {
            library,
            ast,
            stack: Stack::new(),
            scope: Scope::default(),
            current_document: None,
            cache: None,
            parameters: Vec::new(),
        }
    }

    pub fn with_cache(mut self, cache: Option<&'a mut Cache>) -> Self {
        self.cache = cache;
        self
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
        self.parameters.clear();
        let mut instance = self.with_scope(Scope::new(arguments), |e| e.eval(root))?;
        instance.set_parameters(std::mem::take(&mut self.parameters));
        Ok(instance.into())
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
        argument_values: Vec<ArgValue<'_>>,
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

/// Turns a declared parameter and its evaluated value into the protocol type
/// the editor displays, validating it against the declared metadata.
fn resolve_parameter(
    name: &str,
    spec: &ParameterSpec,
    value: &Value,
) -> Result<Parameter, RuntimeError> {
    let invalid = |message: String| RuntimeError::InvalidParameter {
        name: name.to_string(),
        message,
    };

    let kind = spec.kind.unwrap_or(match value {
        Value::Bool(_) => ParameterType::Bool,
        Value::Text(_) => ParameterType::Text,
        _ => ParameterType::Number,
    });

    let parameter_value = match kind {
        ParameterType::Number | ParameterType::Integer => {
            let number = value
                .to_number()
                .map_err(|_| invalid("must be a number".to_string()))?;
            if kind == ParameterType::Integer && number.fract() != 0.0 {
                return Err(invalid("must be a whole number".to_string()));
            }
            ParameterValue::Number(number)
        }
        ParameterType::Bool => ParameterValue::Bool(
            value
                .to_bool()
                .map_err(|_| invalid("must be true or false".to_string()))?,
        ),
        ParameterType::Text => ParameterValue::Text(
            value
                .to_text()
                .map_err(|_| invalid("must be text".to_string()))?,
        ),
    };

    if let ParameterValue::Number(number) = &parameter_value {
        if let Some(min) = spec.min {
            if *number < min {
                return Err(invalid(format!("must be at least {min}")));
            }
        }
        if let Some(max) = spec.max {
            if *number > max {
                return Err(invalid(format!("must be at most {max}")));
            }
        }
    }

    Ok(Parameter {
        name: name.to_string(),
        kind,
        value: parameter_value,
        min: spec.min,
        max: spec.max,
        step: spec.step,
    })
}

impl StatementVisitor for Engine<'_> {
    type Result = Result<Option<Value>, WithStack<RuntimeError>>;

    fn visit_variable(
        &mut self,
        Variable {
            value,
            name,
            parameter,
        }: &Variable,
        _span: &Span,
    ) -> Self::Result {
        let resolved = match value {
            Some(value) => {
                if let Some(value) = self.scope.get(name.as_str()).cloned() {
                    value
                } else {
                    value.walk_expression(self)?
                }
            }
            None => {
                if let Some(value) = self.scope.get(name.as_str()).cloned() {
                    value
                } else {
                    return Err(WithStack::from_err(
                        RuntimeError::UnsetParameter(name.to_string()),
                        &self.stack,
                    ));
                }
            }
        };

        if let Some(spec) = parameter {
            if self.current_document.as_ref() == Some(self.ast.root()) {
                let parameter = resolve_parameter(name, spec, &resolved)
                    .map_err(|e| WithStack::from_err(e, &self.stack))?;
                self.parameters.push(parameter);
            }
        }

        self.scope.set(name.to_string(), resolved);
        Ok(None)
    }

    fn visit_create_part(&mut self, expr: &Expression, _span: &Span) -> Self::Result {
        Ok(Some(self.visit_expression(expr)?))
    }

    fn visit_view(&mut self, view: &View, _span: &Span) -> Self::Result {
        let argument_values = view
            .arguments
            .iter()
            .try_fold(Vec::new(), |mut acc, argument| {
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

        let arguments = Engine::named_argument_values(argument_values)
            .map_err(|e| WithStack::from_err(e, &self.stack))?;

        let camera = parse_view_camera(view.name.clone(), &arguments)
            .map_err(|e| WithStack::from_err(e, &self.stack))?;

        let document = self.current_document.as_ref().map(|d| d.to_string());
        let instance = self.eval_statements(DocId::new_with_path("view", document), &view.body)?;
        let layers = instance.value().flatten().into_iter().cloned().collect();

        Ok(Some(Value::View(Rc::new(ViewValue {
            name: camera.name,
            angle: camera.angle,
            projection: camera.projection,
            zoom: camera.zoom,
            target: camera.target,
            fit: camera.fit,
            show: camera.show,
            layers,
        }))))
    }
}

struct ViewCamera {
    name: Option<String>,
    angle: ViewAngles,
    projection: Projection,
    zoom: Option<f32>,
    target: Option<[f64; 3]>,
    fit: bool,
    show: ShowFlags,
}

fn parse_view_camera(
    default_name: Option<String>,
    arguments: &HashMap<&str, Value>,
) -> Result<ViewCamera, RuntimeError> {
    let mut camera = ViewCamera {
        name: default_name,
        angle: ViewAngles::default(),
        projection: Projection::Perspective,
        zoom: None,
        target: None,
        fit: true,
        show: ShowFlags::default(),
    };

    for (key, value) in arguments {
        match *key {
            "name" => camera.name = Some(value.to_text()?),
            "angle" => camera.angle = parse_view_angles(&value.to_text()?)?,
            "projection" => camera.projection = parse_projection(&value.to_text()?)?,
            "zoom" => camera.zoom = Some(value.to_number()? as f32),
            "target" => {
                let point = value.to_point()?;
                camera.target = Some([point.x(), point.y(), point.z()]);
            }
            "fit" => camera.fit = value.to_bool()?,
            "show" => camera.show = parse_show(value)?,
            other => {
                return Err(RuntimeError::UserDefined(format!(
                    "unknown view argument '{other}'"
                )))
            }
        }
    }

    Ok(camera)
}

fn parse_view_angles(input: &str) -> Result<ViewAngles, RuntimeError> {
    let text = input.trim().to_ascii_lowercase();

    match text.as_str() {
        "iso" => {
            return Ok(ViewAngles {
                x: Some(54.736),
                y: Some(45.0),
                z: Some(0.0),
            })
        }
        "top" => {
            // Rolled so the top view is a straight upward tilt from `front`:
            // the camera keeps the same heading and does not spin.
            return Ok(ViewAngles {
                x: Some(0.0),
                y: Some(0.0),
                z: Some(90.0),
            });
        }
        "bottom" => {
            return Ok(ViewAngles {
                x: Some(180.0),
                y: Some(0.0),
                z: Some(90.0),
            })
        }
        "front" => {
            return Ok(ViewAngles {
                x: Some(90.0),
                y: Some(0.0),
                ..Default::default()
            })
        }
        "back" => {
            return Ok(ViewAngles {
                x: Some(90.0),
                y: Some(180.0),
                ..Default::default()
            })
        }
        "right" => {
            return Ok(ViewAngles {
                x: Some(90.0),
                y: Some(90.0),
                ..Default::default()
            })
        }
        "left" => {
            return Ok(ViewAngles {
                x: Some(90.0),
                y: Some(-90.0),
                ..Default::default()
            })
        }
        "" => return Err(RuntimeError::UserDefined("empty view angle".to_string())),
        _ => {}
    }

    if text.starts_with(|c: char| c.is_ascii_digit() || matches!(c, '-' | '+' | '.')) {
        let y = text
            .parse()
            .map_err(|_| RuntimeError::UserDefined(format!("invalid view angle '{text}'")))?;
        return Ok(ViewAngles {
            y: Some(y),
            ..Default::default()
        });
    }

    let mut angles = ViewAngles::default();
    let bytes = text.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        let axis = bytes[index] as char;
        index += 1;

        let start = index;
        while index < bytes.len() && !bytes[index].is_ascii_alphabetic() {
            index += 1;
        }

        let value: f32 = text[start..index]
            .parse()
            .map_err(|_| RuntimeError::UserDefined(format!("invalid view angle '{text}'")))?;

        match axis {
            'x' => angles.x = Some(value),
            'y' => angles.y = Some(value),
            'z' => angles.z = Some(value),
            _ => {
                return Err(RuntimeError::UserDefined(format!(
                    "unknown view angle axis '{axis}'"
                )))
            }
        }
    }

    Ok(angles)
}

fn parse_projection(input: &str) -> Result<Projection, RuntimeError> {
    match input.trim().to_ascii_lowercase().as_str() {
        "perspective" => Ok(Projection::Perspective),
        "orthographic" | "ortho" => Ok(Projection::Orthographic),
        other => Err(RuntimeError::UserDefined(format!(
            "unknown projection '{other}'"
        ))),
    }
}

fn parse_show(value: &Value) -> Result<ShowFlags, RuntimeError> {
    let names = match value {
        Value::List(values) => values
            .iter()
            .map(|value| value.to_text())
            .collect::<Result<Vec<_>, _>>()?,
        value => vec![value.to_text()?],
    };

    let mut show = ShowFlags {
        model: false,
        points: false,
        lines: false,
        mesh: false,
        annotations: false,
    };

    for name in names {
        match name.as_str() {
            "model" => show.model = true,
            "points" => show.points = true,
            "lines" => show.lines = true,
            "mesh" => show.mesh = true,
            "annotations" => show.annotations = true,
            other => {
                return Err(RuntimeError::UserDefined(format!(
                    "unknown show flag '{other}'"
                )))
            }
        }
    }

    Ok(show)
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
                        let (f, a, signature) = self
                            .library
                            .find(CallSignature::new(name, argument_values))
                            .map_err(|e| WithStack::from_err(e, &self.stack))?;

                        if let Some(cache) = self.cache.as_deref_mut() {
                            let key = Cache::eval_key(signature, &a);
                            if let Some(cached) = cache.get_eval(&key) {
                                cached
                            } else {
                                let value =
                                    f(&a).map_err(|e| WithStack::from_err(e, &self.stack))?;
                                cache.insert_eval(
                                    key,
                                    value.clone(),
                                    a.values().cloned().collect(),
                                );
                                value
                            }
                        } else {
                            f(&a).map_err(|e| WithStack::from_err(e, &self.stack))?
                        }
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

    fn visit_resource(&mut self, factory: &ResourceFactory, _s: &Span) -> Self::Result {
        let mut arguments = HashMap::new();

        for argument in factory.arguments() {
            match argument {
                Argument::Named(name, expression) => {
                    let value = self.visit_expression(expression)?;
                    let literal = value.to_literal().ok_or_else(|| {
                        WithStack::from_err(
                            RuntimeError::InvalidResourceArgument(name.clone()),
                            &self.stack,
                        )
                    })?;
                    arguments.insert(name.clone(), literal);
                }
                Argument::Unnamed(_) => {
                    return Err(WithStack::from_err(
                        RuntimeError::InvalidResourceArgument("".to_string()),
                        &self.stack,
                    ))
                }
            }
        }

        let resource = factory
            .load(&arguments)
            .map_err(|e| WithStack::from_err(RuntimeError::from(e), &self.stack))?;

        resource
            .to_instance()
            .map_err(|e| WithStack::from_err(e, &self.stack))
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

    fn visit_function(&mut self, v: &Rc<Vec<Statement>>) -> Self::Result {
        Ok(Value::Function(Rc::new(Function::Defined {
            clojure: self.scope.clone(),
            statements: v.clone(),
        })))
    }
}
