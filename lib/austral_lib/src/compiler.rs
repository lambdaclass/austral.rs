use crate::{
    ast::{
        ArithExpr, AtomicExpr, CompoundExpr, Expression, FnCallArgs, FunctionDef, LetStmtTarget,
        ModuleDecl, ModuleDef, ModuleDefItem, TypeSpec,
    },
    backend::pass_manager::run_pass_manager,
    lexer,
};
use chumsky::Parser;
use llvm_sys::{
    core::{
        LLVMContextCreate, LLVMContextDispose, LLVMDisposeMemoryBuffer, LLVMDisposeMessage,
        LLVMDisposeModule, LLVMGetBufferSize, LLVMGetBufferStart,
    },
    prelude::{LLVMContextRef, LLVMMemoryBufferRef, LLVMModuleRef},
    target::{
        LLVM_InitializeAllAsmPrinters, LLVM_InitializeAllTargetInfos, LLVM_InitializeAllTargetMCs,
        LLVM_InitializeAllTargets,
    },
    target_machine::{
        LLVMCodeGenFileType, LLVMCodeGenOptLevel, LLVMCodeModel, LLVMCreateTargetMachine,
        LLVMDisposeTargetMachine, LLVMGetDefaultTargetTriple, LLVMGetHostCPUFeatures,
        LLVMGetHostCPUName, LLVMGetTargetFromTriple, LLVMRelocMode,
        LLVMTargetMachineEmitToMemoryBuffer, LLVMTargetRef,
    },
};
use melior::{
    dialect::{arith, func, index, llvm, memref, DialectRegistry},
    ir::{
        attribute::{
            DenseElementsAttribute, FlatSymbolRefAttribute, IntegerAttribute, StringAttribute,
            TypeAttribute,
        },
        operation::OperationBuilder,
        r#type::{FunctionType, IntegerType, MemRefType, RankedTensorType},
        Block, Identifier, Location, Module, Region, Type, Value,
    },
    utility::register_all_llvm_translations,
    Context,
};
use mlir_sys::MlirOperation;
use std::{
    borrow::Cow,
    collections::{hash_map::Entry, HashMap},
    ffi::CStr,
    fmt::Display,
    io::Write,
    mem::MaybeUninit,
    ops::Deref,
    path::Path,
    ptr::{addr_of_mut, null_mut},
    sync::{Mutex, OnceLock},
};
use tempfile::NamedTempFile;

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

pub fn compile_to_binary(
    input: &str,
    is_library: bool,
    output_filename: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let tokens = lexer::lex(input)
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
    register_all_llvm_translations(&context);
    context.load_all_available_dialects();

    let mut module = compile(&context, &ast, &[]);
    run_pass_manager(&context, &mut module)?;
    let object = module_to_object(&module, is_library)?;
    object_to_shared_lib(&object, output_filename)?;

    Ok(())
}

extern "C" {
    /// Translate operation that satisfies LLVM dialect module requirements into an LLVM IR module living in the given context.
    /// This translates operations from any dilalect that has a registered implementation of LLVMTranslationDialectInterface.
    fn mlirTranslateModuleToLLVMIR(
        module_operation_ptr: MlirOperation,
        llvm_context: LLVMContextRef,
    ) -> LLVMModuleRef;
}

#[derive(Debug, Clone)]
pub struct LLVMCompileError(String);

impl std::error::Error for LLVMCompileError {}

impl Display for LLVMCompileError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

