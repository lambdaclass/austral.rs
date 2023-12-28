use crate::ast::{
    AtomicExpr, Expression, FunctionDef, ModuleDecl, ModuleDef, ModuleDefItem, TypeSpec,
};
use melior::{
    dialect::{arith, func},
    ir::{
        attribute::{IntegerAttribute, StringAttribute, TypeAttribute},
        r#type::{FunctionType, IntegerType},
        Block, Location, Module, Region, Type, Value,
    },
    Context,
};
use std::{borrow::Cow, ops::Deref};

pub struct BuildContext<'c> {
    context: &'c Context,
    module: Module<'c>,
}

impl<'c> Deref for BuildContext<'c> {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        self.context
    }
}

pub fn compile<'c>(
    context: &'c Context,
    root: &ModuleDef,
    _interfaces: &[ModuleDecl],
) -> Module<'c> {
    let build_context = BuildContext {
        context,
        module: Module::new(Location::unknown(context)),
    };

    for module_item in &root.contents {
        match module_item {
            ModuleDefItem::Function(data) => compile_function(&build_context, data),
            _ => todo!(),
        }
    }

    build_context.module
}

fn compile_function(ctx: &BuildContext<'_>, root: &FunctionDef) {
    let region = Region::new();

    let block = region.append_block(Block::new(&[]));
    for stmt in &root.body {
        match stmt {
            crate::ast::Statement::Discard(expr) => {
                let _expr = process_expr(expr);
                build_expr(ctx, &block, expr);
            }
            crate::ast::Statement::Return(expr) => {
                let expr = process_expr(expr);
                let value = build_expr(ctx, &block, &expr);
                block.append_operation(func::r#return(&[value], Location::unknown(ctx)));
            }
            _ => todo!(),
        }
    }

    let arg_types = root
        .params
        .iter()
        .map(|_param| todo!())
        .collect::<Vec<Type>>();
    let ret_type = match &root.ret_type {
        TypeSpec::Simple { name } => match name.name.as_str() {
            "ExitCode" => IntegerType::new(ctx, 32).into(),
            _ => todo!(),
        },
        _ => todo!(),
    };

    ctx.module.body().append_operation(func::func(
        ctx,
        StringAttribute::new(ctx, &root.name.name),
        TypeAttribute::new(FunctionType::new(ctx, &arg_types, &[ret_type]).into()),
        region,
        &[],
        Location::unknown(ctx),
    ));
}

fn process_expr(expr: &Expression) -> Cow<Expression> {
    if let Expression::Atomic(AtomicExpr::FnCall(expr)) = expr {
        match expr.target.name.as_str() {
            "ExitSuccess" => return Cow::Owned(Expression::Atomic(AtomicExpr::ConstInt(0))),
            "ExitFailure" => return Cow::Owned(Expression::Atomic(AtomicExpr::ConstInt(1))),
            _ => {}
        }
    }

    Cow::Borrowed(expr)
}

fn build_expr<'c, 'b>(
    ctx: &'c BuildContext<'c>,
    block: &'b Block<'c>,
    expr: &Expression,
) -> Value<'c, 'b> {
    match expr {
        Expression::Atomic(expr) => match expr {
            AtomicExpr::ConstInt(value) => block
                .append_operation(arith::constant(
                    ctx,
                    IntegerAttribute::new(*value as i64, IntegerType::new(ctx, 32).into()).into(),
                    Location::unknown(ctx),
                ))
                .result(0)
                .unwrap()
                .into(),
            _ => todo!(),
        },
        Expression::Compound(_) => todo!(),
    }
}
