use std::ops::AddAssign;
use uuid::Uuid;
use regular_expression::StateGenerator;

#[macro_use]
pub mod util;
pub mod error;
pub mod lexer;

pub use crate::lexer::{
    Token,
    Lexer
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TokenState<T> {
    uuid: u128,
    sequence_number: u128,
    token_kind: Option<T>,
}

impl<T: Clone> TokenState<T> {
    fn new(token_kind: Option<T>) -> TokenState<T> {
        TokenState { uuid: Uuid::new_v4().as_u128(), sequence_number: 0, token_kind }
    }

    fn token_kind(&self) -> &Option<T> {
        &self.token_kind
    }

    fn clear_token_kind(&mut self) {
        self.token_kind = None;
    }
}

impl<T> AddAssign<u128> for TokenState<T> {
    fn add_assign(&mut self, other: u128) {
        self.sequence_number += other;
    }
}

struct TokenStateGenerator<T> {
    token_state: TokenState<T>,
    final_enabled: bool,
}

impl<T: Clone> TokenStateGenerator<T> {
    pub fn new(token: Option<T>) -> TokenStateGenerator<T> {
        TokenStateGenerator { token_state: TokenState::new(token), final_enabled: true }
    }

    fn next_final_enabled(&mut self) -> TokenState<T> {
        let next = self.token_state.clone();
        self.token_state += 1;
        next
    }

    fn next_final_disabled(&mut self) -> TokenState<T> {
        let mut next = self.token_state.clone();
        self.token_state += 1;
        next.clear_token_kind();
        next
    }
}

impl<T: Clone> StateGenerator for TokenStateGenerator<T> {
    type State = TokenState<T>;

    fn next_initial(&mut self) -> TokenState<T> {
        let mut next = self.token_state.clone();
        self.token_state += 1;
        next.clear_token_kind();
        next
    }

    fn next_final(&mut self) -> TokenState<T> {
        if self.final_enabled {
            self.next_final_enabled()
        } else {
            self.next_final_disabled()
        }
    }

    fn disable_final(&mut self) -> &mut TokenStateGenerator<T> {
        self.final_enabled = false;
        self
    }

    fn enable_final(&mut self) -> &mut TokenStateGenerator<T> {
        self.final_enabled = true;
        self
    }
}

