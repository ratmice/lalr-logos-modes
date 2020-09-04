use logos::Lexer;
use logos::Logos;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
pub enum Outer {
    #[error]
    Error,

    #[token("\"")]
    StartString,

    #[regex(r"\p{White_Space}")]
    WhiteSpace,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Logos)]
pub enum Inner<'a> {
    #[error]
    Error,

    #[regex(r#"[^\\"]+"#)]
    Text(&'a str),

    #[token("\\n")]
    EscapedNewline,
    #[regex(r"\\u\{[^}]*\}")]
    EscapedCodepoint(&'a str),
    #[token(r#"\""#)]
    EscapedQuote,

    #[token("\"")]
    EndString,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Tokens<'a> {
    InnerToken(Inner<'a>),
    OuterToken(Outer),
}

pub enum Modes<'source> {
    Outer(Lexer<'source, Outer>),
    Inner(Lexer<'source, Inner<'source>>),
}

pub struct ModeBridge<'source> {
    pub mode: Modes<'source>,
}

// Clones as we switch between modes
impl<'source> Iterator for ModeBridge<'source> {
    type Item = (usize, Tokens<'source>, usize);
    fn next(&mut self) -> Option<Self::Item> {
        use Tokens::*;
        match &mut self.mode {
            Modes::Inner(inner) => {
                let result = inner.next().map(|token| (token, inner.span()));
                if Some(&Inner::EndString) == result.as_ref().map(|(t, _)| t) {
                    self.mode = Modes::Outer(inner.to_owned().morph());
                }
                result.map(|(tok, r)| (r.start, InnerToken(tok), r.end))
            }
            Modes::Outer(outer) => {
                let result = outer.next().map(|token| (token, outer.span()));
                if Some(&Outer::StartString) == result.as_ref().map(|(t, _)| t) {
                    self.mode = Modes::Inner(outer.to_owned().morph());
                }
                result.map(|(tok, r)| (r.start, OuterToken(tok), r.end))
            }
        }
    }
}


impl<'source> Modes<'source> {
    pub fn new(s: &'source str) -> Self {
        Self::Outer(Outer::lexer(s))
    }
}

#[test]
fn iterating_modes() {
    use Inner::*;
    use Tokens::*;
    let s = r#""Hello W\u{00f4}rld\n""#;
    let moded = ModeBridge {
        mode: Modes::new(s),
    };

    let results: Vec<Tokens> = moded.collect();
    let expect = vec![
        OuterToken(Outer::StartString),
        InnerToken(Text),
        InnerToken(EscapedCodepoint),
        InnerToken(Text),
        InnerToken(EscapedNewline),
        InnerToken(EndString),
    ];
    assert_eq!(results, expect);
}
