use crate::common::span::{Location, Span};
use crate::lexer::token::{SpannedToken, Token, Keyword, Delimiter, Operator};
use super::LexError;

pub struct Scanner<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    current_loc: Location,
    source: &'a str,
}

impl<'a> Scanner<'a> {
    
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            current_loc: Location { offset: 0, line: 1, column: 1 },
            source
        }
    }

    //advances the scanner, consuming lexeme
    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next()?; //if none return none

        self.current_loc.offset += c.len_utf8(); //usually 1, except for special characters

        if c == '\n' {
            self.current_loc.line += 1;
            self.current_loc.column = 1;
        } else {
            self.current_loc.column += 1;
        }

        Some(c)
    }
    //check next character
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn peek_next(&self) -> Option<char> {
        let mut iter = self.chars.clone();
        iter.next(); //skip char that peek() sees
        iter.next() //return next char
    }

    //match next character with an expected value
    fn match_char(&mut self, expected: char) -> bool {
        //no implicit self
        if self.peek() == Some(expected) {
            self.advance();
            return true;
        }
        false
    }

    //scan numeric literals; in B, octal numbers are denoted with a leading 0. 
    fn read_number(&mut self, first_char: char) -> Result<Token, LexError> { //enum variants
                                                                              //cannot be return
                                                                              //types
        let start_offset = self.current_loc.offset - first_char.len_utf8();
        let radix = if first_char =='0' { 8 } else { 10 };
        
        //advance to end of number
        while let Some(c) = self.peek() {
            if c.is_digit(radix) {
                self.advance();
            } else {
                break;
            }
        }

        let lexeme = &self.source[start_offset..self.current_loc.offset];

        i64::from_str_radix(lexeme, radix as u32)
            .map(Token::Integer) //if okay return integer token
            .map_err(|_| LexError::InvalidNumber(lexeme.to_string(), self.current_loc)) //else take parseinterror and
                                                                      //raise LexError
    }

    //scan variable identifiers/keywords; read until peek() not alphanumeric
    fn read_identifier(&mut self, first_char: char) -> Result<Token, LexError> {
        let start_offset = self.current_loc.offset - first_char.len_utf8();

        //advance to end of keyword/variable name
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let lexeme = &self.source[start_offset..self.current_loc.offset];

        //match keyword, else variable identifier
        let token = match lexeme {
            "auto" => Token::Keyword(Keyword::Auto),
            "extrn" => Token::Keyword(Keyword::Extrn),
            "if" => Token::Keyword(Keyword::If),
            "else" => Token::Keyword(Keyword::Else),
            "while" => Token::Keyword(Keyword::While),
            "switch" => Token::Keyword(Keyword::Switch),
            "case" => Token::Keyword(Keyword::Case),
            "default" => Token::Keyword(Keyword::Default),
            "return" => Token::Keyword(Keyword::Return),
            "goto" => Token::Keyword(Keyword::Goto),
            _ => Token::Identifier(lexeme.to_string()),
        };

        Ok(token)

    }

    fn read_str(&mut self) -> Result<Token, LexError> {
        let mut content = String::new();

        while let Some(c) = self.advance() {
            match c {

                '"' => { 
                    self.advance();
                    return Ok(Token::StringLiteral(content)); 
                }, //matching ", return string literal


                '*' => { // * used as escape character 
                    match self.advance() {
                        Some('n') => content.push('\n'),
                        Some('t') => content.push('\t'),
                        Some('0') => content.push('\0'),
                        Some('*') => content.push('*'),
                        Some('"') => content.push('"'),
                        Some(other) => content.push(other),
                        None => return Err(LexError::UnterminatedString(self.current_loc)),
                    }
                },


                _ => content.push(c),

            }
        }
        //breaking from while loop implies that closing " was not found, hence raise error
        Err(LexError::UnterminatedString(self.current_loc)) 
    }

    fn skip_whitespace(&mut self) -> Result<(), LexError> {
        loop {
            match self.peek() {
                //normal whitespace
                Some(' ') | Some('\t') | Some('\n') | Some('\r') => { 
                    self.advance();
                }

                //comments
                Some('/') => {
                    if self.peek_next() == Some('*') {
                        self.consume_comment()?;
                    } else {
                        break; //no matching * implies division operator
                    }
                }

                _ => break,                

            }
        }

        Ok(())
    }

    fn consume_comment(&mut self) -> Result<(), LexError> {
        //scanner is still looking at /, so advance twice to skip /*.
        self.advance(); //consume /
        self.advance(); //consume *
        while let Some(c) = self.advance() {
            if c == '*' && self.match_char('/') {
                return Ok(());
            }
        }

        //reached EOF with open comment, raise error
        Err(LexError::UnterminatedComment(self.current_loc))
    }

    fn read_char_literal(&mut self) -> Result<Token, LexError> {
        //scanner has consumed '
        //Historically, B compilers allowed char literals to store 0, 1, or 2 characters.
        let mut chars = Vec::new();

        while let Some(c) = self.advance() {
            match c {
                '\'' => {
                    //closed literal, pack into i64 
                    let mut value: i64 = 0;
                    for &ch in chars.iter().take(2) {
                        value = ( value << 8 ) | (ch as i64); //lshift first char, then add second char
                    }
                    return Ok(Token::CharLiteral(value));
                },

                '*' => {
                    match self.advance() {
                        Some('n') => chars.push('\n'),
                        Some('t') => chars.push('\t'),
                        Some('0') => chars.push('\0'),
                        Some('*') => chars.push('*'),
                        Some('"') => chars.push('"'),
                        Some(other) => chars.push(other),
                        None => return Err(LexError::UnterminatedChar(self.current_loc)), 
                    }
                },

                _ => chars.push(c),
            }
        }

        Err(LexError::UnterminatedChar(self.current_loc))
    }

    
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<SpannedToken, LexError>;

    fn next(&mut self) -> Option<Self::Item> { 

        //skip whitespaces, handle comment errors
        if let Err(e) = self.skip_whitespace() {
            return Some(Err(e));
        };

        //check for EOF
        let _ = self.peek()?;

        let start_loc = self.current_loc;

        //consume first char
        let c = self.advance()?;

        let result = match c {
            '(' => Ok(Token::Delimiter(Delimiter::LParen)),
            ')' => Ok(Token::Delimiter(Delimiter::RParen)),
            '[' => Ok(Token::Delimiter(Delimiter::LBrack)),
            ']' => Ok(Token::Delimiter(Delimiter::RBrack)),
            '{' => Ok(Token::Delimiter(Delimiter::LBrace)),
            '}' => Ok(Token::Delimiter(Delimiter::RBrace)),
            ',' => Ok(Token::Delimiter(Delimiter::Comma)),
            ';' => Ok(Token::Delimiter(Delimiter::Semicolon)),
            ':' => Ok(Token::Delimiter(Delimiter::Colon)),
            '?' => Ok(Token::Delimiter(Delimiter::QMark)),

            '0'..='9' => self.read_number(c),
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(c),
            '"' => self.read_str(),
            '\'' => self.read_char_literal(),

            '=' => {
                if self.match_char('=') { Ok(Token::Operator(Operator::Equal)) }
                else if self.match_char('+') { Ok(Token::Operator(Operator::AssignPlus)) }
                else if self.match_char('-') { Ok(Token::Operator(Operator::AssignMinus)) }
                else if self.match_char('*') { Ok(Token::Operator(Operator::AssignStar)) }
                else if self.match_char('/') { Ok(Token::Operator(Operator::AssignSlash)) }
                else if self.match_char('%') { Ok(Token::Operator(Operator::AssignPercent)) }
                else if self.match_char('&') { Ok(Token::Operator(Operator::AssignAmp)) }
                else { Ok(Token::Operator(Operator::Assign)) }
            }

            '+' => {
                if self.match_char('+') { Ok(Token::Operator(Operator::Inc)) }
                else { Ok(Token::Operator(Operator::Plus)) }
            }

            '-' => {
                if self.match_char('-') { Ok(Token::Operator(Operator::Dec)) }
                else { Ok(Token::Operator(Operator::Minus)) }
            }

            '*' => Ok(Token::Operator(Operator::Star)),
            '/' => Ok(Token::Operator(Operator::Slash)),
            '%' => Ok(Token::Operator(Operator::Percent)),
            '|' => Ok(Token::Operator(Operator::Bar)),
            '^' => Ok(Token::Operator(Operator::Caret)),
            '!' => Ok(Token::Operator(Operator::Bang)),
            '~' => Ok(Token::Operator(Operator::Tilde)),
            '&' => Ok(Token::Operator(Operator::Amp)),

            '>' => {
                if self.match_char('>') { Ok(Token::Operator(Operator::RShift)) }
                else if self.match_char('=') { Ok(Token::Operator(Operator::GreaterEq)) }
                else { Ok(Token::Operator(Operator::Greater)) }
            }

            '<' => {
                if self.match_char('<') { Ok(Token::Operator(Operator::LShift)) }
                else if self.match_char('=') { Ok(Token::Operator(Operator::LessEq)) }
                else { Ok(Token::Operator(Operator::Greater)) }
            }

            _ => Err(LexError::UnexpectedChar(c, self.current_loc)),

        };
        
        Some(result.map(|token| SpannedToken {
            token,
            span: Span { start: start_loc, end: self.current_loc}
        }))


    }
}
