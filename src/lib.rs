use std::collections::{
    BTreeMap as Map,
    BTreeSet as Set,
};

use regular_expression::re::RE;
use finite_automata::{
    enfa::ENFA,
    dfa::DFA,
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

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
struct TokenIds<'a, T> {
    token: &'a Option<T>,
    id: u32,
}

impl<'a, T> TokenIds<'a, T> {
    pub fn new(token: &'a Option<T>) -> TokenIds<'a, T> {
        TokenIds { token, id: 0 }
    }
}

impl<'a, T> Iterator for TokenIds<'a, T> {
    type Item = (&'a Option<T>, u32);

    fn next(&mut self) -> Option<(&'a Option<T>, u32)> {
        let next = (self.token, self.id);
        self.id += 1;
        Some(next)
    }
}

pub fn lex<T: Clone + Ord>(input: &str, productions: &Map<RE, Option<T>>) -> Result<Vec<T>> {
    let mut res = Vec::new();
    for (re, token) in productions {
        res.push(re.into_enfa(&mut TokenIds::new(token)));
    }
    let mut alt = ENFA::new((&None, 0));
    for re in res {
        alt.subsume(&re);
        let re_initial_index = alt.contains_from(&re, re.initial_index()).expect("state does not exist");
        alt.insert((alt.initial_index(), None, re_initial_index));
        for re_final_index in re.final_indices() {
            let re_final_index = alt.contains_from(&re, re_final_index).expect("state does not exist");
            alt.set_final(re_final_index);
        }
    }
    let dfa: DFA<Set<(&Option<T>, u32)>, char> = DFA::from(alt);
    let mut tokens = Vec::new();
    let mut characters: Vec<char> = input.chars().collect();
    let mut source_index = dfa.initial_index();
    while let Some(character) = characters.pop() {
        if let Some(transition_index) = dfa.contains(&(source_index, &character)) {
            let (_, _, target_index) = dfa.at(transition_index);
            source_index = target_index;
        } else {
            if dfa.is_final(source_index) {
                let mut tokens_iter = dfa.at(source_index).iter().map(|(token, _)| token);
                let token = tokens_iter.next().unwrap();
                while let Some(current_token) = tokens_iter.next() {
                    if current_token != token {
                        return Err(Error::from(ErrorKind::InconsistentTokensInFinalState));
                    }
                }
                if let Some(token) = token {
                    tokens.push(token.clone());
                }
                source_index = dfa.initial_index();
                characters.push(character);
            } else {
                return Err(Error::from(ErrorKind::FailedToReachFinalState));
            }
        }
    }
    if dfa.is_final(source_index) {
        let mut tokens_iter = dfa.at(source_index).iter().map(|(token, _)| token);
        let token = tokens_iter.next().unwrap();
        while let Some(current_token) = tokens_iter.next() {
            if current_token != token {
                return Err(Error::from(ErrorKind::InconsistentTokensInFinalState));
            }
        }
        if let Some(token) = token {
            tokens.push(token.clone());
        }
    } else {
        return Err(Error::from(ErrorKind::FailedToReachFinalState));
    }
    Ok(tokens)
}

pub fn productions<T>(_input: &str) -> Result<Map<RE, Option<T>>> {
    Err(Error::from(ErrorKind::NotImplemented))
}

#[cfg(test)]
mod tests {
    use regular_expression::{sym, rep};

    use crate::{Result, lex};

    #[test]
    fn test_1() -> Result<()> {
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
        enum Binary {
            ZERO,
            ONE,
        };
        let expected = vec![Binary::ZERO, Binary::ONE, Binary::ZERO];
        let actual = lex("0 1  0   ", &map![
            sym!('0') => Some(Binary::ZERO),
            sym!('1') => Some(Binary::ONE),
            rep!(sym!(' ')) => None
        ]);
        assert_eq!(expected, actual?);
        Ok(())
    }
}
