use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::Hash;
use std::ops::Range;
use std::u32;

use directed_graph::{DirectedGraph, Vertex, VertexIndex, Edge, EdgeIndex};

use crate::re::RE;

type StateIndex = VertexIndex;
type State<S> = Vertex<S>;
type TransitionIndex = EdgeIndex;
type Transition<T> = Edge<T>;

pub struct FA<S, T> {
    initial: StateIndex,
    graph: DirectedGraph<S, T>,
    finals: HashSet<StateIndex>
}

impl<S, T> FA<S, T>
where
    S: Eq + Hash,
    T: Eq + Hash,
{
    pub fn new(initial: State<S>) -> FA<S, T> {
        let mut graph = DirectedGraph::new();
        let initial = graph.add_vertex(initial);
        FA {
            initial,
            graph,
            finals: HashSet::new(),
        }
    }

    pub fn add_state(&mut self, state: State<S>) -> StateIndex {
        self.graph.add_vertex(state)
    }

    pub fn get_state(&self, state_index: StateIndex) -> &State<S> {
        self.graph.get_vertex(state_index)
    }

    pub fn contains_state(&self, state: &State<S>) -> Option<StateIndex> {
        self.graph.contains_vertex(state)
    }

    pub fn add_transition(&mut self, transition: Transition<T>) -> TransitionIndex {
        self.graph.add_edge(transition)
    }

    pub fn get_transition(&self, transition_index: TransitionIndex) -> &Transition<T> {
        self.graph.get_edge(transition_index)
    }

    pub fn contains_transition(&self, transition: &Transition<T>) -> Option<TransitionIndex> {
        self.graph.contains_edge(transition)
    }

    pub fn set_final(&mut self, state_index: StateIndex) {
        self.graph.get_vertex(state_index); // ensure state_index exists
        self.finals.insert(state_index);
    }

    pub fn get_initial(&self) -> StateIndex {
        self.initial
    }

    pub fn transitions<'a>(&'a self) -> Box<dyn Iterator<Item = TransitionIndex> + 'a> {
        Box::new(self.graph.edges())
    }

    pub fn finals<'a>(&'a self) -> Box<dyn Iterator<Item = StateIndex> + 'a> {
        Box::new(self.finals.iter().map(|&ix| ix))
    }

    pub fn get_transitions_from<'a>(&'a self, state_index: StateIndex) -> Box<dyn Iterator<Item = TransitionIndex> + 'a> {
        self.graph.get_edges_from(state_index)
    }

    pub fn is_final(&self, state_index: StateIndex) -> bool {
        self.graph.get_vertex(state_index); // ensure state_index exists
        self.finals.contains(&state_index)
    }
}

pub type ENFA<S, T> = FA<S, Option<T>>;

impl<S, T> ENFA<S, T>
where
    S: Clone + Eq + Hash + Ord,
    T: Clone + Eq + Hash + Ord,
{
    fn to_dfa(&self) -> FA<BTreeSet<S>, T> {
        let mut stack: Vec<BTreeSet<StateIndex>> = vec![self.get_closure(self.get_initial()).collect()];
        let mut fa = FA::new(State { data: self.get_closure(self.get_initial()).map(|state| self.get_state(state).data.clone()).collect() });
        while let Some(source_closure) = stack.pop() {
            let mut target_closures: HashMap<T, BTreeSet<StateIndex>> = HashMap::new();
            let mut is_final = false;
            for &source in &source_closure {
                if self.is_final(source) {
                    is_final = true;
                }
                for transition in self.get_transitions_from(source) {
                    let transition = self.get_transition(transition);
                    if let Some(transition_data) = transition.data.clone() {
                        if let Some(target_closure) = target_closures.get_mut(&transition_data) {
                            target_closure.extend(self.get_closure(transition.target));
                        } else {
                            target_closures.insert(transition_data, self.get_closure(transition.target).collect());
                        }
                    }
                }
            }
            let source_closure = match fa.contains_state(&State { data: source_closure.iter().map(|&state| self.get_state(state).data.clone()).collect() }) {
                Some(source_closure) => source_closure,
                None                 => fa.add_state(State { data: source_closure.iter().map(|&state| self.get_state(state).data.clone()).collect() }),
            };
            if is_final {
                fa.set_final(source_closure);
            }
            for (transition_data, target_closure) in target_closures.drain() {
                let target_closure = match fa.contains_state(&State { data: target_closure.iter().map(|&state| self.get_state(state).data.clone()).collect() }) {
                    Some(target_closure) => target_closure,
                    None => {
                        let ix = fa.add_state(State { data: target_closure.iter().map(|&state| self.get_state(state).data.clone()).collect() });
                        stack.push(target_closure);
                        ix
                    },
                };
                fa.add_transition(Transition { source: source_closure, data: transition_data, target: target_closure });
            }
        }
        fa
    }
}

