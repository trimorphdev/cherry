mod token;

pub use token::{
    Comment, CommentKind, Float, Group, Iden, Int, IntKind, Loc, Punct, Skipped, Spacing, Str,
    TokenTree,
};

use codespan_reporting::diagnostic::{Diagnostic, Label};
use snailquote::{unescape, UnescapeError};
use unicode_xid::UnicodeXID;

/// Cherry's lexer.
///
/// At this phase in the parser, keywords are interpreted simply as identifiers.
/// This means that, in theory, this lexer can be used for any programming
/// language which uses usual characters and strings.
pub struct Lexer {
    /// The characters to tokenize.  This originates from the source string,
    /// provided at the creation of this lexer.
    chars: Vec<char>,

    /// The index of the current token, in the `chars` list.  This should be the
    /// index of the first character of the next token.
    idx: usize,

    /// List of comments.  The comments in this list will be added onto the next
    /// token found, and then this list will be cleared.
    comments: Vec<Comment>,
}

impl Lexer {
    /// Initializes a new lexer from the provided `source` string.  This
    /// function initializes the lexer with a default index of `0`.
    pub fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            idx: 0,
            comments: vec![],
        }
    }

    /// Returns whether or not `char` is a line breaking character.
    pub fn is_line_break(char: char) -> bool {
        match char {
            '\u{000A}' | '\u{000B}' | '\u{000C}' | '\u{000D}' | '\u{0085}' | '\u{2028}'
            | '\u{2029}' => true,
            _ => false,
        }
    }

    /// Returns whether or not `char` is a whitespace character, excluding any
    /// line breaking whitespace.
    pub fn is_whitespace(char: char) -> bool {
        match char {
            '\u{0009}' | '\u{0020}' | '\u{00A0}' | '\u{1680}' | '\u{2000}' | '\u{2001}'
            | '\u{2002}' | '\u{2003}' | '\u{2004}' | '\u{2005}' | '\u{2006}' | '\u{2007}'
            | '\u{2008}' | '\u{2009}' | '\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' => true,
            _ => false,
        }
    }

    /// Returns whether or not `char` is an identifier starting character.
    /// Checks if `char` is an XID_Start character or an underscore (`_`).
    pub fn is_iden(char: char) -> bool {
        UnicodeXID::is_xid_start(char) || char == '_'
    }

    /// Returns whether or not `char` is a punctuator.
    pub fn is_punct(char: char) -> bool {
        match char {
            '!' | '@' | '#' | '$' | '%' | '&' | '*' | ';' | ':' | ',' | '.' | '<' | '>' | '/'
            | '|' | '-' | '=' | '+' | '?' | '~' => true,
            _ => false,
        }
    }

    /// Returns whether or not `char` is a digit.
    pub fn is_digit(char: char) -> bool {
        match char {
            '0'..='9' => true,
            _ => false,
        }
    }

    /// Returns whether or not `char` is a hexadecimal digit.
    pub fn is_hex_digit(char: char) -> bool {
        match char {
            '0'..='9' => true,
            'a'..='f' => true,
            'A'..='F' => true,
            _ => false,
        }
    }

    /// Returns whether or not `char` is a binary digit.
    pub fn is_bin_digit(char: char) -> bool {
        match char {
            '0' | '1' => true,
            _ => false,
        }
    }

    /// Skips a single line or documentation comment.
    fn skip_line_comment(&mut self) -> Skipped {
        let start_index = self.idx - 2; // the index of the first character of the comment.
        let mut doc = false; // whether or not the comment is a doc comment.
        let mut value = String::new(); // the value of the comment.

        if self.idx < self.chars.len() && self.chars[self.idx] == '/' {
            doc = true;
            self.idx += 1;
        }

        while self.idx < self.chars.len() && self.chars[self.idx] != '\n' {
            self.idx += 1;
            value.push(self.chars[self.idx]);
        }

        Skipped::Comment(Comment {
            loc: start_index..self.idx,
            value: value.trim().to_string(),
            kind: match doc {
                true => CommentKind::Line,
                false => CommentKind::Doc,
            },
        })
    }

    /// Skips a single block comment.
    fn skip_block_comment(&mut self) -> Result<Skipped, Diagnostic<()>> {
        let start_index = self.idx - 2; // the index of the first character of this comment
        let mut value = String::new(); // the value of this comment.

        loop {
            if self.idx >= self.chars.len() {
                return Err(Diagnostic::error()
                    .with_code("E0001")
                    .with_labels(vec![
                        Label::primary((), self.idx..self.idx)
                            .with_message("expected block comment to end here"),
                        Label::secondary((), start_index..start_index + 2)
                            .with_message("help: block comment started here"),
                    ])
                    .with_message("block comment never ends"));
            }

            if self.chars[self.idx] == '*' {
                // could end the block comment?
                self.idx += 1;

                if self.idx >= self.chars.len() {
                    return Err(Diagnostic::error()
                        .with_code("E0001")
                        .with_labels(vec![
                            Label::primary((), self.idx..self.idx)
                                .with_message("expected block comment to end here"),
                            Label::secondary((), start_index..start_index + 2)
                                .with_message("help: block comment started here"),
                        ])
                        .with_message("block comment never ends"));
                }

                if self.chars[self.idx] != '/' {
                    value.push('*');
                    value.push(self.chars[self.idx]);
                    self.idx += 1;
                    continue;
                }

                self.idx += 1;

                break;
            }

            value.push(self.chars[self.idx]);
            self.idx += 1;
        }

        Ok(Skipped::Comment(Comment {
            loc: start_index..self.idx,
            value: value.trim().to_string(),
            kind: CommentKind::Block,
        }))
    }

    /// Skips a single skippable token, such as a whitespace, line break or
    /// comment.  Returns information about the skipped token, if any.
    fn skip_token(&mut self) -> Result<Skipped, Diagnostic<()>> {
        if self.idx >= self.chars.len() {
            return Ok(Skipped::None);
        }

        let first_char = self.chars[self.idx];

        if Lexer::is_whitespace(first_char) {
            self.idx += 1;
            return Ok(Skipped::Whitespace);
        }

        if Lexer::is_line_break(first_char) {
            self.idx += 1;
            return Ok(Skipped::LineBreak);
        }

        if first_char == '/' {
            if self.idx + 1 >= self.chars.len() {
                return Ok(Skipped::None);
            }

            let second_char = self.chars[self.idx + 1];
            self.idx += 2;

            if second_char == '/' {
                // line comment

                return Ok(self.skip_line_comment());
            } else if second_char == '*' {
                // block comment

                return self.skip_block_comment();
            }
        }

        Ok(Skipped::None)
    }

    /// Skips all skippable tokens until the next token is found.
    fn skip(&mut self) -> Result<(), Diagnostic<()>> {
        loop {
            let result = self.skip_token();

            match result {
                Ok(skipped) => match skipped {
                    Skipped::Comment(comment) => {
                        self.comments.push(comment);
                    }
                    Skipped::None => return Ok(()),
                    _ => {}
                },
                Err(err) => return Err(err),
            }
        }
    }

    /// Returns the spacing to the next token.
    fn spacing(&mut self) -> Result<Spacing, Diagnostic<()>> {
        let mut has_whitespace = false;

        loop {
            let result = self.skip_token();

            match result {
                Ok(skipped) => match skipped {
                    Skipped::Comment(comment) => {
                        has_whitespace = true;
                        self.comments.push(comment);
                    }
                    Skipped::Whitespace => has_whitespace = true,
                    Skipped::LineBreak => return Ok(Spacing::LineBreak),
                    Skipped::None => {
                        if has_whitespace {
                            return Ok(Spacing::Whitespace);
                        } else {
                            return Ok(Spacing::None);
                        }
                    }
                },
                Err(err) => return Err(err),
            }
        }
    }

    /// Gets all comments from the `comments` array and returns them, after
    /// clearing the `comments` array.
    fn get_comments(&mut self) -> Vec<Comment> {
        let comments = self.comments.clone();
        self.comments.clear();
        comments
    }

    /// Tokenizes an identifier token.
    fn tokenize_iden(&mut self) -> Result<TokenTree, Diagnostic<()>> {
        let mut value = String::new();
        let start_index = self.idx;

        while self.idx < self.chars.len() && UnicodeXID::is_xid_continue(self.chars[self.idx]) {
            value.push(self.chars[self.idx]);
            self.idx += 1;
        }

        Ok(TokenTree::Iden(Iden {
            loc: start_index..self.idx,
            value,
            comments: self.get_comments(),
            spacing: match self.spacing() {
                Ok(spacing) => spacing,
                Err(err) => return Err(err),
            },
        }))
    }

    /// Tokenizes a hexadecimal number.
    fn tokenize_hexadecimal(&mut self) -> Result<TokenTree, Diagnostic<()>> {
        let start_index = self.idx - 2;

        if self.idx >= self.chars.len() {
            return Err(Diagnostic::error()
                .with_code("E0008")
                .with_labels(vec![Label::primary((), start_index..self.idx)
                    .with_message("expected a hexadecimal number here")])
                .with_message("no hexadecimal number after `0x`"));
        }

        let mut first = true;
        let mut number = String::new();

        loop {
            if self.idx < self.chars.len() {
                if first {
                    return Err(Diagnostic::error()
                        .with_code("E0008")
                        .with_labels(vec![Label::primary((), start_index..self.idx - 1)
                            .with_message("expected a hexadecimal number here")])
                        .with_message("no hexadecimal number after `0x`"));
                } else {
                    break;
                }
            }

            if !Lexer::is_hex_digit(self.chars[self.idx]) {
                if first {
                    return Err(Diagnostic::error()
                        .with_code("E0008")
                        .with_labels(vec![Label::primary((), start_index..self.idx - 1)
                            .with_message("expected a hexadecimal number here")])
                        .with_message("no hexadecimal number after `0x`"));
                } else {
                    break;
                }
            }

            self.idx += 1;
            number.push(self.chars[self.idx]);
            first = false;
        }

        match i64::from_str_radix(&number, 16) {
            Ok(value) => Ok(TokenTree::Int(Int {
                loc: start_index..self.idx,
                kind: IntKind::Hexadecimal,
                value,
                comments: self.get_comments(),
                spacing: match self.spacing() {
                    Ok(spacing) => spacing,
                    Err(err) => return Err(err),
                },
            })),
            Err(_) => Err(Diagnostic::error()
                .with_code("E0009")
                .with_labels(vec![Label::primary((), start_index..self.idx)
                    .with_message("hexadecimal number is too large")])
                .with_message("hexadecimal number is too large.")),
        }
    }

    /// Tokenizes a binary number.
    fn tokenize_binary(&mut self) -> Result<TokenTree, Diagnostic<()>> {
        let start_index = self.idx - 2;

        if self.idx >= self.chars.len() {
            return Err(Diagnostic::error()
                .with_code("E0008")
                .with_labels(vec![Label::primary((), start_index..self.idx)
                    .with_message("expected a binary number here")])
                .with_message("no binary number after `0b`"));
        }

        let mut first = true;
        let mut number = String::new();

        loop {
            if self.idx < self.chars.len() {
                if first {
                    return Err(Diagnostic::error()
                        .with_code("E0008")
                        .with_labels(vec![Label::primary((), start_index..self.idx - 1)
                            .with_message("expected a binary number here")])
                        .with_message("no binary number after `0b`"));
                } else {
                    break;
                }
            }

            if !Lexer::is_bin_digit(self.chars[self.idx]) {
                if first {
                    return Err(Diagnostic::error()
                        .with_code("E0008")
                        .with_labels(vec![Label::primary((), start_index..self.idx - 1)
                            .with_message("expected a binary number here")])
                        .with_message("no binary number after `0b`"));
                } else {
                    break;
                }
            }

            self.idx += 1;
            number.push(self.chars[self.idx]);
            first = false;
        }

        match i64::from_str_radix(&number, 2) {
            Ok(value) => Ok(TokenTree::Int(Int {
                loc: start_index..self.idx,
                kind: IntKind::Binary,
                value,
                comments: self.get_comments(),
                spacing: match self.spacing() {
                    Ok(spacing) => spacing,
                    Err(err) => return Err(err),
                },
            })),
            Err(_) => Err(Diagnostic::error()
                .with_code("E0009")
                .with_labels(vec![Label::primary((), start_index..self.idx)
                    .with_message("hexadecimal number is too large")])
                .with_message("hexadecimal number is too large.")),
        }
    }

    /// Tokenizes a single number token.
    fn tokenize_number(&mut self, negative: bool) -> Result<TokenTree, Diagnostic<()>> {
        let mut number = match negative {
            true => "-".to_string(),
            false => String::new(),
        };
        let first_char = self.chars[self.idx];
        let start_index = self.idx;

        if first_char == '0' {
            if self.idx + 1 >= self.chars.len() {
                number.push('0');
                return Ok(TokenTree::Int(Int {
                    loc: start_index..self.idx,
                    kind: IntKind::Decimal,
                    value: if negative { -0 } else { 0 },
                    comments: self.get_comments(),
                    spacing: match self.spacing() {
                        Ok(spacing) => spacing,
                        Err(err) => return Err(err),
                    },
                }));
            }

            if self.chars[self.idx + 1] == 'x' {
                self.idx += 2;
                return self.tokenize_hexadecimal();
            } else if self.chars[self.idx + 1] == 'b' {
                self.idx += 2;
                return self.tokenize_binary();
            } else {
                number.push('0');
                self.idx += 1;
            }
        }

        let mut is_float = false;

        'main_number_loop: loop {
            if self.idx >= self.chars.len() {
                break;
            }

            let current_char = self.chars[self.idx];

            if Lexer::is_digit(current_char) {
                number.push(current_char);
            } else if current_char == '.' {
                if is_float {
                    break; // second `.` in a number literal
                } else {
                    is_float = true;
                    number.push('.');
                }
            } else if current_char == 'e' || current_char == 'E' {
                if !is_float {
                    return Err(Diagnostic::error()
                        .with_code("E0003")
                        .with_labels(vec![Label::primary((), start_index..self.idx)
                            .with_message("integers may not have an exponent")])
                        .with_message("exponent after `.`"));
                }

                if self.chars[self.idx - 1] == '.' {
                    // an exponent may not immediately follow a `.`
                    self.idx += 1;

                    return Err(Diagnostic::error()
                        .with_code("E0002")
                        .with_labels(vec![
                            Label::primary((), start_index..self.idx)
                                .with_message("exponent cannot immediately follow `.`"),
                            Label::secondary((), self.idx - 2..self.idx - 2)
                                .with_message("try inserting a `0` after this `.`"),
                        ])
                        .with_message("exponent after `.`"));
                }

                number.push(current_char);
                self.idx += 1;

                if self.idx >= self.chars.len() {
                    return Err(Diagnostic::error()
                        .with_code("E0004")
                        .with_labels(vec![Label::primary((), start_index..self.idx)
                            .with_message("expected an exponent value or `+`/`-`")])
                        .with_message("expected an exponent value"));
                }

                let current_char = self.chars[self.idx];
                if current_char == '+' || current_char == '-' {
                    number.push(current_char);

                    self.idx += 1;
                }

                let mut first = false;
                loop {
                    if self.idx >= self.chars.len() {
                        if first {
                            return Err(Diagnostic::error()
                                .with_code("E0004")
                                .with_labels(vec![Label::primary((), start_index..self.idx)
                                    .with_message("expected an exponent value")])
                                .with_message("expected an exponent value"));
                        } else {
                            break 'main_number_loop;
                        }
                    }

                    if !Lexer::is_digit(self.chars[self.idx]) {
                        if first {
                            return Err(Diagnostic::error()
                                .with_code("E0005")
                                .with_labels(vec![Label::primary((), start_index..self.idx)
                                    .with_message("expected a valid exponent value (a number)")])
                                .with_message("expected a valid exponent value"));
                        } else {
                            break 'main_number_loop;
                        }
                    }

                    number.push(self.chars[self.idx]);
                    self.idx += 1;
                    first = false;
                }
            } else {
                break;
            }

            self.idx += 1;
        }

        let comments = self.get_comments();
        let number = number.replace("_", "");

        if is_float {
            match number.parse() {
                Ok(value) => Ok(TokenTree::Float(Float {
                    loc: start_index..self.idx,
                    value,
                    comments,
                    spacing: match self.spacing() {
                        Ok(spacing) => spacing,
                        Err(err) => return Err(err),
                    },
                })),
                Err(_) => Err(Diagnostic::error()
                    .with_code("E0006")
                    .with_labels(vec![Label::primary((), start_index..self.idx)
                        .with_message("float number is too large")])
                    .with_message("float is too large")),
            }
        } else {
            match number.parse() {
                Ok(value) => Ok(TokenTree::Int(Int {
                    loc: start_index..self.idx,
                    kind: IntKind::Decimal,
                    value,
                    comments,
                    spacing: match self.spacing() {
                        Ok(spacing) => spacing,
                        Err(err) => return Err(err),
                    },
                })),
                Err(_) => Err(Diagnostic::error()
                    .with_code("E0007")
                    .with_labels(vec![Label::primary((), start_index..self.idx)
                        .with_message("integer number is too large")])
                    .with_message("integer is too large")),
            }
        }
    }

    // Tokenizes a single string token.
    fn tokenize_string(&mut self) -> Result<TokenTree, Diagnostic<()>> {
        let start_index = self.idx;
        let quote = self.chars[start_index];

        let mut string = quote.to_string();
        self.idx += 1;

        loop {
            if self.idx >= self.chars.len() {
                return Err(Diagnostic::error()
                    .with_code("E0010")
                    .with_labels(vec![Label::primary((), start_index..self.idx)
                        .with_message("string never closes")])
                    .with_message("string never closes"));
            }

            if self.chars[self.idx] == quote {
                self.idx += 1;
                string.push(quote);
                break;
            } else if self.chars[self.idx] == '\\' {
                string.push('\\');

                self.idx += 1;
                if self.idx >= self.chars.len() {
                    return Err(Diagnostic::error()
                        .with_code("E0010")
                        .with_labels(vec![Label::primary((), start_index..self.idx)
                            .with_message("string never closes")])
                        .with_message("string never closes"));
                }

                string.push(self.chars[self.idx]);
                self.idx += 1;
            } else {
                string.push(self.chars[self.idx]);
                self.idx += 1;
            }
        }

        match unescape(&string) {
            Ok(value) => {
                return Ok(TokenTree::Str(Str {
                    loc: start_index..self.idx,
                    value,
                    comments: self.get_comments(),
                    spacing: match self.spacing() {
                        Ok(spacing) => spacing,
                        Err(err) => return Err(err),
                    },
                }))
            }
            Err(err) => match err {
                UnescapeError::InvalidEscape { index, .. } => {
                    let index = start_index + index;

                    return Err(Diagnostic::error()
                        .with_code("E0011")
                        .with_labels(vec![Label::primary((), index..index)
                            .with_message("invalid string escape here")])
                        .with_message("invalid string escape"));
                }
                UnescapeError::InvalidUnicode { index, .. } => {
                    let index = start_index + index;
                    return Err(Diagnostic::error()
                        .with_code("E0012")
                        .with_labels(vec![Label::primary((), index..index)
                            .with_message("invalid unicode escape here")])
                        .with_message("invalid unicode escape in string"));
                }
            },
        }
    }

    /// Tokenizes a group token.
    fn tokenize_group(&mut self, close: char) -> Result<TokenTree, Diagnostic<()>> {
        let start_index = self.idx;
        let mut tokens = vec![];

        self.idx += 1;

        loop {
            if self.idx >= self.chars.len() {
                return Err(Diagnostic::error()
                    .with_code("E0014")
                    .with_labels(vec![
                        Label::primary((), start_index..self.idx)
                            .with_message(format!("group never closes with '{}'", close)),
                        Label::secondary((), start_index..start_index)
                            .with_message("group starts here"),
                    ])
                    .with_message("group never ends"));
            }

            if self.chars[self.idx] == close {
                self.idx += 1;
                break;
            }

            if let Some(result) = self.tokenize() {
                match result {
                    Ok(token) => tokens.push(token),
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(TokenTree::Group(Group {
            loc: start_index..self.idx,
            tokens,
            comments: self.get_comments(),
            spacing: match self.spacing() {
                Ok(spacing) => spacing,
                Err(err) => return Err(err),
            },
        }))
    }

    /// Tokenizes a single token from the `chars` list, then returns it, if
    /// there was another token and it was valid.
    fn tokenize(&mut self) -> Option<Result<TokenTree, Diagnostic<()>>> {
        if let Err(err) = self.skip() {
            return Some(Err(err));
        }

        if self.idx >= self.chars.len() {
            return None;
        }

        let first_char = self.chars[self.idx];
        let start_index = self.idx;

        if Lexer::is_iden(first_char) {
            Some(self.tokenize_iden())
        } else if Lexer::is_punct(first_char) {
            self.idx += 1;

            if first_char == '-' {
                if self.idx < self.chars.len() {
                    if Lexer::is_digit(self.chars[self.idx]) {
                        return Some(self.tokenize_number(true));
                    }
                }
            }

            Some(Ok(TokenTree::Punct(Punct {
                loc: start_index..self.idx,
                value: first_char,
                comments: self.get_comments(),
                spacing: match self.spacing() {
                    Ok(spacing) => spacing,
                    Err(err) => return Some(Err(err)),
                },
            })))
        } else if Lexer::is_digit(first_char) {
            Some(self.tokenize_number(false))
        } else if first_char == '"' || first_char == '\'' {
            Some(self.tokenize_string())
        } else if first_char == '{' || first_char == '[' || first_char == '(' {
            Some(self.tokenize_group(match first_char {
                '{' => '}',
                '[' => ']',
                '(' => ')',
                _ => unreachable!(),
            }))
        } else {
            Some(Err(Diagnostic::error()
                .with_code("E0013")
                .with_labels(vec![Label::primary((), start_index..start_index)
                    .with_message("invalid character here")])
                .with_message("invalid character")))
        }
    }
}

impl Iterator for Lexer {
    type Item = Result<TokenTree, Diagnostic<()>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokenize()
    }
}
