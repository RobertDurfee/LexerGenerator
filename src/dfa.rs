use std::collections::{BTreeMap, BTreeSet};

use crate::nfa::NFA;

#[derive(Debug, PartialEq)]
pub(crate) struct DFA<S, E> {
    initial: S,
    transitions: BTreeMap<S, BTreeMap<E, S>>,
    finals: BTreeSet<S>,
}

impl<S, E> DFA<S, E>
where
    S: Copy + Ord,
    E: Copy + Ord
{
    fn new(initial: S) -> Self {
        Self {
            initial,
            transitions: map![initial => BTreeMap::new()],
            finals: BTreeSet::new(),
        }
    }

    fn insert(&mut self, state: S) {
        if !self.transitions.contains_key(&state) {
            self.transitions.insert(state, BTreeMap::new());
        }
    }

    fn insert_transition(&mut self, start: S, event: E, end: S) {
        if !self.transitions.contains_key(&start) {
            self.transitions.insert(start, BTreeMap::new());
        }
        if !self.transitions.contains_key(&end) {
            self.transitions.insert(end, BTreeMap::new());
        }
        self.transitions.get_mut(&start).unwrap().insert(event, end);
    }

    fn insert_final(&mut self, state: S) {
        if !self.transitions.contains_key(&state) {
            self.transitions.insert(state, BTreeMap::new());
        }
        self.finals.insert(state);
    }
}

impl<S, E> Extend<(S, BTreeMap<E, S>)> for DFA<S, E>
where 
    S: Copy + Ord,
    E: Copy + Ord
{
    fn extend<T: IntoIterator<Item = (S, BTreeMap<E, S>)>>(&mut self, iter: T) {
        for (start, transitions) in iter.into_iter() {
            for (event, end) in transitions.into_iter() {
                self.insert_transition(start, event, end);
            }
        }
    }
}

impl<S, E> From<NFA<S, E>> for DFA<BTreeSet<S>, E> 
where
    S: Clone + Ord,
    E: Clone + Ord
{
    fn from(nfa: NFA<S, E>) -> DFA<BTreeSet<S>, E> {
        let mut start_sets = vec![set![nfa.initial().clone()]];
        let mut transitions = BTreeMap::new();
        let mut finals = BTreeSet::new();
        while let Some(start_set) = start_sets.pop() {
            transitions.insert(start_set.clone(), BTreeMap::new());
            for start in &start_set {
                for (event, end_set) in nfa.get(&start).unwrap().clone() {
                    if !transitions.contains_key(&end_set) {
                        start_sets.push(end_set.clone());
                    }
                    transitions.get_mut(&start_set).unwrap().insert(event, end_set);
                }
                if nfa.is_final(&start) {
                    finals.insert(start_set.clone());
                }
            }
        }
        DFA {
            initial: set![nfa.initial().clone()],
            transitions,
            finals
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::nfa::ENFA;
    use crate::dfa::DFA;
    use crate::re::RE;

    #[test]
    fn test_1() {
        let expected = DFA {
            initial: set![0u128],
            transitions: map![
                set![0u128] => map![
                    None => set![1u128]
                ],
                set![1u128] => map![]
            ],
            finals: set![set![1u128]],
        };
        let actual = DFA::from(ENFA::from(RE::Epsilon));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_2() {
        let expected = DFA {
            initial: set![0u128],
            transitions: map![
                set![0u128] => map![
                    Some('A') => set![1u128]
                ],
                set![1u128] => map![]
            ],
            finals: set![set![1u128]],
        };
        let actual = DFA::from(ENFA::from(RE::Symbol { symbol: 'A' }));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_3() {
        let expected = DFA {
            initial: set![0u128],
            transitions: map![
                set![0u128] => map![
                    None => set![2u128, 4u128]
                ],
                set![2u128, 4u128] => map![
                    None => set![3u128],
                    Some('A') => set![5u128]
                ],
                set![3u128] => map![
                    None => set![1u128]
                ],
                set![5u128] => map![
                    None => set![1u128]
                ],
                set![1u128] => map![]
            ],
            finals: set![set![1u128]],
        };
        let actual = DFA::from(ENFA::from(RE::Alternation {
            res: vec![
                RE::Epsilon,
                RE::Symbol { symbol: 'A' },
            ],
        }));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_4() {
        let expected = DFA {
            initial: set![0u128],
            transitions: map![
                set![0u128] => map![
                    None => set![2u128]
                ],
                set![2u128] => map![
                    Some('A') => set![3u128]
                ],
                set![3u128] => map![
                    None => set![4u128]
                ],
                set![4u128] => map![
                    None => set![5u128]
                ],
                set![5u128] => map![
                    None => set![1u128]
                ],
                set![1u128] => map![]
            ],
            finals: set![set![1u128]],
        };
        let actual = DFA::from(ENFA::from(RE::Concatenation {
            res: vec![
                RE::Symbol { symbol: 'A' },
                RE::Epsilon,
            ],
        }));
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_5() {
        let expected = DFA {
            initial: set![0u128],
            transitions: map![
                set![0u128] => map![
                    None => set![2u128, 1u128]
                ],
                set![2u128, 1u128] => map![
                    Some('A') => set![3u128]
                ],
                set![3u128] => map![
                    None => set![2u128, 1u128]
                ]
            ],
            finals: set![set![2u128, 1u128]],
        };
        let actual = DFA::from(ENFA::from(RE::Repetition {
            re: Box::new(RE::Symbol { symbol: 'A' }),
        }));
        assert_eq!(expected, actual);
    }

}
