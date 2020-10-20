use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
pub enum Token<'input> {
    #[regex("[ \r\t\n]+", logos::skip)]
    Error,

    #[regex("_?[a-zA-Z][_a-zA-Z0-9]*")]
    Identifier(&'input str),
    #[regex("-?[0-9]+(\\.[0-9]+)?")]
    NumLiteral(f64),
    #[regex("[\\+@=-<>\\.|#]+")]
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

    StrLiteral(String),


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
    #[token("changePol")]
    ChangePol,
    #[token("pol")]
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
    #[token("[")]
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
