use super::*;

pub trait Resource: Any + Sized {}

impl<T: Any> Resource for T {}
