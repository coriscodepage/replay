
pub trait value {
    fn to_bool(&self) -> Option<bool>;
    fn to_usize(&self) -> Option<usize>;
    fn to_i64(&self) -> Option<i64>;
    fn to_f32(&self) -> Option<f32>;
    fn to_f64(&self) -> Option<f64>;
}