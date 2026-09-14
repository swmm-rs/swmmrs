//! Shared JavaScript decoding, serialization, and identity helpers.

use js_sys::{Array, Error, Object, Reflect};
use serde::Serialize;
use serde::de::DeserializeOwned;
use swmmrs::engine::enums::ObjectType;
use swmmrs::engine::error::{ErrorCode, SwmmError};
use wasm_bindgen::{JsCast, JsValue};

use super::super::Simulation;

pub(super) fn non_null<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

pub(super) fn decode<T: DeserializeOwned>(value: JsValue, operation: &str) -> Result<T, JsValue> {
    if value.is_null() || value.is_undefined() || !value.is_object() || Array::is_array(&value) {
        return Err(invalid_patch(operation, "patch must be a non-null object"));
    }

    decode_value(value, operation)
}

/// Materialize nested maps so serde rejects unknown fields at every record boundary.
pub(crate) fn decode_value<T: DeserializeOwned>(
    value: JsValue,
    operation: &str,
) -> Result<T, JsValue> {
    let prototype = Object::get_prototype_of(&Object::new());
    let value = json_value(value, operation, &prototype, 0)?;
    serde_json::from_value(value).map_err(|error| invalid_patch(operation, &error.to_string()))
}

fn json_value(
    value: JsValue,
    operation: &str,
    prototype: &Object,
    depth: usize,
) -> Result<serde_json::Value, JsValue> {
    use serde_json::{Map, Number, Value};
    if depth > 64 {
        return Err(invalid_patch(
            operation,
            "record nesting exceeds 64 levels or contains a cycle",
        ));
    }
    if value.is_null() {
        return Ok(Value::Null);
    }
    if let Some(value) = value.as_bool() {
        return Ok(Value::Bool(value));
    }
    if let Some(value) = value.as_string() {
        return Ok(Value::String(value));
    }
    if let Some(value) = value.as_f64() {
        if !value.is_finite() {
            return Err(invalid_patch(operation, "numbers must be finite"));
        }
        let number = if value.fract() == 0.0 && value.abs() <= 9_007_199_254_740_991.0 {
            Number::from(value as i64)
        } else {
            Number::from_f64(value).expect("finite number")
        };
        return Ok(Value::Number(number));
    }
    if Array::is_array(&value) {
        return value
            .unchecked_into::<Array>()
            .iter()
            .map(|item| json_value(item, operation, prototype, depth + 1))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array);
    }
    if value.is_object() {
        let actual = Object::get_prototype_of(value.unchecked_ref());
        if !actual.is_null() && !Object::is(&actual, prototype) {
            return Err(invalid_patch(
                operation,
                "values must be plain records, arrays, strings, finite numbers, booleans, or null",
            ));
        }
        let mut result = Map::new();
        for entry in Object::entries(value.unchecked_ref()).iter() {
            let entry = entry.unchecked_into::<Array>();
            let value = entry.get(1);
            if value.is_undefined() {
                continue;
            }
            result.insert(
                entry.get(0).as_string().expect("object key"),
                json_value(value, operation, prototype, depth + 1)?,
            );
        }
        return Ok(Value::Object(result));
    }
    Err(invalid_patch(operation, "unsupported record value"))
}

fn invalid_patch(operation: &str, detail: &str) -> JsValue {
    let value = Error::new(&format!("{operation}: invalid patch: {detail}"));
    let _ = Reflect::set(
        &value,
        &"code".into(),
        &ErrorCode::ApiPropertyValue.as_i32().into(),
    );
    let _ = Reflect::set(&value, &"operation".into(), &operation.into());
    let _ = Reflect::set(&value, &"detail".into(), &detail.into());
    value.into()
}

pub(super) fn identity_id(
    simulation: &Simulation,
    object_type: ObjectType,
    index: usize,
    operation: &str,
) -> Result<String, JsValue> {
    simulation
        .inner
        .object_identity_at_read(object_type, index)
        .map_err(|error| simulation.error(operation, error))?
        .map(|identity| identity.id)
        .ok_or_else(|| {
            simulation.error(
                operation,
                SwmmError::with_detail(
                    ErrorCode::ApiObjectIndex,
                    "related object index out of range",
                ),
            )
        })
}

pub(super) fn identity_ids(
    simulation: &Simulation,
    object_type: ObjectType,
    operation: &str,
) -> Result<Vec<String>, JsValue> {
    simulation
        .inner
        .object_identities_read(object_type)
        .map_err(|error| simulation.error(operation, error))
        .map(|values| values.into_iter().map(|identity| identity.id).collect())
}

pub(super) fn serialize(value: &impl Serialize) -> Result<JsValue, JsValue> {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .map_err(Into::into)
}
