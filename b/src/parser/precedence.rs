
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
    //Right and Left associativity; if left associative, left bp > right bp.
    //if right associative, left bp = right bp as check is that left bp > minbp.
    pub fn bp(self) -> (u8, u8) {

        match self {
            Precedence::None => (0, 0),
            Precedence::Assignment => (1, 1), //right associative
            Precedence::Ternary => (2, 2), //right associative
            Precedence::BitOr => (4, 3),
            Precedence::BitXor => (6, 5),
            Precedence::BitAnd => (8, 7),
            Precedence::Equality => (10, 9),
            Precedence::Comparison => (12, 11),
            Precedence::Shift => (14, 13),
            Precedence::Addition => (16, 15),
            Precedence::Multiplication => (18, 17),
            Precedence::Unary => (19, 19), //right associative
            Precedence::Postfix => (21, 20)
        }
    }
}