//! The `json` module.
//!
//! `v0.1.0` predeclares `go.starlark.net/lib/json`, which gives
//! `json.decode`, `json.encode` and `json.indent`. starlark-rust has a `json`
//! *function* instead, which is a different thing under the same name, so the
//! module is built here.
//!
//! It is not decoration: the standard library's package-manager components
//! call `json.decode(result.stdout)` to read what a manager reported, and a
//! configuration that cannot do that cannot interrogate anything; see
//! [R-STAR-001].

use std::fmt;

use allocative::Allocative;
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::starlark_module;
use starlark::values::{
    Heap, NoSerialize, ProvidesStaticType, StarlarkValue, Value, starlark_value,
};

/// The `json` module value.
#[derive(Debug, Clone, ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct JsonModule;

// A unit value with no heap references, so it can be allocated frozen and
// handed out as a global constant.
starlark::starlark_simple_value!(JsonModule);

impl fmt::Display for JsonModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("json")
    }
}

#[starlark_value(type = "json")]
impl<'v> StarlarkValue<'v> for JsonModule {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("json", json_methods);
        Some(RES.methods())
    }
}

/// Converts parsed JSON into a Starlark value.
fn to_starlark<'v>(value: &serde_json::Value, heap: Heap<'v>) -> Value<'v> {
    match value {
        serde_json::Value::Null => Value::new_none(),
        serde_json::Value::Bool(b) => Value::new_bool(*b),
        serde_json::Value::Number(n) => n
            .as_i64()
            .map_or_else(|| heap.alloc(n.as_f64().unwrap_or(0.0)), |i| heap.alloc(i)),
        serde_json::Value::String(s) => heap.alloc(s.as_str()),
        serde_json::Value::Array(items) => heap.alloc(
            items
                .iter()
                .map(|i| to_starlark(i, heap))
                .collect::<Vec<_>>(),
        ),
        serde_json::Value::Object(fields) => {
            let mut dict = starlark::collections::SmallMap::new();
            for (key, field) in fields {
                dict.insert_hashed(
                    heap.alloc_str(key)
                        .to_value()
                        .get_hashed()
                        .unwrap_or_else(|_| unreachable!("a string is hashable")),
                    to_starlark(field, heap),
                );
            }
            heap.alloc(starlark::values::dict::Dict::new(dict))
        }
    }
}

#[starlark_module]
fn json_methods(builder: &mut MethodsBuilder) {
    /// Parses JSON text into a value.
    fn decode<'v>(
        this: Value<'v>,
        #[starlark(require = pos)] text: String,
        heap: Heap<'v>,
    ) -> anyhow::Result<Value<'v>> {
        let _ = this;
        let parsed: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| anyhow::anyhow!("json.decode: {e}"))?;
        Ok(to_starlark(&parsed, heap))
    }

    /// Renders a value as JSON.
    fn encode<'v>(
        this: Value<'v>,
        #[starlark(require = pos)] value: Value<'v>,
    ) -> anyhow::Result<String> {
        let _ = this;
        value
            .to_json()
            .map_err(|e| anyhow::anyhow!("json.encode: {e}"))
    }

    /// Re-renders JSON text with indentation.
    fn indent<'v>(
        this: Value<'v>,
        #[starlark(require = pos)] text: String,
        prefix: Option<String>,
        indent: Option<String>,
    ) -> anyhow::Result<String> {
        let _ = this;
        let parsed: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| anyhow::anyhow!("json.indent: {e}"))?;

        let indent = indent.unwrap_or_else(|| "\t".to_owned());
        let mut out = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
        let mut serializer = serde_json::Serializer::with_formatter(&mut out, formatter);
        serde::Serialize::serialize(&parsed, &mut serializer)
            .map_err(|e| anyhow::anyhow!("json.indent: {e}"))?;

        let rendered = String::from_utf8(out).map_err(|e| anyhow::anyhow!("json.indent: {e}"))?;
        Ok(match prefix {
            Some(prefix) if !prefix.is_empty() => rendered
                .lines()
                .enumerate()
                .map(|(i, line)| {
                    if i == 0 {
                        line.to_owned()
                    } else {
                        format!("{prefix}{line}")
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
            _ => rendered,
        })
    }
}