impl<S, T> ENFA<S, T>
where
    S: Eq + Hash,
    T: Eq + Hash,
{
    pub fn get_closure(&self, state_index: StateIndex) -> Box<dyn Iterator<Item = StateIndex>> {
        self.graph.get_vertex(state_index); // ensure state_index exists
        let mut stack = vec![state_index];
        let mut closure = HashSet::new();
        while let Some(source_state_index) = stack.pop() {
            closure.insert(source_state_index);
            for transition_index in self.get_transitions_from(source_state_index) {
                let transition = self.get_transition(transition_index);
                if transition.data.is_none() && !closure.contains(&transition.target) {
                    stack.push(transition.target);
                }
            }
        }
        Box::new(closure.into_iter())
    }
}

impl ENFA<u32, char> {
    fn from_re(re: RE, ids: &mut Range<u32>) -> ENFA<u32, char> {
        fn next(ids: &mut Range<u32>) -> u32 {
            ids.next().expect("no more ids")
        }
        match re {
            RE::Epsilon => {
                let mut eps = ENFA::new(State { data: next(ids) });
                let eps_final = eps.add_state(State { data: next(ids) });
                eps.set_final(eps_final);
                eps.add_transition(Transition { source: eps.get_initial(), data: None, target: eps_final });
                eps
            },
            RE::Symbol { symbol } => {
                let mut sym = ENFA::new(State { data: next(ids) });
                let sym_final = sym.add_state(State { data: next(ids) });
                sym.set_final(sym_final);
                sym.add_transition(Transition { source: sym.get_initial(), data: Some(symbol), target: sym_final });
                sym
            },
            RE::Alternation { res } => {
                let mut alt = ENFA::new(State { data: next(ids) });
                let alt_final = alt.add_state(State { data: next(ids) });
                alt.set_final(alt_final);
                for re in res {
                    let re = ENFA::from_re(re, ids);
                    for transition in re.transitions() {
                        let transition = re.get_transition(transition);
                        let (source, target) = match (alt.contains_state(re.get_state(transition.source)), alt.contains_state(re.get_state(transition.target))) {
                            (Some(source), Some(target)) => (source,                                                 target),
                            (Some(source), None)         => (source,                                                 alt.add_state(re.get_state(transition.target).clone())),
                            (None,         Some(target)) => (alt.add_state(re.get_state(transition.source).clone()), target),
                            (None,         None)         => (alt.add_state(re.get_state(transition.source).clone()), alt.add_state(re.get_state(transition.target).clone())),
                        };
                        alt.add_transition(Transition { source, data: transition.data.clone(), target });
                    }
                    let re_initial = match alt.contains_state(re.get_state(re.get_initial())) {
                        Some(re_initial) => re_initial,
                        None             => alt.add_state(re.get_state(re.get_initial()).clone()),
                    };
                    alt.add_transition(Transition { source: alt.get_initial(), data: None, target: re_initial });
                    for re_final in re.finals() {
                        let re_final = match alt.contains_state(re.get_state(re_final)) {
                            Some(re_final) => re_final,
                            None           => alt.add_state(re.get_state(re_final).clone()),
                        };
                        alt.add_transition(Transition { source: re_final, data: None, target: alt_final });
                    }
                }
                alt
            },
            RE::Concatenation { res } => {
                let mut cat = ENFA::new(State { data: next(ids) });
                let cat_final = cat.add_state(State { data: next(ids) });
                cat.set_final(cat_final);
                let mut prev_re_finals = set![cat.get_initial()];
                for re in res {
                    let re = ENFA::from_re(re, ids);
                    for transition in re.transitions() {
                        let transition = re.get_transition(transition);
                        let (source, target) = match (cat.contains_state(re.get_state(transition.source)), cat.contains_state(re.get_state(transition.target))) {
                            (Some(source), Some(target)) => (source,                                                 target),
                            (Some(source), None)         => (source,                                                 cat.add_state(re.get_state(transition.target).clone())),
                            (None,         Some(target)) => (cat.add_state(re.get_state(transition.source).clone()), target),
                            (None,         None)         => (cat.add_state(re.get_state(transition.source).clone()), cat.add_state(re.get_state(transition.target).clone())),
                        };
                        cat.add_transition(Transition { source, data: transition.data.clone(), target });
                    }
                    let re_initial = match cat.contains_state(re.get_state(re.get_initial())) {
                        Some(re_initial) => re_initial,
                        None             => cat.add_state(re.get_state(re.get_initial()).clone()),
                    };
                    for prev_re_final in prev_re_finals {
                        cat.add_transition(Transition { source: prev_re_final, data: None, target: re_initial });
                    }
                    prev_re_finals = re.finals().map(|re_final| {
                        match cat.contains_state(re.get_state(re_final)) {
                            Some(re_final) => re_final,
                            None           => cat.add_state(re.get_state(re_final).clone()),
                        }
                    }).collect();
                }
                for prev_re_final in prev_re_finals {
                    cat.add_transition(Transition { source: prev_re_final, data: None, target: cat_final });
                }
                cat
            },
            RE::Repetition { re } => {
                let mut rep = ENFA::new(State { data: next(ids) });
                let rep_final = rep.add_state(State { data: next(ids) });
                rep.set_final(rep_final);
                let re = ENFA::from_re(*re, ids);
                for transition in re.transitions() {
                    let transition = re.get_transition(transition);
                    let (source, target) = match (rep.contains_state(re.get_state(transition.source)), rep.contains_state(re.get_state(transition.target))) {
                        (Some(source), Some(target)) => (source,                                                 target),
                        (Some(source), None)         => (source,                                                 rep.add_state(re.get_state(transition.target).clone())),
                        (None,         Some(target)) => (rep.add_state(re.get_state(transition.source).clone()), target),
                        (None,         None)         => (rep.add_state(re.get_state(transition.source).clone()), rep.add_state(re.get_state(transition.target).clone())),
                    };
                    rep.add_transition(Transition { source, data: transition.data.clone(), target });
                }
                let re_initial = match rep.contains_state(re.get_state(re.get_initial())) {
                    Some(re_initial) => re_initial,
                    None             => rep.add_state(re.get_state(re.get_initial()).clone()),
                };
                rep.add_transition(Transition { source: rep.get_initial(), data: None, target: re_initial });
                for re_final in re.finals() {
                    let re_final = match rep.contains_state(re.get_state(re_final)) {
                        Some(re_final) => re_final,
                        None           => rep.add_state(re.get_state(re_final).clone()),
                    };
                    rep.add_transition(Transition { source: re_final, data: None, target: rep_final });
                    rep.add_transition(Transition { source: re_final, data: None, target: re_initial });
                }
                rep.add_transition(Transition { source: rep.get_initial(), data: None, target: rep_final });
                rep
            },
        }
    }
}

