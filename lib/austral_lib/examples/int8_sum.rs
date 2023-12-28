use austral_lib::ast::ModuleDef;
use chumsky::Parser;
use melior::{dialect::DialectRegistry, Context};

fn main() {
    let tokens = austral_lib::lexer::lex(include_str!("../../../programs/examples/int8_sum.aum"))
        .map(|(token, _span)| token)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let ast = ModuleDef::parser().parse(&tokens).into_result().unwrap();

    let context = Context::new();
    context.append_dialect_registry(&{
        let dialect_registry = DialectRegistry::new();
        melior::utility::register_all_dialects(&dialect_registry);
        dialect_registry
    });
    context.load_all_available_dialects();

    let prog = austral_lib::compiler::compile(&context, &ast, &[]);
    println!("{}", prog.as_operation());
}
