
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precedence {
    None,
    Assignment, // =, =+, =-, etc.
    Ternary,    // ?:
    BitOr,     // |
    BitXor,    // ^
    BitAnd,    // &
    Equality,  // ==, !=
    Comparison, // <, >, <=, >=
    Shift,     // <<, >>
    Addition,  // +, -
    Multiplication, // *, /, %
    Unary, //prefix unary and logical not
    Postfix, //postfix unary
}

impl Precedence {
    //Left and Right associativity: if left associative, lbp == rbp
    //If right associative, lbp > rbp
    pub fn bp(self) -> (u8, u8) {

        match self {
            Precedence::None => (0, 0),
            Precedence::Assignment => (2, 1), //right associative
            Precedence::Ternary => (4, 3), //right associative
            Precedence::BitOr => (5, 5),
            Precedence::BitXor => (6, 6),
            Precedence::BitAnd => (7, 7),
            Precedence::Equality => (8, 8),
            Precedence::Comparison => (9, 9),
            Precedence::Shift => (10, 10),
            Precedence::Addition => (11, 11),
            Precedence::Multiplication => (12, 12),
            Precedence::Unary => (14, 13), //right associative (prefix)
            Precedence::Postfix => (15, 15)
        }
    }
}