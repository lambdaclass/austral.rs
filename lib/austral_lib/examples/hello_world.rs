use austral_lib::ast::ModuleDef;
use chumsky::Parser;

fn main() {
    let tokens =
        austral_lib::lexer::lex(include_str!("../../../programs/examples/hello_world.aum"))
            .map(|(token, _span)| token)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
    dbg!(&tokens);

    let ast = ModuleDef::parser().parse(&tokens).into_result().unwrap();
    dbg!(&ast);
}
