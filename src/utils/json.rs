#[macro_export]
macro_rules! json_map {
    ($( $key:expr => $val:expr), *) => {
        {
            let mut map = serde_json::Map::new();
            $(map.insert($key.to_string(), $val); )*

            map
        }
    }
}
