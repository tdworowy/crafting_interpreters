#[derive(Debug)]
pub struct Scaner {
    start: char,
    current: char,
    line: i32,
}

impl Scaner {
    pub fn new(source: char) -> Self {
        Self {
            start: source,
            current: source,
            line: 1,
        }
    }
}
