use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::ops::Range;
use std::u32;

use crate::re::RE;

type Ix = usize;

pub(crate) struct NFA<St, Ev> { 
    st_ix: BTreeMap<Rc<St>, Ix>,
    ix_st: BTreeMap<Ix, Rc<St>>,
    transitions: BTreeMap<Ix, BTreeMap<Ev, BTreeSet<Ix>>>,
    finals: BTreeSet<Ix>,
}

pub(crate) const INIT_IX: Ix = 0;

impl<St, Ev> NFA<St, Ev>
where
    St: Ord,
    Ev: Ord,
{
    pub(crate) fn new(init_st: St) -> Self {
        let init_st_rc = Rc::new(init_st);
        Self {
            st_ix: map![init_st_rc.clone() => INIT_IX],
            ix_st: map![INIT_IX => init_st_rc],
            transitions: map![INIT_IX => BTreeMap::new()],
            finals: BTreeSet::new(),
        }
    }

    pub(crate) fn insert_state(&mut self, st: St) -> Ix {
        if let Some(ix) = self.ix(&st) {
            ix
        } else {
            let ix = self.st_ix.len();
            let st_rc = Rc::new(st);
            self.st_ix.insert(st_rc.clone(), ix);
            self.ix_st.insert(ix, st_rc);
            self.transitions.insert(ix, BTreeMap::new());
            ix
        }
    }

    pub(crate) fn ix(&self, st: &St) -> Option<Ix> {
        self.st_ix.get(st).map(|&ix| ix)
    }

    pub(crate) fn st(&self, ix: Ix) -> Option<&St> {
        self.ix_st.get(&ix).map(|st| &**st)
    }

    fn _ix(&self, st: &St) -> Ix {
        *self.st_ix.get(st).unwrap()
    }

    fn _st(&self, ix: Ix) -> &St {
        self.ix_st.get(&ix).unwrap()
    }

    pub(crate) fn insert_transition(&mut self, start_ix: Ix, ev: Ev, end_ix: Ix) {
        let start_transitions = self.transitions.get_mut(&start_ix).unwrap();
        if let Some(end_ixs) = start_transitions.get_mut(&ev) {
            end_ixs.insert(end_ix);
        } else {
            start_transitions.insert(ev, set![end_ix]);
        }
    }

    pub(crate) fn set_final(&mut self, ix: Ix) {
        self.finals.insert(ix);
    }
 
    pub(crate) fn get_initial(&self) -> &St {
        self._st(INIT_IX)
    }

    pub(crate) fn get_transitions(&self) -> impl Iterator<Item = (&St, &Ev, &St)> {
        self.transitions.iter().flat_map(move |(&start_ix, transitions)| transitions.iter().flat_map(move |(ev, end_ixs)| end_ixs.iter().map(move |&end_ix| (self._st(start_ix), ev, self._st(end_ix)))))
    }

    pub(crate) fn get_finals(&self) -> impl Iterator<Item = &St> {
        self.finals.iter().map(move |&final_ix| self._st(final_ix))
    }

    pub(crate) fn get_outgoing_flat(&self, ix: Ix) -> Option<impl Iterator<Item = (&Ev, &St)>> {
        self.transitions.get(&ix).map(|transitions| transitions.iter().flat_map(move |(ev, end_ixs)| end_ixs.iter().map(move |&end_ix| (ev, self._st(end_ix)))))
    }

    pub(crate) fn get_outgoing_grouped(&self, ix: Ix) -> Option<impl Iterator<Item = (&Ev, impl Iterator<Item = &St>)>> {
        self.transitions.get(&ix).map(|transitions| transitions.iter().map(move |(ev, end_ixs)| (ev, end_ixs.iter().map(move |&end_ix| self._st(end_ix)))))
    }

    pub(crate) fn is_final(&self, ix: Ix) -> bool {
        self.finals.contains(&ix)
    }
}

pub(crate) type ENFA<St, Ev> = NFA<St, Option<Ev>>;

