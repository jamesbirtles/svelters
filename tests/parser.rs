use svelters::{
    error::{CollectingErrorReporter, ParseError, ParseErrorKind},
    nodes::{Comment, CommentText, ConstTag, Mustache, Text},
    parser::{new_span, Parser},
    tokens::{
        CommentEndToken, CommentStartToken, ConstTagToken, MustacheCloseToken, MustacheOpenToken,
        WhitespaceToken,
    },
};
use swc_ecma_ast::{Expr, Ident};

#[test]
fn fragment() {
    let mut error_reporter = CollectingErrorReporter::new();
    let nodes = Parser::new("Hello, {world}!", &mut error_reporter).parse();
    assert_eq!(nodes.len(), 3);
    assert!(error_reporter.is_empty())
}

#[test]
fn text() {
    let mut error_reporter = CollectingErrorReporter::new();
    let nodes = Parser::new("Hello, world!", &mut error_reporter).parse();

    assert_eq!(
        nodes,
        vec![Text {
            text: "Hello, world!".into(),
            span: new_span(0, 13),
        }
        .into()]
    );
    assert!(error_reporter.is_empty())
}

#[test]
fn mustache_expression() {
    let mut error_reporter = CollectingErrorReporter::new();
    let nodes = Parser::new("{hello}", &mut error_reporter).parse();
    let expected_node = Mustache {
        mustache_open: MustacheOpenToken {
            span: new_span(0, 1),
        },
        leading_whitespace: None,
        mustache_item: Box::new(Expr::Ident(Ident::new("hello".into(), new_span(1, 6)))).into(),
        trailing_whitespace: None,
        mustache_close: Some(MustacheCloseToken {
            span: new_span(6, 7),
        }),
        span: new_span(0, 7),
    };

    assert_eq!(nodes, vec![expected_node.into()]);
    assert!(error_reporter.is_empty())
}

#[test]
fn mustache_expression_whitespace() {
    let mut error_reporter = CollectingErrorReporter::new();
    let nodes = Parser::new("{  hello   }", &mut error_reporter).parse();
    let expected_node = Mustache {
        mustache_open: MustacheOpenToken {
            span: new_span(0, 1),
        },
        leading_whitespace: Some(WhitespaceToken {
            span: new_span(1, 3),
        }),
        mustache_item: Box::new(Expr::Ident(Ident::new("hello".into(), new_span(3, 8)))).into(),
        trailing_whitespace: Some(WhitespaceToken {
            span: new_span(8, 11),
        }),
        mustache_close: Some(MustacheCloseToken {
            span: new_span(11, 12),
        }),
        span: new_span(0, 12),
    };

    assert_eq!(nodes, vec![expected_node.into()]);
    assert!(error_reporter.is_empty())
}

#[test]
fn mustache_expression_missing_close() {
    let mut error_reporter = CollectingErrorReporter::new();
    let nodes = Parser::new("{hello", &mut error_reporter).parse();
    let expected_node = Mustache {
        mustache_open: MustacheOpenToken {
            span: new_span(0, 1),
        },
        leading_whitespace: None,
        mustache_item: Box::new(Expr::Ident(Ident::new("hello".into(), new_span(1, 6)))).into(),
        trailing_whitespace: None,
        mustache_close: None,
        span: new_span(0, 6),
    };

    assert_eq!(nodes, vec![expected_node.into()]);
    assert_eq!(
        error_reporter.parse_errors(),
        &[ParseError::new(
            ParseErrorKind::MustacheNotClosed,
            new_span(5, 6)
        )]
    );
}

#[test]
fn comment() {
    let mut error_reporter = CollectingErrorReporter::new();
    let nodes = Parser::new("<!-- a comment -->", &mut error_reporter).parse();

    assert_eq!(
        nodes,
        vec![Comment {
            comment_start: CommentStartToken {
                span: new_span(0, 4)
            },
            comment_text: CommentText {
                text: " a comment ".into(),
                span: new_span(4, 15)
            },
            comment_end: Some(CommentEndToken {
                span: new_span(15, 18)
            }),
            span: new_span(0, 18),
        }
        .into()]
    );
    assert!(error_reporter.is_empty())
}

#[test]
fn mustache_const_tag() {
    let mut error_reporter = CollectingErrorReporter::new();
    let nodes = Parser::new("{@const hello}", &mut error_reporter).parse();
    let expected_node = Mustache {
        mustache_open: MustacheOpenToken {
            span: new_span(0, 1),
        },
        leading_whitespace: None,
        mustache_item: ConstTag {
            const_tag: ConstTagToken {
                span: new_span(1, 7),
            },
            whitespace: WhitespaceToken {
                span: new_span(7, 8),
            },
            expression: Box::new(Expr::Ident(Ident::new("hello".into(), new_span(8, 13)))),
            span: new_span(1, 13),
        }
        .into(),
        trailing_whitespace: None,
        mustache_close: Some(MustacheCloseToken {
            span: new_span(13, 14),
        }),
        span: new_span(0, 14),
    };

    assert_eq!(nodes, vec![expected_node.into()]);
    assert_eq!(
        error_reporter.parse_errors(),
        &[ParseError::new(
            ParseErrorKind::InvalidConstArgs,
            new_span(8, 13)
        )]
    );
}