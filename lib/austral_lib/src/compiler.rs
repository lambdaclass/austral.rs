use crate::ast::{
    ArithExpr, AtomicExpr, CompoundExpr, Expression, FnCallArgs, FunctionDef, LetStmtTarget,
    ModuleDecl, ModuleDef, ModuleDefItem, TypeSpec,
};
use melior::{
    dialect::{arith, func, index, llvm, memref},
    ir::{
        attribute::{
            DenseElementsAttribute, FlatSymbolRefAttribute, IntegerAttribute, StringAttribute,
            TypeAttribute,
        },
        operation::OperationBuilder,
        r#type::{FunctionType, IntegerType, MemRefType, RankedTensorType},
        Block, Identifier, Location, Module, Region, Type, Value,
    },
    Context, Error, pass::{PassManager, self},
};
use std::{
    borrow::Cow,
    collections::{hash_map::Entry, HashMap},
    ops::Deref,
    sync::Mutex,
};

struct BuildContext<'c> {
    context: &'c Context,
    module: Module<'c>,

    literal_str: Mutex<HashMap<String, usize>>,
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

        literal_str: Mutex::new(HashMap::default()),
    };

    build_context.module.body().append_operation(func::func(
        &build_context,
        StringAttribute::new(&build_context, "puts"),
        TypeAttribute::new(
            FunctionType::new(
                context,
                &[llvm::r#type::opaque_pointer(&build_context)],
                &[IntegerType::new(context, 32).into()],
            )
            .into(),
        ),
        Region::new(),
        &[(
            Identifier::new(&build_context, "sym_visibility"),
            StringAttribute::new(&build_context, "private").into(),
        )],
        Location::unknown(context),
    ));
    build_context.module.body().append_operation(func::func(
        &build_context,
        StringAttribute::new(&build_context, "putchar"),
        TypeAttribute::new(
            FunctionType::new(
                context,
                &[IntegerType::new(context, 8).into()],
                &[IntegerType::new(context, 32).into()],
            )
            .into(),
        ),
        Region::new(),
        &[(
            Identifier::new(&build_context, "sym_visibility"),
            StringAttribute::new(&build_context, "private").into(),
        )],
        Location::unknown(context),
    ));

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
            AtomicExpr::ConstStr(value) => {
                let mut literal_str = ctx.literal_str.lock().unwrap();

                let num_literals = literal_str.len();
                let literal_idx = match literal_str.entry(value.clone()) {
                    Entry::Occupied(entry) => *entry.get(),
                    Entry::Vacant(entry) => {
                        ctx.module.body().append_operation(memref::global(
                            ctx,
                            &format!("LiteralStr{num_literals}"),
                            None,
                            MemRefType::new(
                                IntegerType::new(ctx, 8).into(),
                                &[value.len() as u64],
                                None,
                                None,
                            ),
                            Some(
                                DenseElementsAttribute::new(
                                    RankedTensorType::new(
                                        &[value.len() as u64],
                                        IntegerType::new(ctx, 8).into(),
                                        None,
                                    )
                                    .into(),
                                    &value
                                        .bytes()
                                        .map(|x| {
                                            IntegerAttribute::new(
                                                x as i64,
                                                IntegerType::new(ctx, 8).into(),
                                            )
                                            .into()
                                        })
                                        .collect::<Vec<_>>(),
                                )
                                .unwrap()
                                .into(),
                            ),
                            true,
                            None,
                            Location::unknown(ctx),
                        ));

                        *entry.insert(num_literals)
                    }
                };

                let value = block
                    .append_operation(memref::get_global(
                        ctx,
                        &format!("LiteralStr{literal_idx}"),
                        MemRefType::new(
                            IntegerType::new(ctx, 8).into(),
                            &[value.len() as u64],
                            None,
                            None,
                        ),
                        Location::unknown(ctx),
                    ))
                    .result(0)
                    .unwrap()
                    .into();
                let value = block
                    .append_operation(
                        OperationBuilder::new(
                            "memref.extract_aligned_pointer_as_index",
                            Location::unknown(ctx),
                        )
                        .add_operands(&[value])
                        .add_results(&[Type::index(ctx)])
                        .build()
                        .unwrap(),
                    )
                    .result(0)
                    .unwrap()
                    .into();
                let value = block
                    .append_operation(index::castu(
                        value,
                        IntegerType::new(ctx, 64).into(),
                        Location::unknown(ctx),
                    ))
                    .result(0)
                    .unwrap()
                    .into();
                block
                    .append_operation(
                        OperationBuilder::new("llvm.inttoptr", Location::unknown(ctx))
                            .add_operands(&[value])
                            .add_results(&[llvm::r#type::opaque_pointer(ctx)])
                            .build()
                            .unwrap(),
                    )
                    .result(0)
                    .unwrap()
                    .into()
            }
            AtomicExpr::FnCall(expr) => {
                let args = match &expr.args {
                    FnCallArgs::Empty => todo!(),
                    FnCallArgs::Positional(args) => args.as_slice(),
                    FnCallArgs::Named(_) => todo!(),
                }
                .iter()
                .map(|expr| {
                    let expr = process_expr(expr);
                    build_expr(ctx, block, &expr, None, locals)
                })
                .collect::<Vec<_>>();

                match expr.target.name.as_str() {
                    "printLn" => {
                        block.append_operation(func::call(
                            ctx,
                            FlatSymbolRefAttribute::new(ctx, "puts"),
                            &args,
                            &[IntegerType::new(ctx, 32).into()],
                            Location::unknown(ctx),
                        ));

                        let k10 = block
                            .append_operation(arith::constant(
                                ctx,
                                IntegerAttribute::new(10, IntegerType::new(ctx, 8).into()).into(),
                                Location::unknown(ctx),
                            ))
                            .result(0)
                            .unwrap()
                            .into();
                        block
                            .append_operation(func::call(
                                ctx,
                                FlatSymbolRefAttribute::new(ctx, "putchar"),
                                &[k10],
                                &[IntegerType::new(ctx, 32).into()],
                                Location::unknown(ctx),
                            ))
                            .result(0)
                            .unwrap()
                            .into()
                    }
                    _ => todo!(),
                }
            }
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


pub fn run_pass_manager(context: &Context, module: &mut Module) -> Result<(), Error> {
    let pass_manager = PassManager::new(context);
    pass_manager.enable_verifier(true);
    // pass_manager.add_pass(pass::transform::create_canonicalizer());
    pass_manager.add_pass(pass::conversion::create_scf_to_control_flow());
    pass_manager.add_pass(pass::conversion::create_arith_to_llvm());
    pass_manager.add_pass(pass::conversion::create_control_flow_to_llvm());
    pass_manager.add_pass(pass::conversion::create_func_to_llvm());
    pass_manager.add_pass(pass::conversion::create_index_to_llvm());
    pass_manager.add_pass(pass::conversion::create_finalize_mem_ref_to_llvm());
    pass_manager.add_pass(pass::conversion::create_reconcile_unrealized_casts());
    pass_manager.run(module)
}
