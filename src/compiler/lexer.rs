use log::{info, trace, warn, error};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};

use regex::Regex;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Identifier(String),
    IntConstant(i32),
    LongConstant(i64),

    KwInt,
    KwLong,
    KwVoid,
    KwReturn,
    KwIf,
    KwElse,
    KwGoto,
    KwDo,
    KwWhile,
    KwFor,
    KwBreak,
    KwContinue,
    KwSwitch,
    KwCase,
    KwDefault,
    KwStatic,
    KwExtern,

    OpenParenthesis,        // (
    CloseParenthesis,       // )
    OpenBrace,              // {
    CloseBrace,             // }
    Semicolon,              // ;
    Decrement,              // --
    Minus,                  // -
    Tilde,                  // ~
    Plus,                   // +
    Increment,              // ++
    Asterisk,               // *
    Slash,                  // /
    Percent,                // %
    VerticalBar,            // |
    LogicalOr,              // ||
    Ampersand,              // &
    LogicalAnd,             // &&
    Caret,                  // ^
    ShiftLeft,              // <<
    ShiftRight,             // >>
    LessOrEqual,            // <=
    GreaterOrEqual,         // >=
    OpenAngleBracket,       // <
    CloseAngleBracket,      // >
    EqualTo,                // ==
    NotEqualTo,             // !=
    ExclamationMark,        // !
    EqualSign,              // =
    QuestionMark,           // ?
    Colon,                  // :
    Comma,                  // ,

    //Compound Assignments
    AddAssign,              // +=
    SubAssign,              // -=
    MulAssign,              // *=
    DivAssign,              // /=
    ModAssign,              // %=
    BitwiseOrAssign,        // |=
    BitwiseAndAssign,       // &=
    BitwiseXorAssign,       // ^=
    ShiftLeftAssign,        // <<=
    ShiftRightAssign,       // >>=

    EOS
}


pub struct Lexer {
    reader: BufReader<fs::File>,
    current_line: String,
    current_line_number: usize,
    current_line_position: usize,
    current_line_length: usize,
    end_of_stream: bool,
    regex_table: RegexTable,
    cached_tokens: Vec<Token>
}


struct RegexTableEntry {
    r: Regex,
    f: fn(&RegexTable, &str) -> Result<Token, String>
}

struct RegexTable {
    white_space_regex: Regex,
    regexes: Vec<RegexTableEntry>,
    keyword_table: HashMap<&'static str, Token>
}



