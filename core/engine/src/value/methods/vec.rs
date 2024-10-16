use crate::{
    as_cast, native_function,
    value::Value,
    vm::native_fn::{NativeFunction, NativeFunctionParam},
};

native_function!(
    fn __vec_len(vec) {
        let vec = as_cast!(vec, Vec);

        Value::Int(vec.len() as i64)
    }
);

native_function!(
    fn __vec_next(vec) {
        let mut vec = as_cast!(vec, Vec);

        vec = vec.into_iter().skip(1).collect();

        Value::Vec(vec)
    }
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_vec_len() {
        let mut vec = vec![Value::Int(1), Value::Int(2), Value::Int(3)];
        let result = __vec_len().call(vec![Value::Vec(vec.clone())]).unwrap();

        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_vec_next() {
        let mut vec = vec![Value::Int(1), Value::Int(2), Value::Int(3)];
        let result = __vec_next().call(vec![Value::Vec(vec.clone())]).unwrap();

        assert_eq!(result, Value::Vec(vec![Value::Int(2), Value::Int(3)]));
    }
}