pub fn module_to_object(
    module: &Module<'_>,
    is_library: bool,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    static INITIALIZED: OnceLock<()> = OnceLock::new();

    INITIALIZED.get_or_init(|| unsafe {
        LLVM_InitializeAllTargets();
        LLVM_InitializeAllTargetInfos();
        LLVM_InitializeAllTargetMCs();
        LLVM_InitializeAllAsmPrinters();
    });

    unsafe {
        let llvm_context = LLVMContextCreate();

        let op = module.as_operation().to_raw();

        let llvm_module = mlirTranslateModuleToLLVMIR(op, llvm_context);

        let mut null = null_mut();
        let mut error_buffer = addr_of_mut!(null);

        let target_triple = LLVMGetDefaultTargetTriple();
        let target_cpu = LLVMGetHostCPUName();
        let target_cpu_features = LLVMGetHostCPUFeatures();

        let mut target: MaybeUninit<LLVMTargetRef> = MaybeUninit::uninit();

        if LLVMGetTargetFromTriple(target_triple, target.as_mut_ptr(), error_buffer) != 0 {
            let error = CStr::from_ptr(*error_buffer);
            let err = error.to_string_lossy().to_string();
            LLVMDisposeMessage(*error_buffer);
            Err(LLVMCompileError(err))?;
        } else if !(*error_buffer).is_null() {
            LLVMDisposeMessage(*error_buffer);
            error_buffer = addr_of_mut!(null);
        }

        let target = target.assume_init();

        let machine = LLVMCreateTargetMachine(
            target,
            target_triple.cast(),
            target_cpu.cast(),
            target_cpu_features.cast(),
            LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
            if is_library {
                LLVMRelocMode::LLVMRelocDynamicNoPic
            } else {
                LLVMRelocMode::LLVMRelocDefault
            },
            LLVMCodeModel::LLVMCodeModelDefault,
        );

        let mut out_buf: MaybeUninit<LLVMMemoryBufferRef> = MaybeUninit::uninit();

        let ok = LLVMTargetMachineEmitToMemoryBuffer(
            machine,
            llvm_module,
            LLVMCodeGenFileType::LLVMObjectFile,
            error_buffer,
            out_buf.as_mut_ptr(),
        );

        if ok != 0 {
            let error = CStr::from_ptr(*error_buffer);
            let err = error.to_string_lossy().to_string();
            LLVMDisposeMessage(*error_buffer);
            Err(LLVMCompileError(err))?;
        } else if !(*error_buffer).is_null() {
            LLVMDisposeMessage(*error_buffer);
        }

        let out_buf = out_buf.assume_init();

        let out_buf_start: *const u8 = LLVMGetBufferStart(out_buf).cast();
        let out_buf_size = LLVMGetBufferSize(out_buf);

        // keep it in rust side
        let data = std::slice::from_raw_parts(out_buf_start, out_buf_size).to_vec();

        LLVMDisposeMemoryBuffer(out_buf);
        LLVMDisposeTargetMachine(machine);
        LLVMDisposeModule(llvm_module);
        LLVMContextDispose(llvm_context);

        Ok(data)
    }
}

pub fn object_to_shared_lib(object: &[u8], output_filename: &Path) -> Result<(), std::io::Error> {
    // linker seems to need a file and doesn't accept stdin
    let mut file = NamedTempFile::new()?;
    file.write_all(object)?;
    let file = file.into_temp_path();

    let args: &[&str] = {
        #[cfg(target_os = "macos")]
        {
            &[
                "-demangle",
                "-no_deduplicate",
                "-dynamic",
                "-dylib",
                "-L/usr/local/lib",
                "-L/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/lib",
                &file.display().to_string(),
                "-o",
                &output_filename.display().to_string(),
                "-lSystem",
            ]
        }
        #[cfg(target_os = "linux")]
        {
            &[
                "--hash-style=gnu",
                "--eh-frame-hdr",
                "-shared",
                "-o",
                &output_filename.display().to_string(),
                "-L/lib/../lib64",
                "-L/usr/lib/../lib64",
                "-lc",
                &file.display().to_string(),
            ]
        }
        #[cfg(target_os = "windows")]
        {
            unimplemented!()
        }
    };

    let mut linker = std::process::Command::new("ld");
    let proc = linker.args(args.iter()).spawn()?;
    proc.wait_with_output()?;
    Ok(())
}
