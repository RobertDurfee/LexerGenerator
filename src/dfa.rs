use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use crate::nfa::ENFA;
use crate::nfa;

type Ix = usize;

pub(crate) struct DFA<St, Ev> { 
    st_ix: BTreeMap<Rc<St>, Ix>,
    ix_st: BTreeMap<Ix, Rc<St>>,
    transitions: BTreeMap<Ix, BTreeMap<Ev, Ix>>,
    finals: BTreeSet<Ix>,
}

const INIT_IX: Ix = 0;

impl<St, Ev> DFA<St, Ev>
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
        self.transitions.get_mut(&start_ix).unwrap().insert(ev, end_ix);
    }

    pub(crate) fn set_final(&mut self, ix: Ix) {
        self.finals.insert(ix);
    }
 
    pub(crate) fn get_initial(&self) -> &St {
        self._st(INIT_IX)
    }

    pub(crate) fn get_transitions(&self) -> impl Iterator<Item = (&St, &Ev, &St)> {
        self.transitions.iter().flat_map(move |(&start_ix, transitions)| transitions.iter().map(move |(ev, &end_ix)| (self._st(start_ix), ev, self._st(end_ix))))
    }

    pub(crate) fn get_finals(&self) -> impl Iterator<Item = &St> {
        self.finals.iter().map(move |&final_ix| self._st(final_ix))
    }

    pub(crate) fn get_outgoing(&self, ix: Ix) -> Option<impl Iterator<Item = (&Ev, &St)>> {
        self.transitions.get(&ix).map(|transitions| transitions.iter().map(move |(ev, &end_ix)| (ev, self._st(end_ix))))
    }

    pub(crate) fn is_final(&self, ix: Ix) -> bool {
        self.finals.contains(&ix)
    }
}

impl<St, Ev> From<ENFA<St, Ev>> for DFA<BTreeSet<St>, Ev>
where
    St: Clone + Ord,
    Ev: Clone + Ord,
{
    fn from(enfa: ENFA<St, Ev>) -> DFA<BTreeSet<St>, Ev> {
        let initial_closure: BTreeSet<St> = enfa.get_closure(nfa::INIT_IX).unwrap().cloned().collect();
        let mut stack: Vec<BTreeSet<St>> = vec![initial_closure.clone()];
        let mut dfa: DFA<BTreeSet<St>, Ev> = DFA::new(initial_closure);
        while let Some(start_set_st) = stack.pop() {
            for start_st in &start_set_st {
                let start_set_ix = dfa.insert_state(start_set_st.clone());
                if enfa.is_final(enfa.ix(start_st).unwrap()) {
                    dfa.set_final(start_set_ix);
                }
                for (ev, end_set_st) in enfa.get_outgoing_grouped(enfa.ix(start_st).unwrap()).unwrap() {
                    if let Some(ev) = ev {
                        let mut closure_st: BTreeSet<St> = BTreeSet::new();
                        for end_st in end_set_st {
                            closure_st = closure_st.union(&enfa.get_closure(enfa.ix(end_st).unwrap()).unwrap().cloned().collect()).cloned().collect();
                        }
                        if let Some(closure_ix) = dfa.ix(&closure_st) {
                            dfa.insert_transition(start_set_ix, ev.clone(), closure_ix);
                        } else {
                            if dfa.ix(&closure_st).is_none() {
                                stack.push(closure_st.clone());
                            }
                            let closure_ix = dfa.insert_state(closure_st);
                            dfa.insert_transition(start_set_ix, ev.clone(), closure_ix);
                        }
                    }
                }
            }
        }
        dfa
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::dfa::DFA;
    use crate::nfa::ENFA;
    use crate::re::RE;

    struct Expected {
        initial: BTreeSet<u32>,
        transitions: BTreeSet<(BTreeSet<u32>, char, BTreeSet<u32>)>,
        finals: BTreeSet<BTreeSet<u32>>
    }

    type Actual = DFA<BTreeSet<u32>, char>;

    fn assert_eq(expected: Expected, actual: Actual) {
        assert_eq!(expected.initial, actual.get_initial().clone());
        assert_eq!(expected.transitions, actual.get_transitions().map(|(start_set_st, &ev, end_set_st)| (start_set_st.clone(), ev, end_set_st.clone())).collect());
        assert_eq!(expected.finals, actual.get_finals().map(|final_set_st| final_set_st.clone()).collect());
    }

    #[test]
    fn test_1() {
        let expected = Expected {
            initial: set![0, 1],
            transitions: set![],
            finals: set![set![0, 1]]
        };
        let actual = DFA::from(ENFA::from(RE::Epsilon));
        assert_eq(expected, actual);
    }

    #[test]
    fn test_2() {
        let expected = Expected {
            initial: set![0],
            transitions: set![
                (set![0], 'A', set![1])
            ],
            finals: set![set![1]]
        };
        let actual = DFA::from(ENFA::from(RE::Symbol { symbol: 'A' }));
        assert_eq(expected, actual);
    }

    #[test]
    fn test_3() {
        let expected = Expected {
            initial: set![0, 1, 2, 3, 4],
            transitions: set![
                (set![0, 1, 2, 3, 4], 'A', set![1, 5])
            ],
            finals: set![set![0, 1, 2, 3, 4], set![1, 5]]
        };
        let actual = DFA::from(ENFA::from(RE::Alternation {
            res: vec![
                RE::Epsilon,
                RE::Symbol { symbol: 'A' }
            ]
        }));
        assert_eq(expected, actual);
    }

    #[test]
    fn test_4() {
        let expected = Expected {
            initial: set![0, 2],
            transitions: set![
                (set![0, 2], 'A', set![1, 3, 4, 5])
            ],
            finals: set![set![1, 3, 4, 5]]
        };
        let actual = DFA::from(ENFA::from(RE::Concatenation {
            res: vec![
                RE::Symbol { symbol: 'A' },
                RE::Epsilon
            ]
        }));
        assert_eq(expected, actual);
    }

    #[test]
    fn test_5() {
        let expected = Expected {
            initial: set![0, 1, 2],
            transitions: set![
                (set![0, 1, 2], 'A', set![1, 2, 3]),
                (set![1, 2, 3], 'A', set![1, 2, 3])
            ],
            finals: set![set![0, 1, 2], set![1, 2, 3]]
        };
        let actual = DFA::from(ENFA::from(RE::Repetition {
            re: Box::new(RE::Symbol { symbol: 'A' }),
        }));
        assert_eq(expected, actual);
    }
}