impl RegexTable {
    fn new() -> RegexTable {
        let regexes = vec![
            RegexTableEntry { r: Regex::new(r"^[a-zA-Z_]\w*\b").unwrap(),   f: Self::parse_id },
            RegexTableEntry { r: Regex::new(r"^[0-9]+[lL]\b").unwrap(),     f: Self::parse_long_constant },
            RegexTableEntry { r: Regex::new(r"^[0-9]+\b").unwrap(),         f: Self::parse_int_constant },
            RegexTableEntry { r: Regex::new(r"^\?").unwrap(),               f: |_, _| Ok(Token::QuestionMark) },
            RegexTableEntry { r: Regex::new(r"^\(").unwrap(),               f: |_, _| Ok(Token::OpenParenthesis) },
            RegexTableEntry { r: Regex::new(r"^\)").unwrap(),               f: |_, _| Ok(Token::CloseParenthesis) },
            RegexTableEntry { r: Regex::new(r"^\{").unwrap(),               f: |_, _| Ok(Token::OpenBrace) },
            RegexTableEntry { r: Regex::new(r"^\}").unwrap(),               f: |_, _| Ok(Token::CloseBrace) },
            RegexTableEntry { r: Regex::new(r"^;").unwrap(),                f: |_, _| Ok(Token::Semicolon) },
            RegexTableEntry { r: Regex::new(r"^:").unwrap(),                f: |_, _| Ok(Token::Colon) },
            RegexTableEntry { r: Regex::new(r"^,").unwrap(),                f: |_, _| Ok(Token::Comma) },
            // '--' and '-=' must be before '-'
            RegexTableEntry { r: Regex::new(r"^--").unwrap(),               f: |_, _| Ok(Token::Decrement) },
            RegexTableEntry { r: Regex::new(r"^-=").unwrap(),               f: |_, _| Ok(Token::SubAssign) },
            RegexTableEntry { r: Regex::new(r"^-").unwrap(),                f: |_, _| Ok(Token::Minus) },
            RegexTableEntry { r: Regex::new(r"^~").unwrap(),                f: |_, _| Ok(Token::Tilde) },
            // '++' and '+=' must be before '+'
            RegexTableEntry { r: Regex::new(r"^[+][+]").unwrap(),           f: |_, _| Ok(Token::Increment) },
            RegexTableEntry { r: Regex::new(r"^[+]=").unwrap(),             f: |_, _| Ok(Token::AddAssign) },
            RegexTableEntry { r: Regex::new(r"^[+]").unwrap(),              f: |_, _| Ok(Token::Plus) },
            // '*=' must be before '*'
            RegexTableEntry { r: Regex::new(r"^\*=").unwrap(),              f: |_, _| Ok(Token::MulAssign) },
            RegexTableEntry { r: Regex::new(r"^\*").unwrap(),               f: |_, _| Ok(Token::Asterisk) },
            // '/=' must be before '/'
            RegexTableEntry { r: Regex::new(r"^/=").unwrap(),               f: |_, _| Ok(Token::DivAssign) },
            RegexTableEntry { r: Regex::new(r"^/").unwrap(),                f: |_, _| Ok(Token::Slash) },
            // '%=' must be before '%'
            RegexTableEntry { r: Regex::new(r"^%=").unwrap(),               f: |_, _| Ok(Token::ModAssign) },
            RegexTableEntry { r: Regex::new(r"^%").unwrap(),                f: |_, _| Ok(Token::Percent) },
            // '||' and '|=' must be before '|'
            RegexTableEntry { r: Regex::new(r"^\|\|").unwrap(),             f: |_, _| Ok(Token::LogicalOr) },
            RegexTableEntry { r: Regex::new(r"^\|=").unwrap(),              f: |_, _| Ok(Token::BitwiseOrAssign) },
            RegexTableEntry { r: Regex::new(r"^\|").unwrap(),               f: |_, _| Ok(Token::VerticalBar) },
            // '&&' and '&=' must be before '&'
            RegexTableEntry { r: Regex::new(r"^&&").unwrap(),               f: |_, _| Ok(Token::LogicalAnd) },
            RegexTableEntry { r: Regex::new(r"^&=").unwrap(),               f: |_, _| Ok(Token::BitwiseAndAssign) },
            RegexTableEntry { r: Regex::new(r"^&").unwrap(),                f: |_, _| Ok(Token::Ampersand) },
            // '^=' must be before '^'
            RegexTableEntry { r: Regex::new(r"^\^=").unwrap(),              f: |_, _| Ok(Token::BitwiseXorAssign) },
            RegexTableEntry { r: Regex::new(r"^\^").unwrap(),               f: |_, _| Ok(Token::Caret) },
            //'<<=' must be before '<<'
            RegexTableEntry { r: Regex::new(r"^<<=").unwrap(),              f: |_, _| Ok(Token::ShiftLeftAssign) },
            //'>>=' must be before '>>'
            RegexTableEntry { r: Regex::new(r"^>>=").unwrap(),              f: |_, _| Ok(Token::ShiftRightAssign) },
            RegexTableEntry { r: Regex::new(r"^<<").unwrap(),               f: |_, _| Ok(Token::ShiftLeft) },
            RegexTableEntry { r: Regex::new(r"^>>").unwrap(),               f: |_, _| Ok(Token::ShiftRight) },
            RegexTableEntry { r: Regex::new(r"^<=").unwrap(),               f: |_, _| Ok(Token::LessOrEqual) },
            RegexTableEntry { r: Regex::new(r"^>=").unwrap(),               f: |_, _| Ok(Token::GreaterOrEqual) },
            RegexTableEntry { r: Regex::new(r"^<").unwrap(),                f: |_, _| Ok(Token::OpenAngleBracket) },
            RegexTableEntry { r: Regex::new(r"^>").unwrap(),                f: |_, _| Ok(Token::CloseAngleBracket) },
            // '==' must come before plain '='
            RegexTableEntry { r: Regex::new(r"^==").unwrap(),               f: |_, _| Ok(Token::EqualTo) },
            RegexTableEntry { r: Regex::new(r"^!=").unwrap(),               f: |_, _| Ok(Token::NotEqualTo) },
            RegexTableEntry { r: Regex::new(r"^!").unwrap(),                f: |_, _| Ok(Token::ExclamationMark) },
            RegexTableEntry { r: Regex::new(r"^=").unwrap(),                f: |_, _| Ok(Token::EqualSign) }
        ];

        RegexTable {
            white_space_regex: Regex::new(r"\s*").unwrap(),
            regexes,
            keyword_table: HashMap::from([
                ("else",        Token::KwElse),
                ("if",          Token::KwIf),
                ("int",         Token::KwInt),
                ("long",        Token::KwLong),
                ("return",      Token::KwReturn),
                ("void",        Token::KwVoid),
                ("goto",        Token::KwGoto),
                ("do",          Token::KwDo ),
                ("while",       Token::KwWhile),
                ("for",         Token::KwFor),
                ("break",       Token::KwBreak),
                ("continue",    Token::KwContinue),
                ("switch",      Token::KwSwitch),
                ("case",        Token::KwCase),
                ("default",     Token::KwDefault),
                ("static",      Token::KwStatic),
                ("extern",      Token::KwExtern)
            ])
        }
    }

