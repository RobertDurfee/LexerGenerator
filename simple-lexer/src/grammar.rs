use std::{
    collections::BTreeMap as Map,
    str::FromStr,
};
use lazy_static::lazy_static;
use segment_map;
use regular_expression::{
    sym as rsym,
    neg as rneg,
    alt as ralt,
    con as rcon,
    ast as rast,
    plu as rplu,
    sgl as rsgl,
    rng as rrng,
    all as rall,
    Expression,
    Re,
};
use simple_parser_bootstrap::{
    tok as ptok,
    non as pnon,
    alt as palt,
    con as pcon,
    ast as past,
    ParseTree,
};

type Result<T> = std::result::Result<T, &'static str>;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TokenKind {
    REGULAR_EXPRESSION,
    PRODUCTION_OPERATOR,
    TOKEN_KIND,
    SEMICOLON,
}
use TokenKind::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Nonterminal {
    Root,
    Production,
    Consumption,
}
use Nonterminal::*;

pub fn as_productions<T: FromStr>(parse_tree: &ParseTree<Nonterminal, TokenKind>) -> Result<Map<Expression, Option<T>>> {
    if let ParseTree::Nonterminal { nonterminal, children, .. } = parse_tree {
        match nonterminal {
            // Root ::= (Production | Consumption)*;
            Root => {
                let mut productions = Map::new();
                for child in children {
                    productions.extend(as_productions(child)?);
                }
                Ok(productions)
            },
            // Production ::= REGULAR_EXPRESSION PRODUCTION_OPERATOR TOKEN_KIND SEMICOLON;
            Production => {
                Ok(map![as_expression(&children[0])? => Some(as_token_kind(&children[2])?)])
            },
            // Consumption ::= REGULAR_EXPRESSION PRODUCTION_OPERATOR SEMICOLON;
            Consumption => {
                Ok(map![as_expression(&children[0])? => None])
            },
        }
    } else { Err("no productions") }
}

fn as_expression(parse_tree: &ParseTree<Nonterminal, TokenKind>) -> Result<Expression> {
    if let ParseTree::Token { token } = parse_tree {
        // /\/([^\/\n\r\\]|\\.)*\// => REGULAR_EXPRESSION;
        if let REGULAR_EXPRESSION = token.kind() {
            Ok(Re::new(&token.text()[1..token.text().len()-1])?.into_expression())
        } else { Err("not expression") }
    } else { Err("not expression") }
}

fn as_token_kind<T: FromStr>(parse_tree: &ParseTree<Nonterminal, TokenKind>) -> Result<T> {
    if let ParseTree::Token { token } = parse_tree {
        // /[A-Z][0-9A-Z_]*/ => TOKEN_KIND;
        if let TOKEN_KIND = token.kind() {
            if let Ok(token_kind) = T::from_str(token.text()) {
                Ok(token_kind)
            } else { Err("not token kind") }
        } else { Err("not token kind") }
    } else { Err("not token kind") }
}

lazy_static! {
    // /\/([^\/\n\r\\]|\\.)+\// => REGULAR_EXPRESSION;
    // /=>/ => PRODUCTION_OPERATOR;
    // /[A-Z][0-9A-Z_]*/ => TOKEN_KIND;
    // /;/ => SEMICOLON;
    // /[\n\r\t ]/ => ;
    // /\/\/[^\n\r]*/ => ;
    pub(crate) static ref LEXER_PRODUCTIONS: Map<Expression, Option<TokenKind>> = map![
        rcon![
            rsym![rsgl!('/')],
            rplu!(ralt![
                rneg![
                    rsgl!('/'),
                    rsgl!('\n'),
                    rsgl!('\r'),
                    rsgl!('\\')
                ],
                rcon![
                    rsym![rsgl!('\\')],
                    rsym![rall!()]
                ]
            ]),
            rsym![rsgl!('/')]
        ] => Some(REGULAR_EXPRESSION),
        rcon![
            rsym![rsgl!('=')],
            rsym![rsgl!('>')]
        ] => Some(PRODUCTION_OPERATOR),
        rcon![
            rsym![rrng!('A', 'Z')],
            rast!(rsym![
                rrng!('0', '9'),
                rrng!('A', 'Z'),
                rsgl!('_')
            ])
        ] => Some(TOKEN_KIND),
        rsym![rsgl!(';')] => Some(SEMICOLON),
        rsym![
            rsgl!('\n'),
            rsgl!('\r'),
            rsgl!('\t'),
            rsgl!(' ')
        ] => None,
        rcon![
            rsym![rsgl!('/')],
            rsym![rsgl!('/')],
            rast!(rneg![
                rsgl!('\n'),
                rsgl!('\r')
            ])
        ] => None
    ];

    // Root ::= (Production | Consumption)*;
    // Production ::= REGULAR_EXPRESSION PRODUCTION_OPERATOR TOKEN_KIND SEMICOLON;
    // Consumption ::= REGULAR_EXPRESSION PRODUCTION_OPERATOR SEMICOLON;
    pub(crate) static ref PARSER_PRODUCTIONS: Map<Nonterminal, simple_parser_bootstrap::Expression<Nonterminal, TokenKind>> = map![
        Root => past!(palt![
            pnon!(Production),
            pnon!(Consumption)
        ]),
        Production => pcon![
            ptok!(REGULAR_EXPRESSION),
            ptok!(PRODUCTION_OPERATOR),
            ptok!(TOKEN_KIND),
            ptok!(SEMICOLON)
        ],
        Consumption => pcon![
            ptok!(REGULAR_EXPRESSION),
            ptok!(PRODUCTION_OPERATOR),
            ptok!(SEMICOLON)
        ]
    ];
}