impl<St, Ev> ENFA<St, Ev>
where
    St: Ord,
    Ev: Ord,
{
    pub(crate) fn get_closure(&self, ix: Ix) -> Option<impl Iterator<Item = &St>> {
        self.st(ix).map(|st| {
            let mut stack = vec![st];
            let mut closure = BTreeSet::new();
            while let Some(start_st) = stack.pop() {
                closure.insert(start_st);
                for (ev, end_st) in self.get_outgoing_flat(self._ix(start_st)).unwrap() {
                    if ev.is_none() && !closure.contains(&end_st) {
                        stack.push(end_st);
                    }
                }
            }
            closure.into_iter()
        })
    }
}

impl ENFA<u32, char> {
    fn _from_re(re: RE, ids: &mut Range<u32>) -> ENFA<u32, char> {
        match re {
            RE::Epsilon => {
                let mut eps_enfa = ENFA::new(ids.next().expect("no more ids"));
                let eps_enfa_final_ix = eps_enfa.insert_state(ids.next().expect("no more ids"));
                eps_enfa.set_final(eps_enfa_final_ix);
                eps_enfa.insert_transition(INIT_IX, None, eps_enfa_final_ix);
                eps_enfa
            },
            RE::Symbol { symbol } => {
                let mut sym_enfa = ENFA::new(ids.next().expect("no more ids"));
                let sym_enfa_final_ix = sym_enfa.insert_state(ids.next().expect("no more ids"));
                sym_enfa.set_final(sym_enfa_final_ix);
                sym_enfa.insert_transition(INIT_IX, Some(symbol), sym_enfa_final_ix);
                sym_enfa
            },
            RE::Alternation { res } => {
                let mut alt_enfa = ENFA::new(ids.next().expect("no more ids"));
                let alt_enfa_final_ix = alt_enfa.insert_state(ids.next().expect("no more ids"));
                alt_enfa.set_final(alt_enfa_final_ix);
                for re in res {
                    let re_enfa = ENFA::_from_re(re, ids);
                    for (start_st, ev, end_st) in re_enfa.get_transitions() {
                        match (alt_enfa.ix(start_st), alt_enfa.ix(end_st)) {
                            (Some(start_ix), Some(end_ix)) => {
                                alt_enfa.insert_transition(start_ix, ev.clone(), end_ix);
                            },
                            (Some(start_ix), None) => {
                                let end_ix = alt_enfa.insert_state(end_st.clone());
                                alt_enfa.insert_transition(start_ix, ev.clone(), end_ix);
                            },
                            (None, Some(end_ix)) => {
                                let start_ix = alt_enfa.insert_state(start_st.clone());
                                alt_enfa.insert_transition(start_ix, ev.clone(), end_ix);
                            },
                            (None, None) => {
                                let start_ix = alt_enfa.insert_state(start_st.clone());
                                let end_ix = alt_enfa.insert_state(end_st.clone());
                                alt_enfa.insert_transition(start_ix, ev.clone(), end_ix);
                            },
                        }
                    }
                    if let Some(re_enfa_initial_ix) = alt_enfa.ix(re_enfa.get_initial()) {
                        alt_enfa.insert_transition(INIT_IX, None, re_enfa_initial_ix);
                    } else {
                        let re_enfa_initial_ix = alt_enfa.insert_state(re_enfa.get_initial().clone());
                        alt_enfa.insert_transition(INIT_IX, None, re_enfa_initial_ix);
                    }
                    for re_enfa_final_st in re_enfa.get_finals() {
                        if let Some(re_enfa_final_ix) = alt_enfa.ix(re_enfa_final_st) {
                            alt_enfa.insert_transition(re_enfa_final_ix, None, alt_enfa_final_ix);
                        } else {
                            let re_enfa_final_ix = alt_enfa.insert_state(re_enfa_final_st.clone());
                            alt_enfa.insert_transition(re_enfa_final_ix, None, alt_enfa_final_ix);
                        }
                    }
                }
                alt_enfa
            },
            RE::Concatenation { res } => {
                let mut cat_enfa = ENFA::new(ids.next().expect("no more ids"));
                let cat_enfa_final_ix = cat_enfa.insert_state(ids.next().expect("no more ids"));
                cat_enfa.set_final(cat_enfa_final_ix);
                let mut prev_re_enfa_final_ixs = set![INIT_IX];
                for re in res {
                    let re_enfa = ENFA::_from_re(re, ids);
                    for (start_st, ev, end_st) in re_enfa.get_transitions() {
                        match (cat_enfa.ix(start_st), cat_enfa.ix(end_st)) {
                            (Some(start_ix), Some(end_ix)) => {
                                cat_enfa.insert_transition(start_ix, ev.clone(), end_ix);
                            },
                            (Some(start_ix), None) => {
                                let end_ix = cat_enfa.insert_state(end_st.clone());
                                cat_enfa.insert_transition(start_ix, ev.clone(), end_ix);
                            },
                            (None, Some(end_ix)) => {
                                let start_ix = cat_enfa.insert_state(start_st.clone());
                                cat_enfa.insert_transition(start_ix, ev.clone(), end_ix);
                            },
                            (None, None) => {
                                let start_ix = cat_enfa.insert_state(start_st.clone());
                                let end_ix = cat_enfa.insert_state(end_st.clone());
                                cat_enfa.insert_transition(start_ix, ev.clone(), end_ix);
                            },
                        }
                    }
                    for prev_re_enfa_final_ix in prev_re_enfa_final_ixs {
                        if let Some(re_enfa_initial_ix) = cat_enfa.ix(re_enfa.get_initial()) {
                            cat_enfa.insert_transition(prev_re_enfa_final_ix, None, re_enfa_initial_ix);
                        } else {
                            let re_enfa_initial_ix = cat_enfa.insert_state(re_enfa.get_initial().clone());
                            cat_enfa.insert_transition(prev_re_enfa_final_ix, None, re_enfa_initial_ix);
                        }
                    }
                    prev_re_enfa_final_ixs = re_enfa.get_finals().map(|re_enfa_final| {
                        if let Some(re_enfa_final_ix) = cat_enfa.ix(re_enfa_final) {
                            re_enfa_final_ix
                        } else {
                            cat_enfa.insert_state(re_enfa_final.clone())
                        }
                    }).collect();
                }
                for prev_re_enfa_final_ix in prev_re_enfa_final_ixs {
                    cat_enfa.insert_transition(prev_re_enfa_final_ix, None, cat_enfa_final_ix);
                }
                cat_enfa
            },
            RE::Repetition { re } => {
                let mut rep_enfa = ENFA::new(ids.next().expect("no more ids"));
                let rep_enfa_final_ix = rep_enfa.insert_state(ids.next().expect("no more ids"));
                rep_enfa.set_final(rep_enfa_final_ix);
                let re_enfa = ENFA::_from_re(*re, ids);
                for (start_st, ev, end_st) in re_enfa.get_transitions() {
                    match (rep_enfa.ix(start_st), rep_enfa.ix(end_st)) {
                        (Some(start_ix), Some(end_ix)) => {
                            rep_enfa.insert_transition(start_ix, ev.clone(), end_ix);
                        },
                        (Some(start_ix), None) => {
                            let end_ix = rep_enfa.insert_state(end_st.clone());
                            rep_enfa.insert_transition(start_ix, ev.clone(), end_ix);
                        },
                        (None, Some(end_ix)) => {
                            let start_ix = rep_enfa.insert_state(start_st.clone());
                            rep_enfa.insert_transition(start_ix, ev.clone(), end_ix);
                        },
                        (None, None) => {
                            let start_ix = rep_enfa.insert_state(start_st.clone());
                            let end_ix = rep_enfa.insert_state(end_st.clone());
                            rep_enfa.insert_transition(start_ix, ev.clone(), end_ix);
                        },
                    }
                }
                if let Some(re_enfa_initial_ix) = rep_enfa.ix(re_enfa.get_initial()) {
                    rep_enfa.insert_transition(INIT_IX, None, re_enfa_initial_ix);
                    for re_enfa_final in re_enfa.get_finals() {
                        if let Some(re_enfa_final_ix) = rep_enfa.ix(re_enfa_final) {
                            rep_enfa.insert_transition(re_enfa_final_ix, None, rep_enfa_final_ix);
                            rep_enfa.insert_transition(re_enfa_final_ix, None, re_enfa_initial_ix);
                        } else {
                            let re_enfa_final_ix = rep_enfa.insert_state(re_enfa_final.clone());
                            rep_enfa.insert_transition(re_enfa_final_ix, None, rep_enfa_final_ix);
                            rep_enfa.insert_transition(re_enfa_final_ix, None, re_enfa_initial_ix);
                        }
                    }
                } else {
                    let re_enfa_initial_ix = rep_enfa.insert_state(re_enfa.get_initial().clone());
                    rep_enfa.insert_transition(INIT_IX, None, re_enfa_initial_ix);
                    for re_enfa_final in re_enfa.get_finals() {
                        if let Some(re_enfa_final_ix) = rep_enfa.ix(re_enfa_final) {
                            rep_enfa.insert_transition(re_enfa_final_ix, None, rep_enfa_final_ix);
                            rep_enfa.insert_transition(re_enfa_final_ix, None, re_enfa_initial_ix);
                        } else {
                            let re_enfa_final_ix = rep_enfa.insert_state(re_enfa_final.clone());
                            rep_enfa.insert_transition(re_enfa_final_ix, None, rep_enfa_final_ix);
                            rep_enfa.insert_transition(re_enfa_final_ix, None, re_enfa_initial_ix);
                        }
                    }
                }
                rep_enfa.insert_transition(INIT_IX, None, rep_enfa_final_ix);
                rep_enfa
            },
        }
    }
}

