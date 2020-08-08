use std::str::FromStr;
use lexer_bootstrap::{
    error::Result,
    Token,
    Lexer as LexerBootstrap,
};
use crate::grammar::{
    LEXER_PRODUCTIONS,
    PARSER_PRODUCTIONS,
    Nonterminal,
    as_productions,
};
use parser_bootstrap::Parser;

pub struct Lexer<T> {
    lexer: LexerBootstrap<T>
}

impl<T: FromStr> Lexer<T> {
    pub fn new(productions: &str) -> Result<Lexer<T>> {
        let mut lexer = LexerBootstrap::new(LEXER_PRODUCTIONS.clone()); lexer.compile();
        let parser = Parser::new(PARSER_PRODUCTIONS.clone(), Nonterminal::Root);
        let tokens = lexer.lex(productions)?;
        let parse_tree = parser.parse(&tokens).unwrap();
        let productions = as_productions(&parse_tree)?;
        Ok(Lexer { lexer: LexerBootstrap::new(productions) })
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use lexer_bootstrap::{
        error::{
            Result,
            Error,
            ErrorKind,
        },
        Token,
    };
    use crate::Lexer;

    #[test]
    fn test_1() -> Result<()> {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        enum TokenKind {
            A,
            B,
        };
        impl FromStr for TokenKind {
            type Err = Error;
            fn from_str(text: &str) -> Result<Self> {
                use TokenKind::*;
                match text {
                    "A" => Ok(A),
                    "B" => Ok(B),
                    _ => Err(Error::from(ErrorKind::NotTokenKind))
                }
            }
        }
        use TokenKind::*;
        let mut lexer = Lexer::new(r#"
            /A/ => A;
            /B/ => B;
            / / => ;
        "#)?;
        lexer.compile();
        let expected = vec![
            Token::new(A, "A"),
            Token::new(B, "B"),
            Token::new(A, "A"),
        ];
        let actual = lexer.lex("A B  A   ")?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_2() -> Result<()> {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        #[allow(non_camel_case_types)]
        enum TokenKind {
            A_REP,
            B_REP
        };
        impl FromStr for TokenKind {
            type Err = Error;
            fn from_str(text: &str) -> Result<Self> {
                use TokenKind::*;
                match text {
                    "A_REP" => Ok(A_REP),
                    "B_REP" => Ok(B_REP),
                    _ => Err(Error::from(ErrorKind::NotTokenKind))
                }
            }
        }
        use TokenKind::*;
        let mut lexer = Lexer::new(r#"
            /A*/ => A_REP;
            /B*/ => B_REP;
            / / => ;
        "#)?;
        lexer.compile();
        let expected = vec![
            Token::new(A_REP, "AAAAAAA"), 
            Token::new(B_REP, "BBBB"),
            Token::new(B_REP, "BBBB"),
        ];
        let actual = lexer.lex("AAAAAAABBBB   BBBB")?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_3() -> Result<()> {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        #[allow(non_camel_case_types)]
        enum TokenKind {
            A,
            AB,
            BB,
            B,
        };
        impl FromStr for TokenKind {
            type Err = Error;
            fn from_str(text: &str) -> Result<Self> {
                use TokenKind::*;
                match text {
                    "A" => Ok(A),
                    "AB" => Ok(AB),
                    "BB" => Ok(BB),
                    "B" => Ok(B),
                    _ => Err(Error::from(ErrorKind::NotTokenKind))
                }
            }
        }
        use TokenKind::*;
        let mut lexer = Lexer::new(r#"
            /A/ => A;
            /AB/ => AB;
            /BB/ => BB;
            /B/ => B;
        "#)?;
        lexer.compile();
        let expected = vec![
            Token::new(AB, "AB"),
            Token::new(B, "B"),
        ];
        let actual = lexer.lex("ABB")?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_4() -> Result<()> {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        enum TokenKind {
            VERTICAL_BAR,
            ASTERISK,
            PLUS_SIGN,
            QUESTION_MARK,
            LEFT_PARENTHESIS,
            RIGHT_PARENTHESIS,
            LEFT_SQUARE_BRACKET,
            RIGHT_SQUARE_BRACKET,
            LEFT_CURLY_BRACKET,
            RIGHT_CURLY_BRACKET,
            CARET,
            HYPHEN,
            COMMA,
            DIGIT,
            CONTROL,
            UNESCAPED,
            ESCAPED,
            OCTAL,
            HEXADECIMAL,
            UNICODE,
        }
        impl FromStr for TokenKind {
            type Err = Error;
            fn from_str(text: &str) -> Result<Self> {
                use TokenKind::*;
                match text {
                    "VERTICAL_BAR" => Ok(VERTICAL_BAR),
                    "ASTERISK" => Ok(ASTERISK),
                    "PLUS_SIGN" => Ok(PLUS_SIGN),
                    "QUESTION_MARK" => Ok(QUESTION_MARK),
                    "LEFT_PARENTHESIS" => Ok(LEFT_PARENTHESIS),
                    "RIGHT_PARENTHESIS" => Ok(RIGHT_PARENTHESIS),
                    "LEFT_SQUARE_BRACKET" => Ok(LEFT_SQUARE_BRACKET),
                    "RIGHT_SQUARE_BRACKET" => Ok(RIGHT_SQUARE_BRACKET),
                    "LEFT_CURLY_BRACKET" => Ok(LEFT_CURLY_BRACKET),
                    "RIGHT_CURLY_BRACKET" => Ok(RIGHT_CURLY_BRACKET),
                    "CARET" => Ok(CARET),
                    "HYPHEN" => Ok(HYPHEN),
                    "COMMA" => Ok(COMMA),
                    "DIGIT" => Ok(DIGIT),
                    "CONTROL" => Ok(CONTROL),
                    "UNESCAPED" => Ok(UNESCAPED),
                    "ESCAPED" => Ok(ESCAPED),
                    "OCTAL" => Ok(OCTAL),
                    "HEXADECIMAL" => Ok(HEXADECIMAL),
                    "UNICODE" => Ok(UNICODE),
                    _ => Err(Error::from(ErrorKind::NotTokenKind))
                }
            }
        }
        use TokenKind::*;
        let mut lexer = Lexer::new(r#"
            /\|/ => VERTICAL_BAR;
            /\*/ => ASTERISK;
            /\+/ => PLUS_SIGN;
            /\?/ => QUESTION_MARK;
            /\(/ => LEFT_PARENTHESIS;
            /\)/ => RIGHT_PARENTHESIS;
            /\[/ => LEFT_SQUARE_BRACKET;
            /\]/ => RIGHT_SQUARE_BRACKET;
            /\{/ => LEFT_CURLY_BRACKET;
            /\}/ => RIGHT_CURLY_BRACKET;
            /\^/ => CARET;
            /\-/ => HYPHEN;
            /,/ => COMMA;
            /[0-9]/ => DIGIT;
            /\\[nrt]/ => CONTROL;
            /[^\/\|\*\+\?\(\)\[\]\{\}\^\-,0-9\n\r\t\\]/ => UNESCAPED;
            /\\[\/\|\*\+\?\(\)\[\]\{\}\^\-\\]/ => ESCAPED;
            /\\[0-7]{1,3}/ => OCTAL;
            /\\x[0-9a-fA-F]{1,2}/ => HEXADECIMAL;
            /\\(u[0-9a-fA-F]{4}|U[0-9a-fA-F]{8})/ => UNICODE;
        "#)?;
        lexer.compile();
        let expected = vec![
            Token::new(LEFT_SQUARE_BRACKET, "["),
            Token::new(UNESCAPED, "A"),
            Token::new(UNESCAPED, "🦄"),
            Token::new(ESCAPED, "\\^"),
            Token::new(RIGHT_SQUARE_BRACKET, "]"),
            Token::new(LEFT_CURLY_BRACKET, "{"),
            Token::new(DIGIT, "1"),
            Token::new(COMMA, ","),
            Token::new(DIGIT, "2"),
            Token::new(RIGHT_CURLY_BRACKET, "}"),
            Token::new(UNICODE, "\\UDEADBEEF"),
            Token::new(OCTAL, "\\777"),
            Token::new(HEXADECIMAL, "\\x45"),
        ];
        let actual = lexer.lex("[A🦄\\^]{1,2}\\UDEADBEEF\\777\\x45")?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
