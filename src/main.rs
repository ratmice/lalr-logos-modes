use lalrpop_util::lalrpop_mod;
mod lex;

lalrpop_mod!(parser);

fn main() {
    let s = r#""Hello W\u{00f4}rld\n""#;
    println!(
        "foo {:?}",
        parser::modeParser::new().parse(lex::ModeBridge {
            mode: lex::Modes::new(s)
        })
    );
}
