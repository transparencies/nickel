use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum NormalToken<'input> {
    #[regex("[ \r\t\n]+", logos::skip)]

    #[error]
    Error,

    #[regex("_?[a-zA-Z][_a-zA-Z0-9]*")]
    Identifier(&'input str),
    #[regex("-?[0-9]*\\.?[0-9]+", |lex| lex.slice().parse())]
    NumLiteral(f64),
    #[regex("[+@=\\-<>\\.|#]+")]
    BinaryOp(&'input str),

    #[token("Dyn")]
    Dyn,
    #[token("Num")]
    Num,
    #[token("Bool")]
    Bool,
    #[token("Str")]
    Str,
    #[token("List")]
    List,

    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("forall")]
    Forall,
    #[token("in")]
    In,
    #[token("let")]
    Let,
    #[token("switch")]
    Switch,

    #[token("true")]
    True,
    #[token("false")]
    False,

    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("$")]
    Dollar,
    #[token("=")]
    Equals,
    #[token(";")]
    SemiCol,
    #[token(".")]
    Dot,
    #[token(".$")]
    DotDollar,
    #[token("$[")]
    DollarBracket,
    #[token("$=")]
    DollarEquals,
    #[token("${")]
    DollarBrace,
    #[token("\"")]
    DoubleQuote,
    #[token("-$")]
    MinusDollar,
    
    #[token("fun")]
    Fun,
    #[token("import")]
    Import,
    #[token("|")]
    Pipe,
    #[token("->")]
    SimpleArrow,
    #[token("=>")]
    DoubleArrow,
    #[token("#")]
    Hash,
    #[token("`")]
    Backtick,
    #[token("_")]
    Underscore,

    #[token("tag")]
    Tag,
    #[token("Assume(")]
    Assume,
    #[token("Promise(")]
    Promise,
    #[token("Default(")]
    Deflt,
    #[token("Contract(")]
    Contract,
    #[token("ContractDefault(")]
    ContractDeflt,
    #[token("Docstring(")]
    Docstring,

    #[token("isZero")]
    IsZero,
    #[token("isNum")]
    IsNum,
    #[token("isBool")]
    IsBool,
    #[token("isStr")]
    IsStr,
    #[token("isFun")]
    IsFun,
    #[token("isList")]
    IsList,
    #[token("blame")]
    Blame,
    #[token("chngPol")]
    ChangePol,
    #[token("polarity")]
    Polarity,
    #[token("goDom")]
    GoDom,
    #[token("goCodom")]
    GoCodom,
    #[token("wrap")]
    Wrap,
    #[token("embed")]
    Embed,
    #[token("mapRec")]
    MapRec,
    #[token("seq")]
    Seq,
    #[token("deepSeq")]
    DeepSeq,
    #[token("head")]
    Head,
    #[token("tail")]
    Tail,
    #[token("length")]
    Length,
    FieldsOf,

    #[token("unwrap")]
    Unwrap,
    #[token("hasField")]
    HasField,
    #[token("map")]
    Map,
    #[token("elemAt")]
    ElemAt,
    #[token("merge")]
    Merge,

    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("<")]
    LAngleBracket,
    #[token(">")]
    RAngleBracket,
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum StringToken<'input> {
    #[error]
    Error,

    #[regex("(([^\"\\$\\\\]+)|\\$[^{\"\\$\\\\])+\\$?")]
    Literal(&'input str),

    #[token("\"")]
    DoubleQuote,
    #[token("${")]
    DollarBrace,
    #[regex("\\\\.", |lex| lex.slice().chars().nth(1))]
    EscapedChar(char),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token<'input> {
    Normal(NormalToken<'input>),
    Str(StringToken<'input>),
}

pub enum ModalLexer<'input> {
    Normal(logos::Lexer<'input, NormalToken<'input>>),
    Str(logos::Lexer<'input, StringToken<'input>>),
}

impl<'input> Iterator for ModalLexer<'input> {
    type Item = Token<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ModalLexer::Normal(lexer) => lexer.next().map(Token::Normal),
            ModalLexer::Str(lexer) => lexer.next().map(Token::Str),
        }
    }
}

