use log::{info, trace, warn, error};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};

use regex::Regex;


#[derive(Debug, Clone)]
pub enum Token {
    Identifier(String),
    IntConstant(i32),

    KwInt,
    KwVoid,
    KwReturn,

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
    cached_token: Option<Token>
}


struct RegexTableEntry {
    r: Regex,
    f: fn(&RegexTable, &str) -> Token
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
            RegexTableEntry { r: Regex::new(r"^[0-9]+\b").unwrap(),         f: Self::parse_int_constant },
            RegexTableEntry { r: Regex::new(r"^\(").unwrap(),               f: |_, _| Token::OpenParenthesis },
            RegexTableEntry { r: Regex::new(r"^\)").unwrap(),               f: |_, _| Token::CloseParenthesis },
            RegexTableEntry { r: Regex::new(r"^\{").unwrap(),               f: |_, _| Token::OpenBrace },
            RegexTableEntry { r: Regex::new(r"^\}").unwrap(),               f: |_, _| Token::CloseBrace },
            RegexTableEntry { r: Regex::new(r"^;").unwrap(),                f: |_, _| Token::Semicolon },
            //The entry below (decrement, i.e. '--') must be before the one for minus, i.e. '-'
            RegexTableEntry { r: Regex::new(r"^--").unwrap(),               f: |_, _| Token::Decrement },
            RegexTableEntry { r: Regex::new(r"^-").unwrap(),                f: |_, _| Token::Minus },
            RegexTableEntry { r: Regex::new(r"^~").unwrap(),                f: |_, _| Token::Tilde },
            //The entry below (increment, i.e. '++') must be before the one for plus, i.e. '+'
            RegexTableEntry { r: Regex::new(r"^[+][+]").unwrap(),           f: |_, _| Token::Increment },
            RegexTableEntry { r: Regex::new(r"^[+]").unwrap(),              f: |_, _| Token::Plus },
            RegexTableEntry { r: Regex::new(r"^\*").unwrap(),               f: |_, _| Token::Asterisk },
            RegexTableEntry { r: Regex::new(r"^/").unwrap(),                f: |_, _| Token::Slash },
            RegexTableEntry { r: Regex::new(r"^%").unwrap(),                f: |_, _| Token::Percent },
            RegexTableEntry { r: Regex::new(r"^\|\|").unwrap(),             f: |_, _| Token::LogicalOr },
            RegexTableEntry { r: Regex::new(r"^&&").unwrap(),               f: |_, _| Token::LogicalAnd },
            RegexTableEntry { r: Regex::new(r"^\|").unwrap(),               f: |_, _| Token::VerticalBar },
            RegexTableEntry { r: Regex::new(r"^&").unwrap(),                f: |_, _| Token::Ampersand },
            RegexTableEntry { r: Regex::new(r"^\^").unwrap(),               f: |_, _| Token::Caret },
            RegexTableEntry { r: Regex::new(r"^<<").unwrap(),               f: |_, _| Token::ShiftLeft },
            RegexTableEntry { r: Regex::new(r"^>>").unwrap(),               f: |_, _| Token::ShiftRight },
            RegexTableEntry { r: Regex::new(r"^<=").unwrap(),               f: |_, _| Token::LessOrEqual },
            RegexTableEntry { r: Regex::new(r"^>=").unwrap(),               f: |_, _| Token::GreaterOrEqual },
            RegexTableEntry { r: Regex::new(r"^<").unwrap(),                f: |_, _| Token::OpenAngleBracket },
            RegexTableEntry { r: Regex::new(r"^>").unwrap(),                f: |_, _| Token::CloseAngleBracket },
            // '==' must come before plain '='
            RegexTableEntry { r: Regex::new(r"^==").unwrap(),               f: |_, _| Token::EqualTo },
            RegexTableEntry { r: Regex::new(r"^!=").unwrap(),               f: |_, _| Token::NotEqualTo },
            RegexTableEntry { r: Regex::new(r"^!").unwrap(),                f: |_, _| Token::ExclamationMark },
            RegexTableEntry { r: Regex::new(r"^=").unwrap(),                f: |_, _| Token::EqualSign }
        ];

        RegexTable {
            white_space_regex: Regex::new(r"\s*").unwrap(),
            regexes,
            keyword_table: HashMap::from([
                ("int",         Token::KwInt),
                ("return",      Token::KwReturn),
                ("void",        Token::KwVoid)
            ])
        }
    }

    fn parse_id(&self, s: &str) -> Token {
        if let Some(kw) = self.keyword_table.get(s) {
            return kw.clone();
        }

        Token::Identifier(String::from(s))
    }

    fn parse_int_constant(&self, s: &str) -> Token { Token::IntConstant(String::from(s).parse().unwrap()) }

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
            cached_token: None
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
                            let token = (rte.f)(&self.regex_table, token_match.as_str());
                            trace!("FOUND TOKEN {token:?} between [{}..{}]", self.current_line_position + token_match.start(), self.current_line_position + token_match.end());
                            self.current_line_position += token_match.len();

                            return Ok(token);
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
        let token = match self.cached_token.take() {
            Some(cached_token) => cached_token,
            None => self.fetch_token()?
        };

        Ok(token)
    }

    pub fn peek_token(&mut self) -> Result<Token, String>
    {
        let token = self.get_token()?;

        self.cached_token.replace(token.clone());

        Ok(token)
    }

    pub fn get_current_line_number(&self) -> usize { self.current_line_number }

}