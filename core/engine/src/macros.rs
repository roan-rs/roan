#[macro_export]
macro_rules! entries {
    ($($key:expr => $value:expr),*) => {
        {
            let mut map = HashMap::new();
            $(
                map.insert($key.to_string(), $value);
            )*
            map
        }
    };
}
