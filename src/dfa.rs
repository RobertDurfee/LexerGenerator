use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use crate::nfa::{NFA, ENFA};

#[derive(Debug, PartialEq)]
pub(crate) struct DFA<S, E> {
    st_to_ix: BTreeMap<Rc<S>, usize>,
    ix_to_st: Vec<Rc<S>>,
    initial: usize,
    transitions: BTreeMap<usize, BTreeMap<E, usize>>,
    finals: BTreeSet<usize>,
}

impl<S, E> DFA<S, E>
where
    S: Ord,
    E: Ord,
{
    fn new(initial: S) -> Self {
        let initial_rc = Rc::new(initial);
        Self {
            st_to_ix: map![initial_rc.clone() => 0],
            ix_to_st: vec![initial_rc],
            initial: 0,
            transitions: map![0 => BTreeMap::new()],
            finals: BTreeSet::new(),
        }
    }

    fn insert_internal(&mut self, state: S) -> usize {
        if let Some(&ix) = self.st_to_ix.get(&state) {
            ix
        } else {
            let ix = self.st_to_ix.len();
            let state_rc = Rc::new(state);
            self.st_to_ix.insert(state_rc.clone(), ix);
            self.ix_to_st.push(state_rc);
            self.transitions.insert(ix, BTreeMap::new());
            ix
        }
    }
    
    pub(crate) fn insert(&mut self, state: S) {
        self.insert_internal(state);
    }

    pub(crate) fn insert_transition(&mut self, start: S, event: E, end: S) {
        let start_ix = self.insert_internal(start);
        let end_ix = self.insert_internal(end);
        self.transitions.get_mut(&start_ix).unwrap().insert(event, end_ix);
    }

    pub(crate) fn insert_final(&mut self, state: S) {
        let state_ix = self.insert_internal(state);
        self.finals.insert(state_ix);
    }

    pub(crate) fn initial(&self) -> &S {
        &*self.ix_to_st[self.initial]
    }

    pub(crate) fn transitions(&self) -> Transitions<'_, S, E> {
        Transitions::new(&self.ix_to_st, self.transitions.iter())
    }

    pub(crate) fn finals(&self) -> Finals<'_, S> {
        Finals::new(&self.ix_to_st, self.finals.iter())
    }

    pub(crate) fn get(&self, state: &S) -> Option<Get<'_, S, E>> {
        if let Some(ix) = self.st_to_ix.get(state) {
            Some(Get::new(&self.ix_to_st, self.transitions.get(ix).unwrap().iter()))
        } else {
            None
        }
    }

    pub(crate) fn is_final(&self, state: &S) -> bool {
        if let Some(ix) = self.st_to_ix.get(state) {
            self.finals.contains(&ix)
        } else {
            false
        }
    }

    pub(crate) fn contains(&self, state: &S) -> bool {
        self.st_to_ix.contains_key(state)
    }
}

pub(crate) struct Transitions<'a, S, E> {
    iter: Box<(dyn Iterator<Item = (&'a S, &'a E, &'a S)> + 'a)>,
}

impl<'a, S, E> Transitions<'a, S, E> {
    fn new<I: Iterator<Item = (&'a usize, &'a BTreeMap<E, usize>)> + 'a>(ix_to_st: &'a Vec<Rc<S>>, iter: I) -> Self {
        Self {
            iter: Box::new(iter.flat_map(move |(start_ix, transitions)| transitions.iter().map(move |(event, end_ix)| (&*ix_to_st[*start_ix], event, &*ix_to_st[*end_ix])))),
        }
    }
}

impl<'a, S, E> Iterator for Transitions<'a, S, E> {
    type Item = (&'a S, &'a E, &'a S);

    fn next(&mut self) -> Option<(&'a S, &'a E, &'a S)> {
        self.iter.next()
    }
}

pub(crate) struct Finals<'a, S> {
    iter: Box<(dyn Iterator<Item = &'a S> + 'a)>,
}

impl<'a, S> Finals<'a, S> {
    fn new<I: Iterator<Item = &'a usize> + 'a>(ix_to_st: &'a Vec<Rc<S>>, iter: I) -> Self {
        Self {
            iter: Box::new(iter.map(move |ix| &*ix_to_st[*ix])),
        }
    }
}