impl From<RE> for ENFA<u32, char> {
    fn from(re: RE) -> ENFA<u32, char> {
        ENFA::from_re(re, &mut (0..u32::MAX))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::hash::Hash;
    use std::fmt::Debug;

    use crate::fa::{FA, ENFA};
    use crate::re::RE;

    struct Expected<S, T> {
        initial: S,
        transitions: BTreeSet<(S, T, S)>,
        finals: BTreeSet<S>
    }

    fn assert_eq<S: Clone + Debug + Eq + Hash + Ord, T: Clone + Debug + Eq + Hash + Ord>(expected: Expected<S, T>, actual: FA<S, T>) {
        assert_eq!(expected.initial, actual.get_state(actual.get_initial()).data);
        assert_eq!(expected.transitions, actual.transitions().map(|transition| actual.get_transition(transition)).map(|transition| (actual.get_state(transition.source).data.clone(), transition.data.clone(), actual.get_state(transition.target).data.clone())).collect());
        assert_eq!(expected.finals, actual.finals().map(|fin| actual.get_state(fin).data.clone()).collect());
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

    #[test]
    fn test_6() {
        let expected = Expected {
            initial: set![0, 1],
            transitions: set![],
            finals: set![set![0, 1]]
        };
        let actual = ENFA::from(RE::Epsilon).to_dfa();
        assert_eq(expected, actual);
    }

    #[test]
    fn test_7() {
        let expected = Expected {
            initial: set![0],
            transitions: set![
                (set![0], 'A', set![1])
            ],
            finals: set![set![1]]
        };
        let actual = ENFA::from(RE::Symbol { symbol: 'A' }).to_dfa();
        assert_eq(expected, actual);
    }

    #[test]
    fn test_8() {
        let expected = Expected {
            initial: set![0, 1, 2, 3, 4],
            transitions: set![
                (set![0, 1, 2, 3, 4], 'A', set![1, 5])
            ],
            finals: set![set![0, 1, 2, 3, 4], set![1, 5]]
        };
        let actual = ENFA::from(RE::Alternation {
            res: vec![
                RE::Epsilon,
                RE::Symbol { symbol: 'A' }
            ]
        }).to_dfa();
        assert_eq(expected, actual);
    }

    #[test]
    fn test_9() {
        let expected = Expected {
            initial: set![0, 2],
            transitions: set![
                (set![0, 2], 'A', set![1, 3, 4, 5])
            ],
            finals: set![set![1, 3, 4, 5]]
        };
        let actual = ENFA::from(RE::Concatenation {
            res: vec![
                RE::Symbol { symbol: 'A' },
                RE::Epsilon
            ]
        }).to_dfa();
        assert_eq(expected, actual);
    }

    #[test]
    fn test_10() {
        let expected = Expected {
            initial: set![0, 1, 2],
            transitions: set![
                (set![0, 1, 2], 'A', set![1, 2, 3]),
                (set![1, 2, 3], 'A', set![1, 2, 3])
            ],
            finals: set![set![0, 1, 2], set![1, 2, 3]]
        };
        let actual = ENFA::from(RE::Repetition {
            re: Box::new(RE::Symbol { symbol: 'A' })
        }).to_dfa();
        assert_eq(expected, actual);
    }
}