impl From<RE> for ENFA<u32, char> {
    fn from(re: RE) -> ENFA<u32, char> {
        ENFA::_from_re(re, &mut (0..u32::MAX))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::nfa::ENFA;
    use crate::re::RE;

    struct Expected {
        initial: u32,
        transitions: BTreeSet<(u32, Option<char>, u32)>,
        finals: BTreeSet<u32>
    }

    type Actual = ENFA<u32, char>;

    fn assert_eq(expected: Expected, actual: Actual) {
        assert_eq!(expected.initial, *actual.get_initial());
        assert_eq!(expected.transitions, actual.get_transitions().map(|(&start_st, &ev, &end_st)| (start_st, ev, end_st)).collect());
        assert_eq!(expected.finals, actual.get_finals().map(|&final_st| final_st).collect());
    }

    #[test]
    fn test_1() {
        let expected = Expected {
            initial: 0,
            transitions: set![
                (0, None,      1)
            ],
            finals: set![1]
        };
        let actual = ENFA::from(RE::Epsilon);
        assert_eq(expected, actual);
    }

    #[test]
    fn test_2() {
        let expected = Expected {
            initial: 0,
            transitions: set![
                (0, Some('A'), 1)
            ],
            finals: set![1]
        };
        let actual = ENFA::from(RE::Symbol { symbol: 'A' });
        assert_eq(expected, actual);
    }

    #[test]
    fn test_3() {
        let expected = Expected {
            initial: 0,
            transitions: set![
                (0, None,      2),
                (0, None,      4),
                (2, None,      3),
                (4, Some('A'), 5),
                (3, None,      1),
                (5, None,      1)
            ],
            finals: set![1]
        };
        let actual = ENFA::from(RE::Alternation {
            res: vec![
                RE::Epsilon,
                RE::Symbol { symbol: 'A' }
            ]
        });
        assert_eq(expected, actual);
    }

    #[test]
    fn test_4() {
        let expected = Expected {
            initial: 0,
            transitions: set![
                (0, None,      2),
                (2, Some('A'), 3),
                (3, None,      4),
                (4, None,      5),
                (5, None,      1)
            ],
            finals: set![1]
        };
        let actual = ENFA::from(RE::Concatenation {
            res: vec![
                RE::Symbol { symbol: 'A' },
                RE::Epsilon
            ]
        });
        assert_eq(expected, actual);
    }

    #[test]
    fn test_5() {
        let expected = Expected {
            initial: 0,
            transitions: set![
                (0, None,      1),
                (0, None,      2),
                (2, Some('A'), 3),
                (3, None,      2),
                (3, None,      1)
            ],
            finals: set![1]
        };
        let actual = ENFA::from(RE::Repetition {
            re: Box::new(RE::Symbol { symbol: 'A' })
        });
        assert_eq(expected, actual);
    }
}