    fn parse_id(&self, s: &str) -> Result<Token, String> {
        if let Some(kw) = self.keyword_table.get(s) {
            return Ok(kw.clone());
        }

        Ok(Token::Identifier(String::from(s)))
    }

    fn parse_int_constant(&self, s: &str) -> Result<Token, String> {
        if let Ok(int_const) = String::from(s).parse::<i32>() {
            Ok(Token::IntConstant(int_const))
        }
        else if let Ok(long_const) = String::from(s).parse::<i64>() {
            Ok(Token::LongConstant(long_const))
        }
        else {
            Err(format!("Failed to parse integer constant: '{}' Overflow??", s))
        }
    }

    fn parse_long_constant(&self, s: &str) -> Result<Token, String> {
        let mut long_str = String::from(s);
        let suffix = long_str.pop().unwrap();
        assert_eq!(suffix.to_ascii_lowercase(), 'l');

        if let Ok(long_constant) = long_str.parse::<i64>() {
            Ok(Token::LongConstant(long_constant))
        }
        else {
            Err(format!("Failed to parse long constant: '{}' Overflow??", s))
        }
    }

}





impl Lexer {
    pub fn new(input_file_path: &str) -> Result<Lexer, String> {
        let file = match fs::File::open(input_file_path) {
            Ok(f) => f,
            Err(err) => {
                return Err(format!("Error opening file: '{input_file_path}': {err}"));
            }
        };

        Ok(Lexer {
            reader: BufReader::new(file),
            current_line: String::new(),
            current_line_number: 0,
            current_line_position: 0,
            current_line_length: 0,
            end_of_stream: false,
            regex_table: RegexTable::new(),
            cached_tokens: vec![]
        })
    }

    fn read_a_new_line(&mut self) -> Result<(), String>
    {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(size) => {
                if size == 0 {
                    self.end_of_stream = true;
                    return Ok(());
                }
            },
            Err(err) => {
                return Err(format!("Error reading line: {err}"));
            }
        }

        self.current_line_number += 1;
        self.current_line_position = 0;
        self.current_line_length = line.len();
        self.current_line = line;

        Ok(())
    }


    fn fetch_token(&mut self) -> Result<Token, String>
    {
        loop
        {
            if self.end_of_stream {
                return Ok(Token::EOS);
            }

            if self.current_line_position >= self.current_line_length {
                self.read_a_new_line()?;

                if self.end_of_stream {
                    return Ok(Token::EOS);
                }
            }

            while self.current_line_position < self.current_line_length {

                trace!("Current line: [pos={}, len={}] --> '{}'", self.current_line_position, self.current_line_length, self.current_line);

                if let Some(ws_match) = self.regex_table.white_space_regex.find(&self.current_line[self.current_line_position..]) {
                    trace!("SKIP WHITESPACE between [{}..{}]", self.current_line_position + ws_match.start(), self.current_line_position + ws_match.end());
                    if !ws_match.is_empty() {
                        self.current_line_position += ws_match.len();
                    }

                    if self.current_line_position >= self.current_line_length {
                        break;
                    }
                }


                for rte in self.regex_table.regexes.iter() {
                    if let Some(token_match) = rte.r.find(&self.current_line[self.current_line_position..]) {
                        if !token_match.is_empty() {
                            match (rte.f)(&self.regex_table, token_match.as_str()) {
                                Ok(token) => {
                                    trace!("FOUND TOKEN {token:?} between [{}..{}]", self.current_line_position + token_match.start(), self.current_line_position + token_match.end());
                                    self.current_line_position += token_match.len();
                                    return Ok(token);
                                },
                                Err(e) => {
                                    error!("Token parse error at line '{}', position {}: {}", self.current_line, self.current_line_position, e);
                                    return Err(format!("Token parse error at line '{}', position {}: {}", self.current_line, self.current_line_position, e));
                                }
                            }
                        }
                    }
                }

                error!("NO TOKENS match line '{}' at position {}", self.current_line, self.current_line_position);

                return Err(format!("NO TOKENS match line '{}' at position {}", self.current_line, self.current_line_position));

            }
        }
    }

    pub fn get_token(&mut self) -> Result<Token, String>
    {
        let token = match self.cached_tokens.pop() {
            Some(cached_token) => cached_token,
            None => self.fetch_token()?
        };

        Ok(token)
    }

    pub fn peek_token(&mut self) -> Result<Token, String>
    {
        let token = self.get_token()?;

        self.cached_tokens.push(token.clone());

        Ok(token)
    }

    pub fn putback_token(&mut self, token: Token) -> Result<(), String>
    {
        self.cached_tokens.push(token);

        Ok(())
    }

    pub fn get_current_line_number(&self) -> usize { self.current_line_number }

}