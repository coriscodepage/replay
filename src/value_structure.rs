pub trait Value {
    fn to_bool(&self) -> Option<bool>;
    fn to_usize(&self) -> Option<usize>;
    fn to_i64(&self) -> Option<i64>;
    fn to_f32(&self) -> Option<f32>;
    fn to_f64(&self) -> Option<f64>;
}

pub struct None {}

impl Value for None {
    fn to_bool(&self) -> Option<bool> {
        None
    }
    fn to_usize(&self) -> Option<usize> {
        None
    }
    fn to_f32(&self) -> Option<f32> {
        None
    }
    fn to_f64(&self) -> Option<f64> {
        None
    }
    fn to_i64(&self) -> Option<i64> {
        None
    }
}

pub struct Bool {
    pub value: bool,
}

impl Value for Bool {
    fn to_bool(&self) -> Option<bool> {
        Some(self.value)
    }
    fn to_usize(&self) -> Option<usize> {
        match self.value {
            true => Some(1),
            false => Some(0),
        }
    }
    fn to_i64(&self) -> Option<i64> {
        match self.value {
            true => Some(1),
            false => Some(0),
        }
    }
    fn to_f32(&self) -> Option<f32> {
        match self.value {
            true => Some(1.0),
            false => Some(0.0),
        }
    }
    fn to_f64(&self) -> Option<f64> {
        match self.value {
            true => Some(1.0),
            false => Some(0.0),
        }
    }
}

pub struct Usize {
    value: usize,
}

impl Value for Usize {
    fn to_bool(&self) -> Option<bool> {
        Some(self.value != 0)
    }
    fn to_usize(&self) -> Option<usize> {
        Some(self.value)
    }
    fn to_i64(&self) -> Option<i64> {
        return Some(self.value as i64);
    }
    fn to_f32(&self) -> Option<f32> {
        Some(self.value as f32)
    }
    fn to_f64(&self) -> Option<f64> {
        Some(self.value as f64)
    }
}

pub struct I64 {
    value: i64,
}

impl Value for I64 {
    fn to_bool(&self) -> Option<bool> {
        Some(self.value != 0)
    }
    fn to_usize(&self) -> Option<usize> {
        if self.value >= 0 {
            return Some(self.value as usize);
        }
        None
    }
    fn to_i64(&self) -> Option<i64> {
        Some(self.value)
    }
    fn to_f32(&self) -> Option<f32> {
        Some(self.value as f32)
    }
    fn to_f64(&self) -> Option<f64> {
        Some(self.value as f64)
    }
}

pub struct Float {
    value: f32,
}

impl Value for Float {
    fn to_bool(&self) -> Option<bool> {
        Some(self.value != 0.0)
    }
    fn to_usize(&self) -> Option<usize> {
        Some(self.value as usize)
    }
    fn to_i64(&self) -> Option<i64> {
        Some(self.value as i64)
    }
    fn to_f32(&self) -> Option<f32> {
        Some(self.value)
    }
    fn to_f64(&self) -> Option<f64> {
        Some(self.value as f64)
    }
}

pub struct Double {
    value: f64,
}

impl Value for Double {
    fn to_bool(&self) -> Option<bool> {
        Some(self.value != 0.0)
    }
    fn to_usize(&self) -> Option<usize> {
        Some(self.value as usize)
    }
    fn to_i64(&self) -> Option<i64> {
        Some(self.value as i64)
    }
    fn to_f32(&self) -> Option<f32> {
        Some(self.value as f32)
    }
    fn to_f64(&self) -> Option<f64> {
        Some(self.value)
    }
}
