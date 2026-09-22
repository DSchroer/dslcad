use crate::runtime::Value;
use dslcad_storage::protocol::Part;
use indexmap::IndexMap;
use log::warn;
use std::collections::HashMap;
use std::rc::Rc;

/// How many evaluated calls to keep. Larger values trade memory for fewer
/// recomputations while iterating on a model.
const EVAL_CAPACITY: usize = 2048;

/// How many meshed parts to keep. Meshes are much larger than the shapes they
/// come from, so this is deliberately smaller than [`EVAL_CAPACITY`].
const RENDER_CAPACITY: usize = 256;

/// A stable identity for a [`Value`], used to build cache keys.
///
/// Scalars and points are identified by their value, so equivalent results are
/// reused even when they are recomputed from scratch. Geometry is identified by
/// the address of its `Rc`; every cache entry keeps the values referenced by its
/// key alive, so a live address cannot be reused while a key still refers to it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Fingerprint {
    Number(u64),
    Bool(bool),
    Text(String),
    Point(u64, u64, u64),
    Line(usize),
    Plane(usize),
    Shape(usize),
    List(Vec<Fingerprint>),
    Function(usize),
    Script(usize),
}

fn address<T>(value: &Rc<T>) -> usize {
    Rc::as_ptr(value) as *const () as usize
}

impl Fingerprint {
    fn of(value: &Value) -> Self {
        match value {
            Value::Number(number) => Fingerprint::Number(number.to_bits()),
            Value::Bool(boolean) => Fingerprint::Bool(*boolean),
            Value::Text(text) => Fingerprint::Text(text.clone()),
            Value::Point(point) => Fingerprint::Point(
                point.x().to_bits(),
                point.y().to_bits(),
                point.z().to_bits(),
            ),
            Value::Line(wire) => Fingerprint::Line(address(wire)),
            Value::Plane(wire) => Fingerprint::Plane(address(wire)),
            Value::Shape(shape) => Fingerprint::Shape(address(shape)),
            Value::List(values) => Fingerprint::List(values.iter().map(Fingerprint::of).collect()),
            Value::Function(function) => Fingerprint::Function(address(function)),
            Value::Script(script) => Fingerprint::Script(address(script)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct EvalKey {
    signature: usize,
    arguments: Vec<(String, Fingerprint)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RenderKey {
    value: Fingerprint,
    deflection: u64,
}

struct EvalEntry {
    result: Value,
    /// Kept alive so the addresses in `EvalKey` stay valid; never read directly.
    _arguments: Vec<Value>,
}

struct RenderEntry {
    /// Kept alive so the addresses in `RenderKey` stay valid; never read directly.
    _value: Value,
    part: Part,
}

/// An in-memory cache of evaluated and meshed values.
///
/// The engine is rebuilt on every evaluation, so a cache is passed in by the
/// caller and reused across evaluations. In the preview window this means
/// editing one parameter only recomputes the parts of the model that actually
/// changed.
pub struct Cache {
    eval: IndexMap<EvalKey, EvalEntry>,
    render: IndexMap<RenderKey, RenderEntry>,
    hits: usize,
    misses: usize,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            eval: IndexMap::new(),
            render: IndexMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    /// The number of cache hits since the cache was created.
    pub fn hits(&self) -> usize {
        self.hits
    }

    /// The number of cache misses since the cache was created.
    pub fn misses(&self) -> usize {
        self.misses
    }

    pub(crate) fn eval_key(signature: usize, arguments: &HashMap<&str, Value>) -> EvalKey {
        let mut arguments: Vec<(String, Fingerprint)> = arguments
            .iter()
            .map(|(name, value)| ((*name).to_string(), Fingerprint::of(value)))
            .collect();
        arguments.sort_by(|left, right| left.0.cmp(&right.0));
        EvalKey {
            signature,
            arguments,
        }
    }

    pub(crate) fn render_key(value: &Value, deflection: f64) -> RenderKey {
        RenderKey {
            value: Fingerprint::of(value),
            deflection: deflection.to_bits(),
        }
    }

    pub(crate) fn get_eval(&mut self, key: &EvalKey) -> Option<Value> {
        let entry = self.eval.shift_remove(key)?;
        let result = entry.result.clone();
        self.eval.insert(key.clone(), entry);
        self.hits += 1;
        Some(result)
    }

    pub(crate) fn insert_eval(&mut self, key: EvalKey, result: Value, arguments: Vec<Value>) {
        self.misses += 1;
        self.eval.insert(
            key,
            EvalEntry {
                result,
                _arguments: arguments,
            },
        );
        let mut evicted = 0;
        while self.eval.len() > EVAL_CAPACITY {
            self.eval.shift_remove_index(0);
            evicted += 1;
        }
        if evicted > 0 {
            warn!("eval cache full, evicted {evicted} entries (capacity {EVAL_CAPACITY})");
        }
    }

    pub(crate) fn get_render(&mut self, key: &RenderKey) -> Option<Part> {
        let entry = self.render.shift_remove(key)?;
        let part = entry.part.clone();
        self.render.insert(key.clone(), entry);
        self.hits += 1;
        Some(part)
    }

    pub(crate) fn insert_render(&mut self, key: RenderKey, value: Value, part: Part) {
        self.misses += 1;
        self.render.insert(
            key,
            RenderEntry {
                _value: value,
                part,
            },
        );
        let mut evicted = 0;
        while self.render.len() > RENDER_CAPACITY {
            self.render.shift_remove_index(0);
            evicted += 1;
        }
        if evicted > 0 {
            warn!("render cache full, evicted {evicted} entries (capacity {RENDER_CAPACITY})");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprints_scalars_by_value() {
        assert_eq!(
            Fingerprint::of(&Value::Number(1.5)),
            Fingerprint::of(&Value::Number(1.5))
        );
        assert_ne!(
            Fingerprint::of(&Value::Number(1.5)),
            Fingerprint::of(&Value::Number(2.5))
        );
        assert_eq!(
            Fingerprint::of(&Value::Text("a".to_string())),
            Fingerprint::of(&Value::Text("a".to_string()))
        );
    }

    #[test]
    fn eval_keys_are_order_independent() {
        let mut left = HashMap::new();
        left.insert("a", Value::Number(1.0));
        left.insert("b", Value::Number(2.0));

        let mut right = HashMap::new();
        right.insert("b", Value::Number(2.0));
        right.insert("a", Value::Number(1.0));

        assert_eq!(Cache::eval_key(0, &left), Cache::eval_key(0, &right));
    }

    #[test]
    fn eval_keys_depend_on_signature_and_arguments() {
        let mut arguments = HashMap::new();
        arguments.insert("a", Value::Number(1.0));

        assert_ne!(
            Cache::eval_key(0, &arguments),
            Cache::eval_key(1, &arguments)
        );

        let mut other = HashMap::new();
        other.insert("a", Value::Number(2.0));
        assert_ne!(Cache::eval_key(0, &arguments), Cache::eval_key(0, &other));
    }

    #[test]
    fn it_caches_eval_results() {
        let mut cache = Cache::new();
        let mut arguments = HashMap::new();
        arguments.insert("a", Value::Number(1.0));

        let key = Cache::eval_key(0, &arguments);
        cache.insert_eval(key.clone(), Value::Number(42.0), vec![Value::Number(1.0)]);

        let hit = cache.get_eval(&key).unwrap();
        assert_eq!(Ok(42.0), hit.to_number());
        assert_eq!(1, cache.hits());
    }
}
