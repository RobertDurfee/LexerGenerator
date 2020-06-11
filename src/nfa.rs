use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::usize;

use crate::re::RE;

#[derive(Debug, PartialEq)]
pub(crate) struct NFA<S, E> { 
    st_to_ix: BTreeMap<Rc<S>, usize>,
    ix_to_st: Vec<Rc<S>>,
    initial: usize,
    transitions: BTreeMap<usize, BTreeMap<E, BTreeSet<usize>>>,
    finals: BTreeSet<usize>,
}

impl<S, E> NFA<S, E>
where
    S: Ord,
    E: Ord,
{
    pub(crate) fn new(initial: S) -> Self {
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
        let transitions = self.transitions.get_mut(&start_ix).unwrap();
        if let Some(ends) = transitions.get_mut(&event) {
            ends.insert(end_ix);
        } else {
            transitions.insert(event, set![end_ix]);
        }
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

    pub(crate) fn get_flat(&self, state: &S) -> Option<GetFlat<'_, S, E>> {
        if let Some(ix) = self.st_to_ix.get(state) {
            Some(GetFlat::new(&self.ix_to_st, self.transitions.get(ix).unwrap().iter()))
        } else {
            None
        }
    }

    pub(crate) fn get_grouped(&self, state: &S) -> Option<GetGrouped<'_, S, E>> {
        if let Some(ix) = self.st_to_ix.get(state) {
            Some(GetGrouped::new(&self.ix_to_st, self.transitions.get(ix).unwrap().iter()))
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
    fn new<I: Iterator<Item = (&'a usize, &'a BTreeMap<E, BTreeSet<usize>>)> + 'a>(ix_to_st: &'a Vec<Rc<S>>, iter: I) -> Self {
        Self {
            iter: Box::new(iter.flat_map(move |(start_ix, transitions)| transitions.iter().flat_map(move |(event, end_ixs)| end_ixs.iter().map(move |end_ix| (&*ix_to_st[*start_ix], event, &*ix_to_st[*end_ix]))))),
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

pub(crate) struct GetFlat<'a, S, E> {
    iter: Box<(dyn Iterator<Item = (&'a E, &'a S)> + 'a)>,
}

impl<'a, S, E> GetFlat<'a, S, E> {
    fn new<I: Iterator<Item = (&'a E, &'a BTreeSet<usize>)> + 'a>(ix_to_st: &'a Vec<Rc<S>>, iter: I) -> Self {
        Self {
            iter: Box::new(iter.flat_map(move |(event, end_ixs)| end_ixs.iter().map(move |end_ix| (event, &*ix_to_st[*end_ix])))),
        }
    }
}

impl<'a, S, E> Iterator for GetFlat<'a, S, E> {
    type Item = (&'a E, &'a S);

    fn next(&mut self) -> Option<(&'a E, &'a S)> {
        self.iter.next()
    }
}

pub(crate) struct GetGrouped<'a, S, E> {
    iter: Box<(dyn Iterator<Item = (&'a E, Group<'a, S>)> + 'a)>,
}

impl<'a, S, E> GetGrouped<'a, S, E> {
    fn new<I: Iterator<Item = (&'a E, &'a BTreeSet<usize>)> + 'a>(ix_to_st: &'a Vec<Rc<S>>, iter: I) -> Self {
        Self {
            iter: Box::new(iter.map(move |(event, end_ixs)| (event, Group::new(ix_to_st, end_ixs.iter())))),
        }
    }
}

impl<'a, S, E> Iterator for GetGrouped<'a, S, E> {
    type Item = (&'a E, Group<'a, S>);

    fn next(&mut self) -> Option<(&'a E, Group<'a, S>)> {
        self.iter.next()
    }
}

pub(crate) struct Group<'a, S> {
    iter: Box<(dyn Iterator<Item = &'a S> + 'a)>,
}

impl<'a, S> Group<'a, S> {
    fn new<I: Iterator<Item = &'a usize> + 'a>(ix_to_st: &'a Vec<Rc<S>>, iter: I) -> Self {
        Self {
            iter: Box::new(iter.map(move |ix| &*ix_to_st[*ix])),
        }
    }
}

impl<'a, S> Iterator for Group<'a, S> {
    type Item = &'a S;

    fn next(&mut self) -> Option<&'a S> {
        self.iter.next()
    }
}

pub(crate) type ENFA<S, E> = NFA<S, Option<E>>;

