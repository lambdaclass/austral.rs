use crate::ast::{
    ArithExpr, AtomicExpr, CompoundExpr, Expression, FunctionDef, LetStmtTarget, ModuleDecl,
    ModuleDef, ModuleDefItem, TypeSpec,
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
use std::{borrow::Cow, collections::HashMap, ops::Deref};

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

    let arg_types = root
        .params
        .iter()
        .map(|_param| todo!())
        .collect::<Vec<Type>>();
    let ret_type = build_type(ctx, &root.ret_type);

    let block = region.append_block(Block::new(&[]));
    let mut locals = HashMap::new();

    for stmt in &root.body {
        match stmt {
            crate::ast::Statement::Let(stmt) => {
                let expr = process_expr(&stmt.value);
                match &stmt.target {
                    LetStmtTarget::Simple { name, r#type } => {
                        let value =
                            build_expr(ctx, &block, &expr, Some(build_type(ctx, r#type)), &locals);
                        locals.insert(name.name.as_str(), value);
                    }
                    LetStmtTarget::Destructure(_) => todo!(),
                }
            }
            crate::ast::Statement::Discard(expr) => {
                let expr = process_expr(expr);
                build_expr(ctx, &block, &expr, None, &locals);
            }
            crate::ast::Statement::Return(expr) => {
                let expr = process_expr(expr);
                let value = build_expr(ctx, &block, &expr, Some(ret_type), &locals);
                block.append_operation(func::r#return(&[value], Location::unknown(ctx)));
            }
            _ => todo!(),
        }
    }

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

fn build_type<'c>(ctx: &'c BuildContext<'c>, r#type: &TypeSpec) -> Type<'c> {
    match r#type {
        TypeSpec::Simple { name } => match name.name.as_str() {
            "Int8" => IntegerType::new(ctx, 8).into(),
            "ExitCode" => IntegerType::new(ctx, 32).into(),
            _ => todo!(),
        },
        _ => todo!(),
    }
}

fn build_expr<'c, 'b>(
    ctx: &'c BuildContext<'c>,
    block: &'b Block<'c>,
    expr: &Expression,

    target_type: Option<Type<'c>>,
    locals: &HashMap<&str, Value<'c, 'b>>,
) -> Value<'c, 'b> {
    match expr {
        Expression::Atomic(expr) => match expr {
            AtomicExpr::ConstInt(value) => block
                .append_operation(arith::constant(
                    ctx,
                    IntegerAttribute::new(*value as i64, target_type.unwrap()).into(),
                    Location::unknown(ctx),
                ))
                .result(0)
                .unwrap()
                .into(),
            AtomicExpr::Path(expr) => {
                assert!(expr.extra.is_empty());
                *locals.get(expr.first.name.as_str()).unwrap()
            }
            _ => todo!(),
        },
        Expression::Compound(expr) => match expr {
            CompoundExpr::Arith(expr) => match expr {
                ArithExpr::Add(lhs, rhs) => {
                    let lhs = process_expr(&Expression::Atomic(lhs.clone())).into_owned();
                    let rhs = process_expr(&Expression::Atomic(rhs.clone())).into_owned();
                    let lhs_value = build_expr(ctx, block, &lhs, target_type, locals);
                    let rhs_value = build_expr(ctx, block, &rhs, target_type, locals);

                    block
                        .append_operation(arith::addi(lhs_value, rhs_value, Location::unknown(ctx)))
                        .result(0)
                        .unwrap()
                        .into()
                }
                _ => todo!(),
            },
            _ => todo!(),
        },
    }
}