impl<'input> ModalLexer<'input> {
    pub fn span(&self) -> std::ops::Range<usize> {
        match self {
            ModalLexer::Normal(lexer) => lexer.span(),
            ModalLexer::Str(lexer) => lexer.span(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum LexicalError {
    /// A closing brace '}' does not match an opening brace '{'.
    UnmatchedCloseBrace(usize),
    /// Invalid escape sequence in a string literal.
    InvalidEscapeSequence(usize),
    /// Generic lexer error
    Generic(usize, usize),
}

pub struct Lexer<'input> {
    pub lexer: Option<ModalLexer<'input>>,
    pub brace_count: usize,
    pub brace_stack: Vec<usize>,
}

impl<'input> Lexer<'input> {
    pub fn new(s: &'input str) -> Self {
        Lexer {
            lexer: Some(ModalLexer::Normal(NormalToken::lexer(s))),
            brace_stack: Vec::new(),
            brace_count: 0,
        }
    }

    fn enter_str(&mut self) {
        match self.lexer.take() {
            Some(ModalLexer::Normal(lexer)) => {
                self.brace_stack.push(self.brace_count);
                self.brace_count = 0;
                self.lexer.replace(ModalLexer::Str(lexer.morph()));
            }
            _ => panic!("lexer::enter_str"),
        }
    }

    fn enter_normal(&mut self) {
        match self.lexer.take() {
            Some(ModalLexer::Str(lexer)) => {
                //brace_count must be zero, and we do not push it on the stack
                self.lexer.replace(ModalLexer::Normal(lexer.morph()));
            }
            _ => panic!("lexer::enter_normal"),
        }
    }

    fn leave_str(&mut self) {
        match self.lexer.take() {
            Some(ModalLexer::Str(lexer)) => {
                // We can only enter string mode from normal mode, so the brace stack should not be
                // empty
                self.brace_count = self.brace_stack.pop().unwrap();
                self.lexer.replace(ModalLexer::Normal(lexer.morph()));
            }
            _ => panic!("lexer::leave_str"),
        }
    }

    fn leave_normal(&mut self) {
        match self.lexer.take() {
            Some(ModalLexer::Normal(lexer)) => {
                // brace_count must be 0
                self.lexer.replace(ModalLexer::Normal(lexer.morph()));
            }
            _ => panic!("lexer::leave_normal"),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<(usize, Token<'input>, usize), LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        use Token::*;

        let lexer = self.lexer.as_mut().unwrap();
        let mut token = lexer.next();
        let span = lexer.span();

        match token.as_ref() {
            Some(Normal(NormalToken::DoubleQuote)) => self.enter_str(),
            Some(Normal(NormalToken::LBrace)) => self.brace_count += 1,
            Some(Normal(NormalToken::RBrace)) => {
                if self.brace_count == 0 {
                    if self.brace_stack.is_empty() {
                        return Some(Err(LexicalError::UnmatchedCloseBrace(span.start)));
                    }
                    self.leave_normal();
                }
                else {
                    self.brace_count -= 1;
                }
            },
            Some(Str(StringToken::DoubleQuote)) => {
                self.leave_str();
                // To make things simpler on the parser side, we only return one variant for
                // DoubleQuote, namely the the normal one 
                token = Some(Normal(NormalToken::DoubleQuote));
            }
            Some(Str(StringToken::DollarBrace)) =>
                self.enter_normal(),
            // Convert escape sequences
            Some(Str(StringToken::EscapedChar(c))) => {
                if let Some(esc) = escape_char(*c) {
                    token = Some(Str(StringToken::EscapedChar(esc)));
                }
                else {
                    return Some(Err(LexicalError::InvalidEscapeSequence(span.end)))
                }
            }
            // Early report errors for now. This could change in the future
            Some(Str(StringToken::Error)) | Some(Normal(NormalToken::Error)) =>
                return Some(Err(LexicalError::Generic(span.start, span.end))),
            _ => (),
        }
        
        token.map(|t| Ok((span.start, t, span.end)))
    }
}

fn escape_char(chr: char) -> Option<char> {
    match chr {
        '\'' => Some('\''),
        '"' => Some('"'),
        '\\' => Some('\\'),
        '$' => Some('$'),
        'n' => Some('\n'),
        'r' => Some('\r'),
        't' => Some('\t'),
        _ => { println!("WTF {}", chr); None} ,
    }
}
