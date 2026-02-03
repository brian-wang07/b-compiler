use crate::common::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Auto,
    Extrn,
    If,
    Else,
    While,
    Switch,
    Case,
    Default,
    Return,
    Goto,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Operator {
    //arithmetic and bitwise
    Plus,
    Minus,
    Star, //* for multiplication and unary indirection/dereference
    Slash,
    Percent,
    Amp, //& Bitwise AND and unary reference
    Bar, //| Bitwise OR
    Caret, //^ Bitwise XOR
    LShift, //<<
    RShift, //>>
    Bang, // ! Logical NOT
    Tilde, //~ Bitwise NOT/Ones compliment

    //Comparison 
    Equal,  //==
    NotEqual, 
    Less,
    Greater,
    LessEq,
    GreaterEq,

    //Increment and Decrement
    Inc, //++
    Dec, //--

    //Assignment - Reversed Compound Assignment (=+ instead of +=)
    Assign, //=
    AssignPlus, //=+
    AssignMinus, //=-
    AssignStar, //=*
    AssignSlash, //=/
    AssignPercent, //=%
    AssignAmp, //=&
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Delimiter {
    //()
    LParen,
    RParen,
    //[]
    LBrack,
    RBrack,
    //{}
    LBrace,
    RBrace,

    Comma,
    Semicolon,
    Colon,
    QMark
}

//main Token Enum

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    Keyword(Keyword),
    Operator(Operator),
    Delimiter(Delimiter),
    Identifier(&'a str),
    Integer(i64),
    // * instead of / used for escape characters
    StringLiteral(String),
    CharLiteral(i64),
    EOF, //End of file
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken<'a> {
    pub token: Token<'a>,
    pub span: Span,
}
