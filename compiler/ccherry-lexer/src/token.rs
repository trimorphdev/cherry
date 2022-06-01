//! Tokens for the Cherry lexer.

use std::ops::Range;

pub type Loc = Range<usize>;

/// The spacing between this token and the next token.
#[derive(Clone, Debug, PartialEq)]
pub enum Spacing {
    /// Either there is no token after this one, or there is no whitespace
    /// between this token and the next token.
    None,

    /// There is whitespace between this token and the next.
    Whitespace,

    /// There is a line break between this token and the next.
    LineBreak,
}

/// What comment syntax was used.
#[derive(Clone, Debug, PartialEq)]
pub enum CommentKind {
    /// The comment started with `//`.
    Line,

    /// The comment started with `///`.
    Doc,

    /// A block comment, which starts with `/*` and ends with `*/`.
    Block,
}

/// A comment token.
///
/// This will never be outputted directly by the lexer.  Comments may be found
/// in tokens that have comments before them.
#[derive(Clone, Debug, PartialEq)]
pub struct Comment {
    /// The location of this comment.
    pub loc: Loc,

    /// The value of this comment, without any trailing whitespace or parts of
    /// the comment syntax, such as `//` or `/**/`.
    pub value: String,

    /// What kind of comment this is.
    pub kind: CommentKind,
}

/// Information about a token which was skipped.
#[derive(Clone, Debug, PartialEq)]
pub enum Skipped {
    /// A comment token was skipped.
    Comment(Comment),

    /// A whitespace or line break token was skipped.
    Whitespace,

    /// A line breaking token.
    LineBreak,

    /// Nothing was skipped, the current character is not skippable.
    None,
}

/// An identifier literal token.
#[derive(Clone, Debug, PartialEq)]
pub struct Iden {
    /// The location of this identifier.
    pub loc: Loc,

    /// The value of this identifier.
    pub value: String,

    /// The comments before this identifier.
    pub comments: Vec<Comment>,

    /// The spacing of this identifier.
    pub spacing: Spacing,
}

/// A punctuation token.
#[derive(Clone, Debug, PartialEq)]
pub struct Punct {
    /// The location of this punctuator.
    pub loc: Loc,

    /// The value of this punctuator.
    pub value: char,

    /// The comments before this punctuator.
    pub comments: Vec<Comment>,

    /// The spacing of this punctuator.
    pub spacing: Spacing,
}

/// Whether an integer is a decimal, hexadecimal or binary literal.
#[derive(Clone, Debug, PartialEq)]
pub enum IntKind {
    /// A decimal literal.
    Decimal,

    /// A hexadecimal literal.
    Hexadecimal,

    /// A binary literal.
    Binary,
}

/// An integer literal token.
///
/// By this point, the lexer has already converted this token to a usable
/// integer value, rather than keeping it as a string.
#[derive(Clone, Debug, PartialEq)]
pub struct Int {
    /// The location of this integer literal.
    pub loc: Loc,

    /// The kind of this integer literal.
    pub kind: IntKind,

    /// The value of this integer literal.
    pub value: i64,

    /// The comments before this integer literal.
    pub comments: Vec<Comment>,

    /// The spacing of this integer literal.
    pub spacing: Spacing,
}

/// A float literal token.
#[derive(Clone, Debug, PartialEq)]
pub struct Float {
    /// The location of this float literal.
    pub loc: Loc,

    /// The value of this float literal.
    pub value: f64,

    /// The comments before this float literal.
    pub comments: Vec<Comment>,

    /// The spacing of this float literal.
    pub spacing: Spacing,
}

/// A string token.
#[derive(Clone, Debug, PartialEq)]
pub struct Str {
    /// The location of this string literal.
    pub loc: Loc,

    /// The (unescaped) value of this string literal.
    pub value: String,

    /// The comments before this string literal.
    pub comments: Vec<Comment>,

    /// The spacing of this string literal.
    pub spacing: Spacing,
}

/// A group token.
#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    /// The location of this group.
    pub loc: Loc,

    /// The (unescaped) value of this group.
    pub tokens: Vec<TokenTree>,

    /// The comments before this group.
    pub comments: Vec<Comment>,

    /// The spacing of this group.
    pub spacing: Spacing,
}

/// A tree of tokens.
#[derive(Clone, Debug, PartialEq)]
pub enum TokenTree {
    /// An identifier token.
    Iden(Iden),

    /// An punctuation token.
    Punct(Punct),

    /// An integer literal token.
    Int(Int),

    /// A float literal token.
    Float(Float),

    /// A string literal token.
    Str(Str),

    /// A group token.
    Group(Group),
}