impl ENFA<usize, char> {
    fn _from<I: Iterator<Item = usize>>(re: RE, ids: &mut I) -> Self {
        match re {
            RE::Epsilon => {
                let eps_enfa_initial = ids.next().expect("no more ids");
                let mut eps_enfa = ENFA::new(eps_enfa_initial);
                let eps_enfa_final = ids.next().expect("no more ids");
                eps_enfa.insert_final(eps_enfa_final);
                eps_enfa.insert_transition(eps_enfa_initial, None, eps_enfa_final);
                eps_enfa
            },
            RE::Symbol { symbol } => {
                let sym_enfa_initial = ids.next().expect("no more ids");
                let mut sym_enfa = ENFA::new(sym_enfa_initial);
                let sym_enfa_final = ids.next().expect("no more ids");
                sym_enfa.insert_final(sym_enfa_final);
                sym_enfa.insert_transition(sym_enfa_initial, Some(symbol), sym_enfa_final);
                sym_enfa
            },
            RE::Alternation { res } => {
                let alt_enfa_initial = ids.next().expect("no more ids");
                let mut alt_enfa = ENFA::new(alt_enfa_initial);
                let alt_enfa_final= ids.next().expect("no more ids");
                alt_enfa.insert_final(alt_enfa_final);
                for re in res {
                    let re_enfa = ENFA::_from(re, ids);
                    for (start, transitions) in re_enfa.transitions {
                        for (event, ends) in transitions {
                            for end in ends {
                                alt_enfa.insert_transition(*re_enfa.ix_to_st[start], event, *re_enfa.ix_to_st[end]);
                            }
                        }
                    }
                    alt_enfa.insert_transition(alt_enfa_initial, None, *re_enfa.ix_to_st[re_enfa.initial]);
                    for re_enfa_final in re_enfa.finals {
                        alt_enfa.insert_transition(*re_enfa.ix_to_st[re_enfa_final], None, alt_enfa_final);
                    }
                }
                alt_enfa
            },
            RE::Concatenation { res } => {
                let cat_enfa_initial = ids.next().expect("no more ids");
                let mut cat_enfa = ENFA::new(cat_enfa_initial);
                let cat_enfa_final = ids.next().expect("no more ids");
                cat_enfa.insert_final(cat_enfa_final);
                let mut prev_re_enfa_finals = set![cat_enfa_initial];
                for re in res {
                    let re_enfa = ENFA::_from(re, ids);
                    let re_enfa_finals = re_enfa.finals.iter().map(|f| *re_enfa.ix_to_st[*f]).collect();
                    for (start, transitions) in re_enfa.transitions {
                        for (event, ends) in transitions {
                            for end in ends {
                                cat_enfa.insert_transition(*re_enfa.ix_to_st[start], event, *re_enfa.ix_to_st[end]);
                            }
                        }
                    }
                    for prev_re_enfa_final in prev_re_enfa_finals {
                        cat_enfa.insert_transition(prev_re_enfa_final, None, *re_enfa.ix_to_st[re_enfa.initial]);
                    }
                    prev_re_enfa_finals = re_enfa_finals;
                }
                for prev_re_enfa_final in prev_re_enfa_finals {
                    cat_enfa.insert_transition(prev_re_enfa_final, None, cat_enfa_final);
                }
                cat_enfa
            },
            RE::Repetition { re } => {
                let rep_enfa_initial = ids.next().expect("no more ids");
                let mut rep_enfa = ENFA::new(rep_enfa_initial);
                let rep_enfa_final = ids.next().expect("no more ids");
                rep_enfa.insert_final(rep_enfa_final);
                let re_enfa = ENFA::_from(*re, ids);
                for (start, transitions) in re_enfa.transitions {
                    for (event, ends) in transitions {
                        for end in ends {
                            rep_enfa.insert_transition(*re_enfa.ix_to_st[start], event, *re_enfa.ix_to_st[end]);
                        }
                    }
                }
                rep_enfa.insert_transition(rep_enfa_initial, None, *re_enfa.ix_to_st[re_enfa.initial]);
                for re_enfa_final in re_enfa.finals {
                    rep_enfa.insert_transition(*re_enfa.ix_to_st[re_enfa_final], None, rep_enfa_final);
                    rep_enfa.insert_transition(*re_enfa.ix_to_st[re_enfa_final], None, *re_enfa.ix_to_st[re_enfa.initial]);
                }
                rep_enfa.insert_transition(rep_enfa_initial, None, rep_enfa_final);
                rep_enfa
            },
        }
    }
}

impl From<RE> for ENFA<usize, char> {
    fn from(re: RE) -> Self {
        ENFA::_from(re, &mut (0..usize::MAX))
    }
}

