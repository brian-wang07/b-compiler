//derive line + col number via lexer; store \n chars in vec and bin search
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Span {
    pub start: u32, //usize is not necessary, and is double the size.
    pub end: u32,
}

//debugging
impl Default for Span {
    fn default() -> Self {
        Span {
            start: 0,
            end: 0
        }
    }
}
