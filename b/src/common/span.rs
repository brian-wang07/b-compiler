#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

pub struct Span {
    pub start: Location,
    pub end: Location,
}
