use lexer_bootstrap::{
    error::Result,
    Token,
    Lexer as LexerBootstrap,
};

pub struct Lexer<T> {
    lexer: LexerBootstrap<T>
}

impl<T> Lexer<T> {
    pub fn new(_productions: &str) -> Lexer<T> {
        panic!("Not implemented")
    }
}

impl<T: Clone + Ord> Lexer<T> {
    pub fn compile(&mut self) {
        self.lexer.compile()
    }

    pub fn lex(&self, text: &str) -> Result<Vec<Token<T>>> {
        self.lexer.lex(text)
    }
}
