use std::collections::{
    BTreeMap as Map,
    BTreeSet as Set,
    VecDeque,
};
use std::fmt::Debug;

use uuid::Uuid;

use regular_expression::{
    RE,
    StateGenerator,
};
use finite_automata::{
    ENFA,
    DFA,
    Subsume,
    ContainsFrom,
    Insert,
    Contains,
    At,
};

#[macro_use]
pub mod util;
pub mod error;
pub mod lexer;

use error::{Error, ErrorKind, Result};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TokenState<'a, T> {
    uuid: u128,
    sequence_number: u128,
    token: &'a Option<T>,
}

impl<'a, T: Copy> TokenState<'a, T> {
    fn new(token: &'a Option<T>) -> TokenState<'a, T> {
        TokenState { uuid: Uuid::new_v4().as_u128(), sequence_number: 0, token }
    }

    fn visible(&self) -> TokenState<'a, T> {
        *self
    }

    fn hidden(&self) -> TokenState<'a, T> {
        TokenState { token: &None, ..*self }
    }

    fn increment(&mut self) {
        self.sequence_number += 1
    }

    fn token(&self) -> &Option<T> {
        self.token
    }
}

struct TokenStateGenerator<'a, T> {
    token_state: TokenState<'a, T>,
    final_enabled: bool,
}

impl<'a, T: Copy> TokenStateGenerator<'a, T> {
    pub fn new(token: &'a Option<T>) -> TokenStateGenerator<'a, T> {
        TokenStateGenerator { token_state: TokenState::new(token), final_enabled: true }
    }

    fn next_final_enabled(&mut self) -> TokenState<'a, T> {
        let next = self.token_state.visible();
        self.token_state.increment();
        next
    }

    fn next_final_disabled(&mut self) -> TokenState<'a, T> {
        let next = self.token_state.hidden();
        self.token_state.increment();
        next
    }
}

impl<'a, T: Copy> StateGenerator for TokenStateGenerator<'a, T> {
    type State = TokenState<'a, T>;

    fn next_initial(&mut self) -> TokenState<'a, T> {
        let next = self.token_state.hidden();
        self.token_state.increment();
        next
    }

    fn next_ephemeral(&mut self) -> TokenState<'a, T> {
        let next = self.token_state.hidden();
        self.token_state.increment();
        next
    }

    fn next_final(&mut self) -> TokenState<'a, T> {
        if self.final_enabled {
            self.next_final_enabled()
        } else {
            self.next_final_disabled()
        }
    }

    fn disable_final(&mut self) -> &mut TokenStateGenerator<'a, T> {
        self.final_enabled = false;
        self
    }

    fn enable_final(&mut self) -> &mut TokenStateGenerator<'a, T> {
        self.final_enabled = true;
        self
    }
}

pub fn lex<T: Copy + Debug + Ord>(input: &str, dfa: &DFA<Set<TokenState<'_, T>>, char>) -> Result<Vec<T>> {
    let mut tokens = Vec::new();
    let mut characters: VecDeque<char> = input.chars().collect();
    let mut source_index = dfa.initial_index();
    while let Some(character) = characters.pop_front() {
        if let Some(transition_index) = dfa.contains(&(source_index, &character)) {
            let (_, _, target_index) = dfa.at(transition_index);
            source_index = target_index;
        } else {
            if dfa.is_final(source_index) {
                let mut token = &None;
                for token_state in dfa.at(source_index) {
                    if token.is_none() {
                        token = token_state.token();
                    } else {
                        if token_state.token().is_some() && token_state.token() != token {
                            return Err(Error::new(ErrorKind::InconsistentTokensInFinalState, format!("{:?} != {:?}", token_state.token(), token)));
                        }
                    }
                }
                if let Some(token) = token {
                    tokens.push(token.clone());
                }
                source_index = dfa.initial_index();
                characters.push_front(character);
            } else {
                return Err(Error::new(ErrorKind::FailedToReachFinalState, format!("{:?}", dfa.at(source_index))));
            }
        }
    }
    if dfa.is_final(source_index) {
        let mut token = &None;
        for token_state in dfa.at(source_index) {
            if token.is_none() {
                token = token_state.token();
            } else {
                if token_state.token().is_some() && token_state.token() != token {
                    return Err(Error::new(ErrorKind::InconsistentTokensInFinalState, format!("{:?} != {:?}", token_state.token(), token)));
                }
            }
        }
        if let Some(token) = token {
            tokens.push(token.clone());
        }
    } else {
        return Err(Error::new(ErrorKind::FailedToReachFinalState, format!("{:?}", dfa.at(source_index))));
    }
    Ok(tokens)
}

pub fn compile<T: Copy + Ord>(productions: &Map<RE, Option<T>>) -> Result<DFA<Set<TokenState<'_, T>>, char>> {
    let mut res = Vec::new();
    for (re, token) in productions {
        res.push(re.into_enfa(&mut TokenStateGenerator::new(token)));
    }
    let mut alt = ENFA::new(TokenState::new(&None));
    for re in res {
        alt.subsume(&re);
        let re_initial_index = alt.contains_from(&re, re.initial_index()).expect("state does not exist");
        alt.insert((alt.initial_index(), None, re_initial_index));
        for re_final_index in re.final_indices() {
            let re_final_index = alt.contains_from(&re, re_final_index).expect("state does not exist");
            alt.set_final(re_final_index);
        }
    }
    let dfa: DFA<Set<TokenState<'_, T>>, char> = DFA::from(alt);
    Ok(dfa)
}

// TODO: this should compile from a lexer grammar instead
// pub fn compile<T: Copy + Ord>(_input: &str) -> Result<DFA<Set<TokenState<'_, T>>, char>> {
//     Err(Error::from(ErrorKind::NotImplemented))
// }

#[cfg(test)]
mod tests {
    use regular_expression::{sym, rep, cat};

    use crate::{Result, lex, compile};

    #[test]
    fn test_1() -> Result<()> {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        enum Token {
            ZERO,
            ONE,
        };
        let expected = vec![Token::ZERO, Token::ONE, Token::ZERO];
        let actual = lex("0 1  0   ", &compile(&map![
            sym!('0') => Some(Token::ZERO),
            sym!('1') => Some(Token::ONE),
            rep!(sym!(' ')) => None
        ])?);
        assert_eq!(expected, actual?);
        Ok(())
    }

    #[test]
    fn test_2() -> Result<()> {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        #[allow(non_camel_case_types)]
        enum Token {
            ZERO_REP,
            ONE_REP
        };
        let expected = vec![Token::ZERO_REP, Token::ONE_REP, Token::ONE_REP];
        let actual = lex("00000001111   1111", &compile(&map![
            rep!(sym!('0')) => Some(Token::ZERO_REP),
            rep!(sym!('1')) => Some(Token::ONE_REP),
            rep!(sym!(' ')) => None
        ])?);
        assert_eq!(expected, actual?);
        Ok(())
    }

    #[test]
    fn test_3() -> Result<()> {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        #[allow(non_camel_case_types)]
        enum Token {
            ZERO,
            ZERO_ONE,
            ONE_ONE,
            ONE,
        };
        let expected = vec![Token::ZERO_ONE, Token::ONE];
        let actual = lex("011", &compile(&map![
            sym!('0') => Some(Token::ZERO),
            cat![sym!('0'), sym!('1')] => Some(Token::ZERO_ONE),
            cat![sym!('1'), sym!('1')] => Some(Token::ONE_ONE),
            sym!('1') => Some(Token::ONE)
        ])?);
        assert_eq!(expected, actual?);
        Ok(())
    }
}
