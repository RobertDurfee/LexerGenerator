use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::util::IDGenerator;
use crate::re::RE;

#[derive(Debug, PartialEq)]
pub(crate) struct NFA<S, E> 
where
    S: Eq + Hash,
    E: Eq + Hash
{ 
    initial: S,
    transitions: HashMap<S, HashMap<E, HashSet<S>>>,
    finals: HashSet<S>,
}

impl<S, E> NFA<S, E>
where
    S: Copy + Eq + Hash,
    E: Copy + Eq + Hash
{
    fn new(initial: S) -> Self {
        Self {
            initial,
            transitions: map![initial => HashMap::new()],
            finals: HashSet::new(),
        }
    }

    fn insert(&mut self, state: S) {
        if !self.transitions.contains_key(&state) {
            self.transitions.insert(state, HashMap::new());
        }
    }

    fn insert_transition(&mut self, start: S, event: E, end: S) {
        if !self.transitions.contains_key(&start) {
            self.transitions.insert(start, HashMap::new());
        }
        if !self.transitions.contains_key(&end) {
            self.transitions.insert(end, HashMap::new());
        }
        let start_transitions = self.transitions.get_mut(&start).unwrap();
        if !start_transitions.contains_key(&event) {
            start_transitions.insert(event, HashSet::new());
        }
        start_transitions.get_mut(&event).unwrap().insert(end);
    }

    fn insert_final(&mut self, state: S) {
        if !self.transitions.contains_key(&state) {
            self.transitions.insert(state, HashMap::new());
        }
        self.finals.insert(state);
    }
}

impl<S, E> Extend<(S, HashMap<E, HashSet<S>>)> for NFA<S, E>
where 
    S: Copy + Eq + Hash,
    E: Copy + Eq + Hash
{
    fn extend<T: IntoIterator<Item = (S, HashMap<E, HashSet<S>>)>>(&mut self, iter: T) {
        for (start, transitions) in iter.into_iter() {
            for (event, ends) in transitions.into_iter() {
                for end in ends.into_iter() {
                    self.insert_transition(start, event, end);
                }
            }
        }
    }
}

type ENFA<S, E> = NFA<S, Option<E>>;

impl ENFA<u128, char> {
    fn _from(re: RE, ids: &mut IDGenerator) -> Self {
        match re {
            RE::Epsilon => {
                let eps_enfa_initial = ids.next();
                let mut eps_enfa = ENFA::new(eps_enfa_initial);
                let eps_enfa_final = ids.next();
                eps_enfa.insert_final(eps_enfa_final);
                eps_enfa.insert_transition(eps_enfa_initial, None, eps_enfa_final);
                eps_enfa
            },
            RE::Symbol { symbol } => {
                let sym_enfa_initial = ids.next();
                let mut sym_enfa = ENFA::new(sym_enfa_initial);
                let sym_enfa_final = ids.next();
                sym_enfa.insert_final(sym_enfa_final);
                sym_enfa.insert_transition(sym_enfa_initial, Some(symbol), sym_enfa_final);
                sym_enfa
            },
            RE::Alternation { res } => {
                let alt_enfa_initial = ids.next();
                let mut alt_enfa = ENFA::new(alt_enfa_initial);
                let alt_enfa_final= ids.next();
                alt_enfa.insert_final(alt_enfa_final);
                for re in res {
                    let re_enfa = ENFA::_from(re, ids);
                    alt_enfa.extend(re_enfa.transitions);
                    alt_enfa.insert_transition(alt_enfa_initial, None, re_enfa.initial);
                    for re_enfa_final in re_enfa.finals {
                        alt_enfa.insert_transition(re_enfa_final, None, alt_enfa_final);
                    }
                }
                alt_enfa
            },
            RE::Concatenation { res } => {
                let cat_enfa_initial = ids.next();
                let mut cat_enfa = ENFA::new(cat_enfa_initial);
                let cat_enfa_final = ids.next();
                cat_enfa.insert_final(cat_enfa_final);
                let mut prev_re_enfa_finals = set![cat_enfa_initial];
                for re in res {
                    let re_enfa = ENFA::_from(re, ids);
                    cat_enfa.extend(re_enfa.transitions);
                    for prev_re_enfa_final in prev_re_enfa_finals {
                        cat_enfa.insert_transition(prev_re_enfa_final, None, re_enfa.initial);
                    }
                    prev_re_enfa_finals = re_enfa.finals;
                }
                for prev_re_enfa_final in prev_re_enfa_finals {
                    cat_enfa.insert_transition(prev_re_enfa_final, None, cat_enfa_final);
                }
                cat_enfa
            },
            RE::Repetition { re } => {
                let rep_enfa_initial = ids.next();
                let mut rep_enfa = ENFA::new(rep_enfa_initial);
                let rep_enfa_final = ids.next();
                rep_enfa.insert_final(rep_enfa_final);
                let re_enfa = ENFA::_from(*re, ids);
                rep_enfa.extend(re_enfa.transitions);
                rep_enfa.insert_transition(rep_enfa_initial, None, re_enfa.initial);
                for re_enfa_final in re_enfa.finals {
                    rep_enfa.insert_transition(re_enfa_final, None, rep_enfa_final);
                    rep_enfa.insert_transition(re_enfa_final, None, re_enfa.initial);
                }
                rep_enfa.insert_transition(rep_enfa_initial, None, rep_enfa_final);
                rep_enfa
            },
        }
    }
}

