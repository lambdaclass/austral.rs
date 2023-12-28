# OCaml code analysis

## Module source types

- **TwoFileModuleSource:** used when compiling a combo of interface and implementation files
- **BodyModuleSource:** used when compiling a single file, without the associated interface file

See the definition [here](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/Compiler.ml#L56).

## Pervasive modules

These are modules that are imported by every Austral module: `Option`, `Either`, `fixedArraySize`, `abort`, `ExitCode`, among others.

See the definition [here](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/BuiltIn.ml#L15).

## Compilation call stack

The [Entrypoint function](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/Cli.ml#L13) is called when running `austral` and it calls `Austral_core.Cli.main`.

The flow for `austral compile file.aum` ([BodyModuleSource](#module-source-types)) would be:

1. `Cli.main` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/Cli.ml#L13)
    1. `CliUtil.parse_args` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/CliUtil.ml#L73)
    1. `CliParser.parse` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/CliParser.ml#L229)
    1. `CliEngine.exec` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/CliEngine.ml#L72) → `CliEngine.exec_compile` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/CliEngine.ml#L119) → `CliEngine.exec_target` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/CliEngine.ml#L164) → `CliEngine.exec_compile_to_bin` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/CliEngine.ml#L175) → `Compiler.compile_multiple` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/Compiler.ml#L111) → `Compiler.compile_mod` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/Compiler.ml#L92)
        1. `Compiler.parse_and_combine` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/Compiler.ml#L68): [Parsing, Lexing and Combining](#parsing-lexing-and-combining) 
        1. `ReturnCheck.check_ends_in_return` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/ReturnCheck.ml#L69): [Return check](#return-check)
        1. `DesugaringPass.desugar` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/DesugaringPass.ml#L30)
        1. `ExtractionPass.extract` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/ExtractionPass.ml#L201): [Extraction pass](#extraction-pass)
        1. `TypingPass.augment_module` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/TypingPass.ml#L663): Receives the AST and converts it to a TAST.
        1. `LinearityCheck.check_module_linearity` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/LinearityCheck.ml#L758): Receives the TAST.
        1. `BodyExtractionPass.extract_bodies` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/BodyExtractionPass.ml#L10)
        1. `Monomorphize.monomorphize` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/Monomorphize.ml#L522): Resolves generic functions
        1. `CodeGen.gen_module` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/CodeGen.ml#L724) + `CRenderer.render_unit` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/CRenderer.ml#L33): Generates C code

### Parsing, Lexing and Combining

See the implementation [here](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/Compiler.ml#L68).

1. `Compiler.add_file` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/Env.ml#L95): Adds the input file to the Environment
1.  `ParserInterface.parse_module_body` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/ParserInterface.ml#L33): [Parser](#parser)
    1. `ParserInterface.parse'` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/ParserInterface.ml#L17): Lexer
1. `Compiler.append_import_to_body` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/Compiler.ml#L42): Adds the [pervasive modules](#pervasive-modules) to the CST.
1. `CombiningPass.body_as_combined` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/CombiningPass.ml#L450): [Combining Pass](#combining-pass)

#### Parser

Generates the CST.

Generates definitions, called `Concrete*Def` (e.g. `ConcreteConstantDef`, `ConcreteFunctionDef`, etc.).

#### Combining Pass

In this case, it defines an empty interface module for the input file.

This includes the abstraction pass, where the CST becomes an AST.
The corresponding call stack is:

1. `CombiningPass.body_as_combined` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/CombiningPass.ml#L450): [Combining Pass](#combining-pass) → `CombiningPass.parse_defs` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/CombiningPass.ml#L410) → `CombiningPass.parse_def` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/CombiningPass.ml#L413) → `CombiningPass.private_def` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/CombiningPass.ml#L260)
    1. `AbstractionPass.abs_expr` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/AbstractionPass.ml#L104) + `AbstractionPass.abs_stmt` [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/AbstractionPass.ml#L17)

It also parses the definitions generated by the [parser](#parser) into declarations.
It renames the definitions `Concrete*Def` into the declarations `C*` (e.g. `ConcreteFunctionDef` is converted into `CFunction`).

### Return check

It only works for function and instance declarations (i.e. `CFunction` and `CInstance`).
It bypasses the check for the rest of the types.

It recursively goes through for each declaration, calling the `ends_in_return` function.
For details, see [here](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/ReturnCheck.ml#L33).

### Extraction Pass

It adds the following to the Environment:
- The input module to the module table. [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/ExtractionPass.ml#L243).
- The module declarations `C*` (e.g. `CFunction`, `CUnion`, etc.) to the declarations table. [code](https://github.com/austral/austral/blob/246f521c46825b58f81b2e489d2933be4e5ed9ad/lib/ExtractionPass.ml#L248).
