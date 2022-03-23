#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_yaml::Value;

use anyhow::{anyhow, Result};

macro_rules! yaml_mapping {
    ($( $key:expr => $val:expr), *) => {
        {
            let mut map = serde_yaml::Mapping::new();
            $(map.insert(serde_yaml::Value::String($key.to_string()), $val); )*

            map
        }
    }
}


pub fn yaml_get_value_e<T: DeserializeOwned + Default>(value: Value, path: &str) -> Result<T> {
    let v = yaml_get_value_by_path(value, path);

    match serde_yaml::from_value(v) {
        Ok(v) => Ok(v),
        Err(e) => Err(anyhow!(e)),
    }
}

/// Return the value corresponding to a given path.
///
/// If no value is found, it will return Value::Null.
fn yaml_get_value_by_path(value: Value, path: &str) -> Value {
    let mut val = value.clone();

    if path.is_empty() {
        return val;
    }

    let slice_path: Vec<&str> = path.split('.').collect();

    for k in slice_path.iter() {
        let val_type = yaml_get_value_type(&val);
        val = match k.parse::<usize>() {
            Ok(n) => {
                // n can be an integer.
                // So if val is an array, val[n] is an element of array
                if val_type.as_str() == "sequence" {
                    val[n].clone()
                } else { // Otherwise, it is a just an element of a object so string
                    val[k].clone()
                }
            },
            Err(_) => val[k].clone(),
        };

        if val == Value::Null {
            return val;
        }
    }

    val
}

fn yaml_get_value_type(value: &Value) -> String {
    match value {
        Value::Number(_) => "number".to_string(),
        Value::Bool(_) => "bool".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Sequence(_) => "sequence".to_string(),
        Value::Mapping(_) => "mapping".to_string(),
        Value::Null => "null".to_string(),
    }
}