impl From<RE> for ENFA<u128, char> {
    fn from(re: RE) -> Self {
        ENFA::_from(re, &mut IDGenerator::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::nfa::ENFA;
    use crate::re::RE;

    #[test]
    fn test_1() {
        let expected = ENFA {
            initial: 0u128,
            transitions: map![
                0u128 => map![
                    None => set![1u128]
                ],
                1u128 => map![]
            ],
            finals: set![1u128],
        };
        let actual = ENFA::from(RE::Epsilon);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_2() {
        let expected = ENFA {
            initial: 0u128,
            transitions: map![
                0u128 => map![
                    Some('A') => set![1u128]
                ],
                1u128 => map![]
            ],
            finals: set![1u128],
        };
        let actual = ENFA::from(RE::Symbol { symbol: 'A' });
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_3() {
        let expected = ENFA {
            initial: 0u128,
            transitions: map![
                0u128 => map![
                    None => set![2u128, 4u128]
                ],
                1u128 => map![],
                2u128 => map![
                    None => set![3u128]
                ],
                3u128 => map![
                    None => set![1u128]
                ],
                4u128 => map![
                    Some('A') => set![5u128]
                ],
                5u128 => map![
                    None => set![1u128]
                ]
            ],
            finals: set![1u128],
        };
        let actual = ENFA::from(RE::Alternation {
            res: vec![
                RE::Epsilon,
                RE::Symbol { symbol: 'A' },
            ],
        });
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_4() {
        let expected = ENFA {
            initial: 0u128,
            transitions: map![
                0u128 => map![
                    None => set![2u128]
                ],
                1u128 => map![],
                2u128 => map![
                    Some('A') => set![3u128]
                ],
                3u128 => map![
                    None => set![4u128]
                ],
                4u128 => map![
                    None => set![5u128]
                ],
                5u128 => map![
                    None => set![1u128]
                ]
            ],
            finals: set![1u128],
        };
        let actual = ENFA::from(RE::Concatenation {
            res: vec![
                RE::Symbol { symbol: 'A' },
                RE::Epsilon,
            ],
        });
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_5() {
        let expected = ENFA {
            initial: 0u128,
            transitions: map![
                0u128 => map![
                    None => set![1u128, 2u128]
                ],
                1u128 => map![],
                2u128 => map![
                    Some('A') => set![3u128]
                ],
                3u128 => map![
                    None => set![2u128, 1u128]
                ]
            ],
            finals: set![1u128],
        };
        let actual = ENFA::from(RE::Repetition {
            re: Box::new(RE::Symbol { symbol: 'A' }),
        });
        assert_eq!(expected, actual);
    }
}
