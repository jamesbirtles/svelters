use super::{fragment::FragmentState, State, StateTransition};
use crate::{
    error::ParseErrorKind,
    nodes::{ConstTag, Mustache, MustacheItem},
    parser::Parser,
    tokens::{ConstTagToken, MustacheCloseToken, MustacheOpenToken},
};
use swc_common::{source_map::BytePos, Spanned};
use swc_ecma_ast::{AssignOp, EsVersion, Expr};
use swc_ecma_parser::{lexer::Lexer, StringInput, Syntax, TsConfig};

#[derive(Debug, Default)]
pub struct MustacheState;

impl StateTransition for MustacheState {
    fn next_state(self, parser: &mut Parser<'_>) -> State {
        let start = parser.position();
        let mustache_open = MustacheOpenToken {
            span: parser
                .eat_char('{')
                .expect("should not enter state without moustache open"),
        };
        let leading_whitespace = parser.allow_whitespace();

        let mustache_item = if let Some(span) = parser.eat_chars("@const") {
            self.parse_const_tag(parser, ConstTagToken { span })
        } else {
            Some(self.parse_mustache_tag(parser))
        };
        let Some(mustache_item) = mustache_item else {
            // Some error trying to parse the mustache item, it should have been emitted, and so
            // we'll try to keep parsing from here as best we can.
            return FragmentState.into();
        };

        let trailing_whitespace = parser.allow_whitespace();

        let mustache_close = match parser.eat_char('}') {
            Some(span) => Some(MustacheCloseToken { span }),
            None => {
                parser.error(ParseErrorKind::MustacheNotClosed);
                None
            }
        };

        parser.push_node(Mustache {
            mustache_open,
            leading_whitespace,
            mustache_item,
            trailing_whitespace,
            mustache_close,
            span: parser.span_from(start),
        });
        FragmentState.into()
    }
}

impl MustacheState {
    fn parse_js_expression(&self, parser: &mut Parser<'_>) -> Box<Expr> {
        let source = parser.text();
        let mut ecma_parser = swc_ecma_parser::Parser::new_from(Lexer::new(
            Syntax::Typescript(TsConfig::default()),
            EsVersion::EsNext,
            StringInput::new(
                &source[parser.position()..],
                BytePos(parser.position() as u32),
                BytePos(source.len() as u32),
            ),
            None,
        ));

        let expression = ecma_parser.parse_expr().unwrap();
        parser.eat_to(expression.span_hi().0 as usize);
        expression
    }

    fn parse_mustache_tag(self, parser: &mut Parser<'_>) -> MustacheItem {
        self.parse_js_expression(parser).into()
    }

    fn parse_const_tag(
        self,
        parser: &mut Parser<'_>,
        const_tag: ConstTagToken,
    ) -> Option<MustacheItem> {
        let whitespace =
            parser.require_whitespace(ParseErrorKind::MissingWhitespaceAfterConstTag)?;
        let expression = self.parse_js_expression(parser);

        if !matches!(&*expression, Expr::Assign(expr) if expr.op == AssignOp::Assign) {
            parser.error_with_span(ParseErrorKind::InvalidConstArgs, expression.span());
        }

        let span = const_tag.span().with_hi(expression.span_hi());
        Some(
            ConstTag {
                const_tag,
                whitespace,
                expression,
                span,
            }
            .into(),
        )
    }
}