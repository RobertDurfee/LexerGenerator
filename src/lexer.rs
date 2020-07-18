use std::collections::{
    BTreeSet as Set,
    BTreeMap as Map,
    VecDeque,
};
use interval_map::Interval;
use finite_automata::{
    Enfa,
    Dfa,
    Subsume,
    states_contains_from,
};
use regular_expression::Re;
use crate::{
    error::{
        Result,
        Error,
        ErrorKind,
    },
    TokenState,
    TokenStateGenerator,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Lexer<T> {
    productions: Map<Re, Option<T>>,
    dfa: Option<Dfa<Set<TokenState<T>>, u32>>
}

impl<T> From<Map<Re, Option<T>>> for Lexer<T> {
    fn from(productions: Map<Re, Option<T>>) -> Lexer<T> {
        Lexer { productions, dfa: None }
    }
}

impl<T: Clone + Ord> Lexer<T> {
    pub fn parse(_productions: &str) -> Lexer<T> {
        panic!("Not implemented")
    }

    pub fn compile(&mut self) {
        if self.dfa.is_none() {
            let mut res = Vec::new();
            for (re, token) in &self.productions {
                res.push(re.as_enfa(&mut TokenStateGenerator::new(token.clone())));
            }
            let mut alt = Enfa::new(TokenState::new(None));
            for re in res {
                alt.subsume(&re);
                let re_initial_index = states_contains_from(&alt, &re, re.initial_index()).expect("state does not exist");
                alt.transitions_insert((alt.initial_index(), Interval::empty(), re_initial_index));
                for re_final_index in re.final_indices() {
                    let re_final_index = states_contains_from(&alt, &re, re_final_index).expect("state does not exist");
                    alt.set_final(re_final_index);
                }
            }
            self.dfa = Some(Dfa::from(&alt));
        }
    }

    pub fn lex(&self, text: &str) -> Result<Vec<T>> {
        if let Some(dfa) = self.dfa.as_ref() {
            let mut tokens = Vec::new();
            let mut characters: VecDeque<char> = text.chars().collect();
            let mut source_index = dfa.initial_index();
            while let Some(character) = characters.pop_front() {
                if let Some(transition_index) = dfa.transitions_contains_outgoing((source_index, &character.into())) {
                    let (_, _, target_index) = dfa.transitions_index(transition_index);
                    source_index = target_index;
                } else {
                    if dfa.is_final(source_index) {
                        let mut token = &None;
                        for token_state in dfa.states_index(source_index) {
                            if token.is_none() {
                                token = token_state.token();
                            } else {
                                if token_state.token().is_some() && token_state.token() != token {
                                    return Err(Error::from(ErrorKind::InconsistentTokensInFinalState));
                                }
                            }
                        }
                        if let Some(token) = token {
                            tokens.push(token.clone());
                        }
                        source_index = dfa.initial_index();
                        characters.push_front(character);
                    } else {
                        return Err(Error::from(ErrorKind::FailedToReachFinalState));
                    }
                }
            }
            if dfa.is_final(source_index) {
                let mut token = &None;
                for token_state in dfa.states_index(source_index) {
                    if token.is_none() {
                        token = token_state.token();
                    } else {
                        if token_state.token().is_some() && token_state.token() != token {
                            return Err(Error::from(ErrorKind::InconsistentTokensInFinalState));
                        }
                    }
                }
                if let Some(token) = token {
                    tokens.push(token.clone());
                }
            } else {
                return Err(Error::from(ErrorKind::FailedToReachFinalState));
            }
            Ok(tokens)
        } else {
            Err(Error::from(ErrorKind::NotCompiled))
        }
    }
}

#[cfg(test)]
mod tests {
    use interval_map::Interval;
    use regular_expression::{
        sym,
        rep,
        con,
        // alt,
    };
    use crate::{
        error::Result,
        Lexer,
    };

    #[test]
    fn test_1() -> Result<()> {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        enum Token {
            A,
            B,
        };
        let mut lexer = Lexer::from(map![
            sym![Interval::singleton(A)] => Some(Token::A),
            sym![Interval::singleton(B)] => Some(Token::B),
            rep!(sym![Interval::singleton(SPACE)]) => None
        ]);
        lexer.compile();
        assert_eq!(vec![Token::A, Token::B, Token::A], lexer.lex("A B  A   ")?);
        Ok(())
    }

    #[test]
    fn test_2() -> Result<()> {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        #[allow(non_camel_case_types)]
        enum Token {
            A_REP,
            B_REP
        };
        let mut lexer = Lexer::from(map![
            rep!(sym![Interval::singleton(A)]) => Some(Token::A_REP),
            rep!(sym![Interval::singleton(B)]) => Some(Token::B_REP),
            rep!(sym![Interval::singleton(SPACE)]) => None
        ]);
        lexer.compile();
        assert_eq!(vec![Token::A_REP, Token::B_REP, Token::B_REP], lexer.lex("AAAAAAABBBB   BBBB")?);
        Ok(())
    }

    #[test]
    fn test_3() -> Result<()> {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        #[allow(non_camel_case_types)]
        enum Token {
            A,
            AB,
            BB,
            B,
        };
        let mut lexer = Lexer::from(map![
            sym![Interval::singleton(A)] => Some(Token::A),
            con![sym![Interval::singleton(A)], sym![Interval::singleton(B)]] => Some(Token::AB),
            con![sym![Interval::singleton(B)], sym![Interval::singleton(B)]] => Some(Token::BB),
            sym![Interval::singleton(B)] => Some(Token::B)
        ]);
        lexer.compile();
        assert_eq!(vec![Token::AB, Token::B], lexer.lex("ABB")?);
        Ok(())
    }

    static A: u32 = 65;
    static B: u32 = 66;
    static SPACE: u32 = 32;
}
