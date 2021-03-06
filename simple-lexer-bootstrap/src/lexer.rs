use std::collections::{
    BTreeSet as Set,
    BTreeMap as Map,
    VecDeque,
};
use segment_map::Segment;
use finite_automata::{
    Enfa,
    Dfa,
    Subsume,
    states_contains_from,
};
use regular_expression_bootstrap::Expression;
use crate::{
    TokenState,
    TokenStateGenerator,
};

type Result<T> = std::result::Result<T, &'static str>;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Token<T> {
    kind: T,
    text: String,
}

impl<T> Token<T> {
    pub fn new(kind: T, text: &str) -> Token<T> {
        Token { kind, text: String::from(text) }
    }

    pub fn kind(&self) -> &T {
        &self.kind
    }
    
    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Lexer<T> {
    productions: Map<Expression, Option<T>>,
    dfa: Dfa<Set<TokenState<T>>, u32>
}

impl<T: Clone + Ord> Lexer<T> {
    pub fn new(productions: Map<Expression, Option<T>>) -> Lexer<T> {
        let mut fas = Vec::new();
        for (expression, token) in &productions {
            fas.push(expression.as_enfa(&mut TokenStateGenerator::new(token.clone())));
        }
        let mut alt = Enfa::new(TokenState::new(None));
        for fa in fas {
            alt.subsume(&fa);
            let fa_initial_index = states_contains_from(&alt, &fa, fa.initial_index()).expect("state does not exist");
            alt.transitions_insert((alt.initial_index(), Segment::empty(), fa_initial_index));
            for fa_final_index in fa.final_indices() {
                let fa_final_index = states_contains_from(&alt, &fa, fa_final_index).expect("state does not exist");
                alt.set_final(fa_final_index);
            }
        }
        Lexer { productions, dfa: Dfa::from(&alt) }
    }

