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
            RegexTableEntry { r: Regex::new(r"^[0-9]+[lL]\b").unwrap(),     f: Self::parse_long_constant },
            RegexTableEntry { r: Regex::new(r"^[0-9]+\b").unwrap(),         f: Self::parse_int_constant },
            RegexTableEntry { r: Regex::new(r"^\?").unwrap(),               f: |_, _| Token::QuestionMark },
            RegexTableEntry { r: Regex::new(r"^\(").unwrap(),               f: |_, _| Token::OpenParenthesis },
            RegexTableEntry { r: Regex::new(r"^\)").unwrap(),               f: |_, _| Token::CloseParenthesis },
            RegexTableEntry { r: Regex::new(r"^\{").unwrap(),               f: |_, _| Token::OpenBrace },
            RegexTableEntry { r: Regex::new(r"^\}").unwrap(),               f: |_, _| Token::CloseBrace },
            RegexTableEntry { r: Regex::new(r"^;").unwrap(),                f: |_, _| Token::Semicolon },
            RegexTableEntry { r: Regex::new(r"^:").unwrap(),                f: |_, _| Token::Colon },
            RegexTableEntry { r: Regex::new(r"^,").unwrap(),                f: |_, _| Token::Comma },
            // '--' and '-=' must be before '-'
            RegexTableEntry { r: Regex::new(r"^--").unwrap(),               f: |_, _| Token::Decrement },
            RegexTableEntry { r: Regex::new(r"^-=").unwrap(),               f: |_, _| Token::SubAssign },
            RegexTableEntry { r: Regex::new(r"^-").unwrap(),                f: |_, _| Token::Minus },
            RegexTableEntry { r: Regex::new(r"^~").unwrap(),                f: |_, _| Token::Tilde },
            // '++' and '+=' must be before '+'
            RegexTableEntry { r: Regex::new(r"^[+][+]").unwrap(),           f: |_, _| Token::Increment },
            RegexTableEntry { r: Regex::new(r"^[+]=").unwrap(),             f: |_, _| Token::AddAssign },
            RegexTableEntry { r: Regex::new(r"^[+]").unwrap(),              f: |_, _| Token::Plus },
            // '*=' must be before '*'
            RegexTableEntry { r: Regex::new(r"^\*=").unwrap(),              f: |_, _| Token::MulAssign },
            RegexTableEntry { r: Regex::new(r"^\*").unwrap(),               f: |_, _| Token::Asterisk },
            // '/=' must be before '/'
            RegexTableEntry { r: Regex::new(r"^/=").unwrap(),               f: |_, _| Token::DivAssign },
            RegexTableEntry { r: Regex::new(r"^/").unwrap(),                f: |_, _| Token::Slash },
            // '%=' must be before '%'
            RegexTableEntry { r: Regex::new(r"^%=").unwrap(),               f: |_, _| Token::ModAssign },
            RegexTableEntry { r: Regex::new(r"^%").unwrap(),                f: |_, _| Token::Percent },
            // '||' and '|=' must be before '|'
            RegexTableEntry { r: Regex::new(r"^\|\|").unwrap(),             f: |_, _| Token::LogicalOr },
            RegexTableEntry { r: Regex::new(r"^\|=").unwrap(),              f: |_, _| Token::BitwiseOrAssign },
            RegexTableEntry { r: Regex::new(r"^\|").unwrap(),               f: |_, _| Token::VerticalBar },
            // '&&' and '&=' must be before '&'
            RegexTableEntry { r: Regex::new(r"^&&").unwrap(),               f: |_, _| Token::LogicalAnd },
            RegexTableEntry { r: Regex::new(r"^&=").unwrap(),               f: |_, _| Token::BitwiseAndAssign },
            RegexTableEntry { r: Regex::new(r"^&").unwrap(),                f: |_, _| Token::Ampersand },
            // '^=' must be before '^'
            RegexTableEntry { r: Regex::new(r"^\^=").unwrap(),              f: |_, _| Token::BitwiseXorAssign },
            RegexTableEntry { r: Regex::new(r"^\^").unwrap(),               f: |_, _| Token::Caret },
            //'<<=' must be before '<<'
            RegexTableEntry { r: Regex::new(r"^<<=").unwrap(),              f: |_, _| Token::ShiftLeftAssign },
            //'>>=' must be before '>>'
            RegexTableEntry { r: Regex::new(r"^>>=").unwrap(),              f: |_, _| Token::ShiftRightAssign },
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

    fn parse_id(&self, s: &str) -> Token {
        if let Some(kw) = self.keyword_table.get(s) {
            return kw.clone();
        }

        Token::Identifier(String::from(s))
    }

    fn parse_int_constant(&self, s: &str) -> Token {
        if let Ok(int_const) = String::from(s).parse::<i32>() {
            Token::IntConstant(int_const)
        }
        else {
            Token::LongConstant(String::from(s).parse::<i64>().unwrap())
        }
    }

    fn parse_long_constant(&self, s: &str) -> Token {
        let mut long_str = String::from(s);
        let suffix = long_str.pop().unwrap();
        assert_eq!(suffix.to_ascii_lowercase(), 'l');
        Token::LongConstant(long_str.parse::<i64>().unwrap())
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