use austral_lib::ast::ModuleDef;
use chumsky::Parser as _;
use clap::Parser;
use std::fs;

fn main() {
    let args = CmdLine::parse();
    let input_file = fs::read_to_string(args.input_path).unwrap();

    let tokens = austral_lib::lexer::lex(input_file.as_str())
        .map(|(token, _span)| token)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let ast = ModuleDef::parser().parse(&tokens).into_result().unwrap();
    dbg!(&ast);
}

#[derive(Clone, Debug, Parser)]
struct CmdLine {
    input_path: String,
}
