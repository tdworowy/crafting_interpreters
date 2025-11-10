#[macro_export]
macro_rules! grow_capacity {
    ($capacity:expr) => {{
        let cap = $capacity;
        if cap < 8 { 8 } else { cap * 2 }
    }};
}
