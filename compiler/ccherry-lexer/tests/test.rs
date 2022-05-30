extern crate ccherry_lexer;

use ccherry_lexer::{Comment, CommentKind, Float, Group, Iden, Int, IntKind, Lexer, Spacing, TokenTree};

#[test]
fn iden() {
    let mut lexer = Lexer::new("test identifier");

    assert_eq!(lexer.next(), Some(Ok(TokenTree::Iden(Iden {
        loc: 0..4,
        value: "test".to_string(),
        comments: vec![],
        spacing: Spacing::Whitespace,
    }))));

    assert_eq!(lexer.next(), Some(Ok(TokenTree::Iden(Iden {
        loc: 5..15,
        value: "identifier".to_string(),
        comments: vec![],
        spacing: Spacing::None,
    }))));
}

#[test]
fn only_comment() {
    let mut lexer = Lexer::new("/* test comment */");

    assert_eq!(lexer.next(), None);
}

#[test]
fn comment_before_iden() {
    let mut lexer = Lexer::new("/* test comment */ function");

    assert_eq!(lexer.next(), Some(Ok(TokenTree::Iden(Iden {
        loc: 19..27,
        value: "function".to_string(),
        comments: vec![
            Comment {
                loc: 0..18,
                value: "test comment".to_string(),
                kind: CommentKind::Block,
            }
        ],
        spacing: Spacing::None,
    }))));
}

#[test]
fn integer() {
    let mut lexer = Lexer::new("1234 4321");

    assert_eq!(lexer.next(), Some(Ok(TokenTree::Int(Int {
        loc: 0..4,
        kind: IntKind::Decimal,
        value: 1234,
        comments: vec![],
        spacing: Spacing::Whitespace,
    }))));

    assert_eq!(lexer.next(), Some(Ok(TokenTree::Int(Int {
        loc: 5..9,
        kind: IntKind::Decimal,
        value: 4321,
        comments: vec![],
        spacing: Spacing::None,
    }))));
}

#[test]
fn float() {
    let mut lexer = Lexer::new("1234.0213 4321.432");

    assert_eq!(lexer.next(), Some(Ok(TokenTree::Float(Float {
        loc: 0..9,
        value: 1234.0213,
        comments: vec![],
        spacing: Spacing::Whitespace,
    }))));

    assert_eq!(lexer.next(), Some(Ok(TokenTree::Float(Float {
        loc: 10..18,
        value: 4321.432,
        comments: vec![],
        spacing: Spacing::None,
    }))));
}

#[test]
fn code_block_group() {
    let mut lexer = Lexer::new("{ iden }");

    assert_eq!(lexer.next(), Some(Ok(TokenTree::Group(Group {
        loc: 0..8,
        tokens: vec![
            TokenTree::Iden(Iden {
                loc: 2..6,
                value: "iden".to_string(),
                comments: vec![],
                spacing: Spacing::Whitespace,
            })
        ],
        comments: vec![],
        spacing: Spacing::None,
    }))));
}