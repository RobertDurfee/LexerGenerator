use std::collections::BTreeMap as Map;
use lazy_static::lazy_static;
use interval_map;
use re_bootstrap::{
    sym as rsym,
    neg as rneg,
    alt as ralt,
    con as rcon,
    ast as rast,
    sgl as rsgl,
    rng as rrng,
    all as rall,
    Re,
};
use lexer_bootstrap::{
    error::Result,
    map,
};
use parser_bootstrap::{
    tok as ptok,
    non as pnon,
    con as pcon,
    ast as past,
    ParseTree,
    Expression,
};

#[allow(non_camel_case_types)]
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub enum TokenKind {
    REGULAR_EXPRESSION,
    PRODUCTION_OPERATOR,
    TOKEN_KIND,
    SEMICOLON,
}
use TokenKind::*;

#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub enum Nonterminal {
    Grammar,
    Production,
}
use Nonterminal::*;

pub enum Grammar<T> {
    Phantom(T),
}

impl<T> Grammar<T> {
    pub fn new(_parse_tree: ParseTree<Nonterminal, TokenKind>) -> Result<Grammar<T>> {
        panic!("Not implemented")
    }

    pub fn productions(&self) -> Result<Map<Re, Option<T>>> {
        panic!("Not implemented")
    }

    fn re(&self) -> Result<Re> {
        panic!("Not implemented")
    }

    fn token_kind(&self) -> Result<Option<T>> {
        panic!("Not implemented")
    }
}

lazy_static! {
    // '"([^"\n\r\\]|\\.)*"' => REGULAR_EXPRESSION;
    // "'([^'\n\r\\]|\\.)*'" => REGULAR_EXPRESSION;
    // "=>" => PRODUCTION_OPERATOR;
    // "[A-Z][0-9A-Z_]*" => TOKEN_KIND;
    // ";" => SEMICOLON;
    // "[\n\r\t ]" => ;
    // "//[^\n\r]" => ;
    static ref LEXER_PRODUCTIONS: Map<Re, Option<TokenKind>> = map![
        rcon![
            rsym![rsgl!('"')],
            rast!(ralt![
                rneg![
                    rsgl!('"'),
                    rsgl!('\n'),
                    rsgl!('\r'),
                    rsgl!('\\')
                ],
                rcon![
                    rsym![rsgl!('\\')],
                    rsym![rall!()]
                ]
            ]),
            rsym![rsgl!('"')]
        ] => Some(REGULAR_EXPRESSION),
        rcon![
            rsym![rsgl!('\'')],
            rast!(ralt![
                rneg![
                    rsgl!('\''),
                    rsgl!('\n'),
                    rsgl!('\r'),
                    rsgl!('\\')
                ],
                rcon![
                    rsym![rsgl!('\\')],
                    rsym![rall!()]
                ]
            ]),
            rsym![rsgl!('\'')]
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
        rsym![rsgl!(':')] => Some(SEMICOLON),
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

    // Grammar ::= Production*;
    // Production ::= REGULAR_EXPRESSION PRODUCTION_OPERATOR TOKEN_KIND SEMICOLON;
    static ref PARSER_PRODUCTIONS: Map<Nonterminal, Expression<Nonterminal, TokenKind>> = map![
        Grammar => past!(pnon!(Production)),
        Production => pcon![
            ptok!(REGULAR_EXPRESSION),
            ptok!(PRODUCTION_OPERATOR),
            ptok!(TOKEN_KIND),
            ptok!(SEMICOLON)
        ]
    ];
}
