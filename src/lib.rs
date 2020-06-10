enum NonTerminal {
    Grammar,
    Production,
    Consumption,
    Lookahead,
    Alternation,
    Concatenation,
    Repetition,
    Atom,
    CharacterClass,
    CharacterClassRange,
    Literal,
    RepetitionRange,
    RepetitionMinimum,
    RepetitionMaximum,
    RepetitionExact,
}

enum Token {
    PRODUCTION_OPERATOR,
    TOKEN,
    SEMICOLON,
    SLASH,
    VERTICAL_BAR,
    ASTERISK,
    PLUS_SIGN,
    QUESTION_MARK,
    CARET,
    DOLLAR_SIGN,
    FULL_STOP,
    LEFT_PARENTHESIS,
    RIGHT_PARENTHESIS,
    LEFT_SQUARE_BRACKET,
    RIGHT_SQUARE_BRACKET,
    HYPHEN,
    UNESCAPED_LITERAL,
    ESCAPED_LITERAL,
    CONTROL_LITERAL,
    OCTAL_LITERAL,
    HEXADECIMAL_LITERAL,
    UNICODE_LITERAL,
    LEFT_CURLY_BRACKET,
    RIGHT_CURLY_BRACKET,
    COMMA,
    INTEGER,
    NONE,
}

lazy_static! {
    static ref PARSER_PRODUCTIONS: HashMap<NonTerminal, Box<Term<NonTerminal, Token>>> = parser_productions![
        Grammar ::= plus!(alt!(nt!(Production), nt!(Consumption)));
        Production ::= cat!(nt!(Lookahead), t!(PRODUCTION_OPERATOR), t!(TOKEN), t!(SEMICOLON));
        Consumption ::= cat!(nt!(Lookahead), t!(PRODUCTION_OPERATOR), t!(SEMICOLON));
        Lookahead ::= cat!(nt!(Alternation), star!(cat!(t!(SLASH), nt!(Alternation))));
        Alternation ::= cat!(nt!(Concatenation), star!(cat!(t!(VERTICAL_BAR), nt!(Concatenation))));
        Concatenation ::= plus!(nt!(Repetition));
        Repetition ::= cat!(nt!(Atom), qm!(alt!(t!(ASTERISK), t!(PLUS_SIGN), t!(QUESTION_MARK), nt!(RepetitionExact), nt!(RepetitionMinimum), nt!(RepetitionMaximum), nt!(RepetitionRange))));
        Atom ::= alt!(nt!(CharacterClass), nt!(Literal), t!(CARET), t!(DOLLAR_SIGN), t!(FULL_STOP), cat!(t!(LEFT_PARENTHESIS), nt!(Lookahead), t!(RIGHT_PARENTHESIS)));
        CharacterClass ::= cat!(t!(LEFT_SQUARE_BRACKET), qm!(t!(CARET)), plus!(alt!(nt!(CharacterClassRange), nt!(Literal))), t!(RIGHT_SQUARE_BRACKET));
        CharacterClassRange ::= cat!(nt!(Literal), t!(HYPHEN), nt!(Literal));
        Literal ::= alt!(t!(UNESCAPED_LITERAL), t!(ESCAPED_LITERAL), t!(CONTROL_LITERAL), t!(OCTAL_LITERAL), t!(HEXADECIMAL_LITERAL), t!(UNICODE_LITERAL));
        RepetitionRange ::= cat!(t!(LEFT_CURLY_BRACKET), t!(INTEGER), t!(COMMA), t!(INTEGER), t!(RIGHT_CURLY_BRACKET));
        RepetitionMinimum ::= cat!(t!(LEFT_CURLY_BRACKET), t!(INTEGER), t!(COMMA), t!(RIGHT_CURLY_BRACKET));
        RepetitionMaximum ::= cat!(t!(LEFT_CURLY_BRACKET), t!(COMMA), t!(INTEGER), t!(RIGHT_CURLY_BRACKET));
        RepetitionExact ::= cat!(t!(LEFT_CURLY_BRACKET), t!(INTEGER), t!(RIGHT_CURLY_BRACKET));
    ];
    static ref LEXER_PRODUCTIONS: HashMap<Box<Pattern>, Token> = lexer_productions![
        cat!(l!('='), l!('>')) => PRODUCTION_OPERATOR;
        cc!(ccr!(l!('A'), l!('Z')), l!('_')) => TOKEN;
        l!(';') => SEMICOLON;
        l!('/') => SLASH;
        l!('|') => VERTICAL_BAR;
        l!('*') => ASTERISK;
        l!('+') => PLUS_SIGN;
        l!('?') => QUESTION_MARK;
        l!('^') => CARET;
        l!('$') => DOLLAR_SIGN;
        l!('.') => FULL_STOP;
        l!('(') => LEFT_PARENTHESIS;
        l!(')') => RIGHT_PARENTHESIS;
        l!('[') => LEFT_SQUARE_BRACKET;
        l!(']') => RIGHT_SQUARE_BRACKET;
        l!('-') => HYPHEN;
        ncc!(l!('.'), l!('^'), l!('$'), l!('*'), l!('+'), l!('-'), l!('?'), l!('('), l!(')'), l!('['), l!(']'), l!('{'), l!('}'), l!('\\'), l!('|'), l!('/')) => UNESCAPED_LITERAL;
        cc!(l!('.'), l!('^'), l!('$'), l!('*'), l!('+'), l!('-'), l!('?'), l!('('), l!(')'), l!('['), l!(']'), l!('{'), l!('}'), l!('\\'), l!('|'), l!('/')) => ESCAPED_LITERAL;
        cat!(l!('\\'), cc!(l!('a'), l!('b'), l!('f'), l!('n'), l!('r'), l!('t'), l!('v'))) => CONTROL_LITERAL;
        cat!(l!('\\'), rr!(cc!(ccr!(l!('0'), l!('7'))), 1, 3)) => OCTAL_LITERAL;
        cat!(l!('\\'), l!('x'), rmx!(cc!(ccr!(l!('0'), l!('9')), ccr!(l!('a'), l!('f')), ccr!(l!('A'), l!('F'))), 2)) => HEXADECIMAL_LITERAL;
        cat!(l!('\\'), alt!(cat!(l!('u'), rex!(cc!(ccr!(l!('0'), l!('9')), ccr!(l!('a'), l!('f')), ccr!(l!('A'), l!('F'))), 4)), cat!(l!('U'), rex!(cc!(ccr!(l!('0'), l!('9')), ccr!(l!('a'), l!('f')), ccr!(l!('A'), l!('F'))), 8)))) => UNICODE_LITERAL;
        l!('{') => LEFT_CURLY_BRACKET;
        l!('}') => RIGHT_CURLY_BRACKET;
        l!(',') => COMMA;
        plus!(cc!(ccr!(l!('0'), l!('9')))) => INTEGER;
        alt!(cat!(l!('/'), l!('/'), star!(any!()), eol!()), cat!(l!('/'), l!('*'), star!(any!()), l!('*'), l!('/'))) => NONE;
        cc!(l!(' '), l!('\t'), l!('\n'), l!('\r')) => NONE;
    ];
}
