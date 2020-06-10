use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::nfa::NFA;

pub(crate) struct DFA<S, E> {
    initial: S,
    transitions: HashMap<S, HashMap<E, S>>,
    finals: HashSet<S>,
}

impl<S, E> DFA<S, E>
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
        self.transitions.get_mut(&start).unwrap().insert(event, end);
    }

    fn insert_final(&mut self, state: S) {
        if !self.transitions.contains_key(&state) {
            self.transitions.insert(state, HashMap::new());
        }
        self.finals.insert(state);
    }
}

impl<S, E> Extend<(S, HashMap<E, S>)> for DFA<S, E>
where 
    S: Copy + Eq + Hash,
    E: Copy + Eq + Hash
{
    fn extend<T: IntoIterator<Item = (S, HashMap<E, S>)>>(&mut self, iter: T) {
        for (start, transitions) in iter.into_iter() {
            for (event, end) in transitions.into_iter() {
                self.insert_transition(start, event, end);
            }
        }
    }
}

impl<S, E> From<NFA<S, E>> for DFA<S, E> 
where
    S: Eq + Hash,
    E: Eq + Hash
{
    fn from(_nfa: NFA<S, E>) -> DFA<S, E> {
        panic!("Not implemented")
    }
}