// impl<S, E> ENFA<S, E>
// where
//     S: Clone + Ord,
//     E: Ord
// {
//     fn closure(&self, start: &S) -> BTreeSet<S> {
//         let mut starts = vec![start.clone()];
//         let mut closure = BTreeSet::new();
//         while let Some(start) = starts.pop() {
//             closure.insert(start.clone());
//             if let Some(transitions) = self.get(&start) {
//                 for (event, ends) in transitions {
//                     if event.is_none() {
//                         for end in ends {
//                             if !closure.contains(end) {
//                                 starts.push(end.clone());
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//         closure
//     }
// }
// 
// impl<S, E> From<ENFA<S, E>> for NFA<BTreeSet<S>, E> 
// where
//     S: Clone + Ord,
//     E: Clone + Ord
// {
//     fn from(enfa: ENFA<S, E>) -> NFA<BTreeSet<S>, E> {
//         let mut start_sets = vec![enfa.closure(enfa.initial())];
//         let mut transitions = BTreeMap::new();
//         let mut finals = BTreeSet::new();
//         while let Some(start_set) = start_sets.pop() {
//             transitions.insert(start_set.clone(), BTreeMap::new());
//             for start in &start_set {
//                 for (event, end_set) in enfa.get(&start).unwrap().clone() {
//                     if let Some(event) = event {
//                         transitions.get_mut(&start_set).unwrap().insert(event.clone(), BTreeSet::new());
//                         for end in end_set {
//                             let closure = enfa.closure(&end);
//                             if !transitions.contains_key(&closure) {
//                                 start_sets.push(closure.clone());
//                             }
//                             transitions.get_mut(&start_set).unwrap().get_mut(&event).unwrap().insert(closure);
//                         }
//                     }
//                 }
//                 if enfa.is_final(&start) {
//                     finals.insert(start_set.clone());
//                 }
//             }
//         }
//         NFA {
//             initial: enfa.closure(enfa.initial()),
//             transitions,
//             finals,
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::nfa::ENFA;
    use crate::re::RE;

    struct Expected<'a> {
        initial: &'a usize,
        transitions: BTreeSet<(&'a usize, &'a Option<char>, &'a usize)>,
        finals: BTreeSet<&'a usize>,
    }

    type Actual = ENFA<usize, char>;

    fn assert_eq(expected: Expected, actual: Actual) {
        assert_eq!(expected.initial, actual.initial());
        assert_eq!(expected.transitions, actual.transitions().collect());
        assert_eq!(expected.finals, actual.finals().collect());
    }

    #[test]
    fn test_1() {
        let expected = Expected {
            initial: &0,
            transitions: set![
                (&0, &None,      &1)
            ],
            finals: set![&1],
        };
        let actual = ENFA::from(RE::Epsilon);
        assert_eq(expected, actual);
    }

    #[test]
    fn test_2() {
        let expected = Expected {
            initial: &0,
            transitions: set![
                (&0, &Some('A'), &1)
            ],
            finals: set![&1],
        };
        let actual = ENFA::from(RE::Symbol { symbol: 'A' });
        assert_eq(expected, actual);
    }

    #[test]
    fn test_3() {
        let expected = Expected {
            initial: &0,
            transitions: set![
                (&0, &None,      &2),
                (&0, &None,      &4),
                (&2, &None,      &3),
                (&4, &Some('A'), &5),
                (&3, &None,      &1),
                (&5, &None,      &1)
            ],
            finals: set![&1],
        };
        let actual = ENFA::from(RE::Alternation {
            res: vec![
                RE::Epsilon,
                RE::Symbol { symbol: 'A' },
            ],
        });
        assert_eq(expected, actual);
    }

    #[test]
    fn test_4() {
        let expected = Expected {
            initial: &0,
            transitions: set![
                (&0, &None,      &2),
                (&2, &Some('A'), &3),
                (&3, &None,      &4),
                (&4, &None,      &5),
                (&5, &None,      &1)
            ],
            finals: set![&1],
        };
        let actual = ENFA::from(RE::Concatenation {
            res: vec![
                RE::Symbol { symbol: 'A' },
                RE::Epsilon,
            ],
        });
        assert_eq(expected, actual);
    }

    #[test]
    fn test_5() {
        let expected = Expected {
            initial: &0,
            transitions: set![
                (&0, &None,      &1),
                (&0, &None,      &2),
                (&2, &Some('A'), &3),
                (&3, &None,      &2),
                (&3, &None,      &1)
            ],
            finals: set![&1],
        };
        let actual = ENFA::from(RE::Repetition {
            re: Box::new(RE::Symbol { symbol: 'A' }),
        });
        assert_eq(expected, actual);
    }
}