    pub fn lex(&self, text: &str) -> Result<Vec<Token<T>>> {
        let mut tokens = Vec::new();
        let mut token_text = String::from("");
        let mut characters: VecDeque<char> = text.chars().collect();
        let mut source_index = self.dfa.initial_index();
        while let Some(character) = characters.pop_front() {
            if let Some(transition_index) = self.dfa.transitions_contains_outgoing((source_index, &character.into())) {
                let (_, _, target_index) = self.dfa.transitions_index(transition_index);
                token_text.push(character);
                source_index = target_index;
            } else {
                if self.dfa.is_final(source_index) {
                    let mut token_kind = &None;
                    for token_state in self.dfa.states_index(source_index) {
                        if token_kind.is_none() {
                            token_kind = token_state.token_kind();
                        } else {
                            if token_state.token_kind().is_some() && token_state.token_kind() != token_kind {
                                return Err("inconsistent tokens in final state");
                            }
                        }
                    }
                    if let Some(token_kind) = token_kind {
                        tokens.push(Token::new(token_kind.clone(), token_text.as_str()));
                    }
                    token_text.clear();
                    characters.push_front(character);
                    source_index = self.dfa.initial_index();
                } else { return Err("partial match"); }
            }
        }
        if self.dfa.is_final(source_index) {
            let mut token_kind = &None;
            for token_state in self.dfa.states_index(source_index) {
                if token_kind.is_none() {
                    token_kind = token_state.token_kind();
                } else {
                    if token_state.token_kind().is_some() && token_state.token_kind() != token_kind {
                        return Err("inconsistent tokens in final state");
                    }
                }
            }
            if let Some(token_kind) = token_kind {
                tokens.push(Token::new(token_kind.clone(), token_text.as_str()));
            }
        } else { return Err("partial match"); }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use regular_expression_bootstrap::{
        sym,
        rep,
        con,
        neg,
        alt,
        sgl,
        rng,
        ast,
    };
    use crate::{
        Lexer,
        Token,
    };
    use super::Result;

    #[test]
    fn test_1() -> Result<()> {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        enum TokenKind {
            A,
            B,
        };
        use TokenKind::*;
        let lexer = Lexer::new(map![
            sym![sgl!('A')] => Some(A),
            sym![sgl!('B')] => Some(B),
            ast!(sym![sgl!(' ')]) => None
        ]);
        let expected = vec![
            Token::new(A, "A"),
            Token::new(B, "B"),
            Token::new(A, "A"),
        ];
        let actual = lexer.lex("A B  A   ")?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_2() -> Result<()> {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        #[allow(non_camel_case_types)]
        enum TokenKind {
            A_REP,
            B_REP
        };
        use TokenKind::*;
        let lexer = Lexer::new(map![
            ast!(sym![sgl!('A')]) => Some(A_REP),
            ast!(sym![sgl!('B')]) => Some(B_REP),
            ast!(sym![sgl!(' ')]) => None
        ]);
        let expected = vec![
            Token::new(A_REP, "AAAAAAA"), 
            Token::new(B_REP, "BBBB"),
            Token::new(B_REP, "BBBB"),
        ];
        let actual = lexer.lex("AAAAAAABBBB   BBBB")?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_3() -> Result<()> {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        #[allow(non_camel_case_types)]
        enum TokenKind {
            A,
            AB,
            BB,
            B,
        };
        use TokenKind::*;
        let lexer = Lexer::new(map![
            sym![sgl!('A')] => Some(A),
            con![sym![sgl!('A')], sym![sgl!('B')]] => Some(AB),
            con![sym![sgl!('B')], sym![sgl!('B')]] => Some(BB),
            sym![sgl!('B')] => Some(B)
        ]);
        let expected = vec![
            Token::new(AB, "AB"),
            Token::new(B, "B"),
        ];
        let actual = lexer.lex("ABB")?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_4() -> Result<()> {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        enum TokenKind {
            FULL_STOP,
            CARET,
            DOLLAR_SIGN,
            ASTERISK,
            PLUS_SIGN,
            HYPHEN,
            QUESTION_MARK,
            LEFT_PARENTHESIS,
            RIGHT_PARENTHESIS,
            LEFT_SQUARE_BRACKET,
            RIGHT_SQUARE_BRACKET,
            LEFT_CURLY_BRACKET,
            RIGHT_CURLY_BRACKET,
            VERTICAL_BAR,
            COMMA,
            DIGIT_LITERAL,
            UNESCAPED_LITERAL,
            ESCAPED_LITERAL,
            CONTROL_LITERAL,
            OCTAL_LITERAL,
            HEXADECIMAL_LITERAL,
            UNICODE_LITERAL,
        }
        use TokenKind::*;
        let lexer = Lexer::new(map![
            sym![sgl!('.')] => Some(FULL_STOP),
            sym![sgl!('^')] => Some(CARET),
            sym![sgl!('$')] => Some(DOLLAR_SIGN),
            sym![sgl!('*')] => Some(ASTERISK),
            sym![sgl!('+')] => Some(PLUS_SIGN),
            sym![sgl!('-')] => Some(HYPHEN),
            sym![sgl!('?')] => Some(QUESTION_MARK),
            sym![sgl!('(')] => Some(LEFT_PARENTHESIS),
            sym![sgl!(')')] => Some(RIGHT_PARENTHESIS),
            sym![sgl!('[')] => Some(LEFT_SQUARE_BRACKET),
            sym![sgl!(']')] => Some(RIGHT_SQUARE_BRACKET),
            sym![sgl!('{')] => Some(LEFT_CURLY_BRACKET),
            sym![sgl!('}')] => Some(RIGHT_CURLY_BRACKET),
            sym![sgl!('|')] => Some(VERTICAL_BAR),
            sym![sgl!(',')] => Some(COMMA),
            sym![rng!('0', '9')] => Some(DIGIT_LITERAL),
            neg![sgl!('.'), sgl!('^'), sgl!('$'), sgl!('*'), sgl!('+'), sgl!('-'), sgl!('?'), sgl!('('), sgl!(')'), sgl!('['), sgl!(']'), sgl!('{'), sgl!('}'), sgl!('|'), sgl!(','), rng!('0', '9')] => Some(UNESCAPED_LITERAL),
            con![sym![sgl!('\\')], sym![sgl!('.'), sgl!('^'), sgl!('$'), sgl!('*'), sgl!('+'), sgl!('-'), sgl!('?'), sgl!('('), sgl!(')'), sgl!('['), sgl!(']'), sgl!('{'), sgl!('}'), sgl!('|')]] => Some(ESCAPED_LITERAL),
            con![sym![sgl!('\\')], sym![sgl!('n'), sgl!('r'), sgl!('t')]] => Some(CONTROL_LITERAL),
            con![sym![sgl!('\\')], rep!(sym![rng!('0', '7')], Some(1), Some(3))] => Some(OCTAL_LITERAL),
            con![sym![sgl!('\\')], sym![sgl!('x')], rep!(sym![rng!('0', '9'), rng!('a', 'f'), rng!('A', 'F')], Some(1), Some(2))] => Some(HEXADECIMAL_LITERAL),
            con![sym![sgl!('\\')], alt![con![sym![sgl!('u')], rep!(sym![rng!('0', '9'), rng!('a', 'f'), rng!('A', 'F')], Some(4), Some(4))], con![sym![sgl!('U')], rep!(sym![rng!('0', '9'), rng!('a', 'f'), rng!('A', 'F')], Some(8), Some(8))]]] => Some(UNICODE_LITERAL)
        ]);
        let expected = vec![
            Token::new(LEFT_SQUARE_BRACKET, "["),
            Token::new(UNESCAPED_LITERAL, "A"),
            Token::new(UNESCAPED_LITERAL, "🦄"),
            Token::new(ESCAPED_LITERAL, "\\."),
            Token::new(RIGHT_SQUARE_BRACKET, "]"),
            Token::new(LEFT_CURLY_BRACKET, "{"),
            Token::new(DIGIT_LITERAL, "1"),
            Token::new(COMMA, ","),
            Token::new(DIGIT_LITERAL, "2"),
            Token::new(RIGHT_CURLY_BRACKET, "}"),
            Token::new(UNICODE_LITERAL, "\\UDEADBEEF"),
            Token::new(OCTAL_LITERAL, "\\777"),
            Token::new(HEXADECIMAL_LITERAL, "\\x45"),
        ];
        let actual = lexer.lex("[A🦄\\.]{1,2}\\UDEADBEEF\\777\\x45")?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
