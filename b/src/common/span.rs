#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

impl Default for Location {
    fn default() -> Self {
        Location {
            offset: 0,
            line: 0,
            column: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: Location,
    pub end: Location,
}

impl Default for Span {
    fn default() -> Self {     
        Span {
            start: Location::default(),
            end: Location::default()
        }
    }
}