impl<'a, S> Iterator for Finals<'a, S> {
    type Item = &'a S;

    fn next(&mut self) -> Option<&'a S> {
        self.iter.next()
    }
}

pub(crate) struct Get<'a, S, E> {
    iter: Box<(dyn Iterator<Item = (&'a E, &'a S)> + 'a)>,
}

impl<'a, S, E> Get<'a, S, E> {
    fn new<I: Iterator<Item = (&'a E, &'a usize)> + 'a>(ix_to_st: &'a Vec<Rc<S>>, iter: I) -> Self {
        Self {
            iter: Box::new(iter.map(move |(event, end_ix)| (event, &*ix_to_st[*end_ix]))),
        }
    }
}

impl<'a, S, E> Iterator for Get<'a, S, E> {
    type Item = (&'a E, &'a S);

    fn next(&mut self) -> Option<(&'a E, &'a S)> {
        self.iter.next()
    }
}

impl<S, E> From<ENFA<S, E>> for DFA<BTreeSet<S>, E> {
    fn from(_enfa: ENFA<S, E>) -> DFA<BTreeSet<S>, E> {
        panic!();
    }
}

impl<S, E> From<NFA<S, E>> for DFA<BTreeSet<S>, E> 
where
    S: Copy + Ord,
    E: Copy + Ord,
{
    fn from(nfa: NFA<S, E>) -> DFA<BTreeSet<S>, E> {
        let mut dfa = DFA::new(set![*nfa.initial()]);
        let mut stack = vec![set![*nfa.initial()]];
        while let Some(start_set) = stack.pop() {
            for start in &start_set {
                for (&event, end_set) in nfa.get_grouped(&start).unwrap() {
                    let end_set = end_set.copied().collect();
                    if !dfa.contains(&end_set) {
                        stack.push(end_set.clone());
                    }
                    dfa.insert_transition(start_set.clone(), event, end_set.clone());
                }
                if nfa.is_final(&start) {
                    dfa.insert_final(start_set.clone());
                }
            }
        }
        dfa
    }
}

//     fn insert(&mut self, state: S) {
//         if !self.transitions.contains_key(&state) {
//             self.transitions.insert(state, BTreeMap::new());
//         }
//     }
// 
//     fn insert_transition(&mut self, start: S, event: E, end: S) {
//         if !self.transitions.contains_key(&start) {
//             self.transitions.insert(start, BTreeMap::new());
//         }
//         if !self.transitions.contains_key(&end) {
//             self.transitions.insert(end, BTreeMap::new());
//         }
//         self.transitions.get_mut(&start).unwrap().insert(event, end);
//     }
// 
//     fn insert_final(&mut self, state: S) {
//         if !self.transitions.contains_key(&state) {
//             self.transitions.insert(state, BTreeMap::new());
//         }
//         self.finals.insert(state);
//     }
// }
// 
// impl<S, E> Extend<(S, BTreeMap<E, S>)> for DFA<S, E>
// where 
//     S: Copy + Ord,
//     E: Copy + Ord
// {
//     fn extend<T: IntoIterator<Item = (S, BTreeMap<E, S>)>>(&mut self, iter: T) {
//         for (start, transitions) in iter.into_iter() {
//             for (event, end) in transitions.into_iter() {
//                 self.insert_transition(start, event, end);
//             }
//         }
//     }
// }
// 
// impl<S, E> From<NFA<S, E>> for DFA<BTreeSet<S>, E> 
// where
//     S: Clone + Ord,
//     E: Clone + Ord
// {
//     fn from(nfa: NFA<S, E>) -> DFA<BTreeSet<S>, E> {
//         let mut start_sets = vec![set![nfa.initial().clone()]];
//         let mut transitions = BTreeMap::new();
//         let mut finals = BTreeSet::new();
//         while let Some(start_set) = start_sets.pop() {
//             transitions.insert(start_set.clone(), BTreeMap::new());
//             for start in &start_set {
//                 for (event, end_set) in nfa.get(&start).unwrap().clone() {
//                     if !transitions.contains_key(&end_set) {
//                         start_sets.push(end_set.clone());
//                     }
//                     transitions.get_mut(&start_set).unwrap().insert(event, end_set);
//                 }
//                 if nfa.is_final(&start) {
//                     finals.insert(start_set.clone());
//                 }
//             }
//         }
//         DFA {
//             initial: set![nfa.initial().clone()],
//             transitions,
//             finals
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::nfa::ENFA;
    use crate::dfa::DFA;
    use crate::re::RE;

    struct Expected {
        initial: BTreeSet<usize>,
        transitions: BTreeSet<(BTreeSet<usize>, Option<char>, BTreeSet<usize>)>,
        finals: BTreeSet<BTreeSet<usize>>,
    }

    type Actual = DFA<BTreeSet<usize>, Option<char>>;

    fn assert_eq(expected: Expected, actual: Actual) {
        assert_eq!(expected.initial, actual.initial());
        assert_eq!(expected.transitions, actual.transitions().collect());
        assert_eq!(expected.finals, actual.finals().collect());
    }

    #[test]
    fn test_1() {
        let expected = Expected {
            initial: set![0],
            transitions: set![
                (set![0], None, set![1])
            ],
            finals: set![set![1]],
        };
        let actual = DFA::from(ENFA::from(RE::Epsilon));
        assert_eq(expected, actual);
    }

