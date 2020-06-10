pub(crate) enum RE {
    Epsilon,
    Symbol { symbol: char },
    Alternation { res: Vec<RE> },
    Concatenation { res: Vec<RE> },
    Repetition { re: Box<RE> },
}