//     #[test]
//     fn test_2() {
//         let expected = DFA {
//             initial: set![0u128],
//             transitions: map![
//                 set![0u128] => map![
//                     Some('A') => set![1u128]
//                 ],
//                 set![1u128] => map![]
//             ],
//             finals: set![set![1u128]],
//         };
//         let actual = DFA::from(ENFA::from(RE::Symbol { symbol: 'A' }));
//         assert_eq!(expected, actual);
//     }
// 
//     #[test]
//     fn test_3() {
//         let expected = DFA {
//             initial: set![0u128],
//             transitions: map![
//                 set![0u128] => map![
//                     None => set![2u128, 4u128]
//                 ],
//                 set![2u128, 4u128] => map![
//                     None => set![3u128],
//                     Some('A') => set![5u128]
//                 ],
//                 set![3u128] => map![
//                     None => set![1u128]
//                 ],
//                 set![5u128] => map![
//                     None => set![1u128]
//                 ],
//                 set![1u128] => map![]
//             ],
//             finals: set![set![1u128]],
//         };
//         let actual = DFA::from(ENFA::from(RE::Alternation {
//             res: vec![
//                 RE::Epsilon,
//                 RE::Symbol { symbol: 'A' },
//             ],
//         }));
//         assert_eq!(expected, actual);
//     }
// 
//     #[test]
//     fn test_4() {
//         let expected = DFA {
//             initial: set![0u128],
//             transitions: map![
//                 set![0u128] => map![
//                     None => set![2u128]
//                 ],
//                 set![2u128] => map![
//                     Some('A') => set![3u128]
//                 ],
//                 set![3u128] => map![
//                     None => set![4u128]
//                 ],
//                 set![4u128] => map![
//                     None => set![5u128]
//                 ],
//                 set![5u128] => map![
//                     None => set![1u128]
//                 ],
//                 set![1u128] => map![]
//             ],
//             finals: set![set![1u128]],
//         };
//         let actual = DFA::from(ENFA::from(RE::Concatenation {
//             res: vec![
//                 RE::Symbol { symbol: 'A' },
//                 RE::Epsilon,
//             ],
//         }));
//         assert_eq!(expected, actual);
//     }
// 
//     #[test]
//     fn test_5() {
//         let expected = DFA {
//             initial: set![0u128],
//             transitions: map![
//                 set![0u128] => map![
//                     None => set![2u128, 1u128]
//                 ],
//                 set![2u128, 1u128] => map![
//                     Some('A') => set![3u128]
//                 ],
//                 set![3u128] => map![
//                     None => set![2u128, 1u128]
//                 ]
//             ],
//             finals: set![set![2u128, 1u128]],
//         };
//         let actual = DFA::from(ENFA::from(RE::Repetition {
//             re: Box::new(RE::Symbol { symbol: 'A' }),
//         }));
//         assert_eq!(expected, actual);
//     }
// 
}
