# austral
An implementation of the Austral language compiler in Rust.
Austral is a new systems programming language. It uses linear types to provide memory safety and capability-secure code.

## Requisites
- LLVM 17 with MLIR support.

## Building

First make sure you have a working installation of LLVM with MLIR support. If you are using a Mac
you can run:
```bash
brew install llvm@17
```
It will install all the required dependencies.
To compile the project you need to first export some environment variables.
Look at the [env-macos.sh script](env-macos.sh) to see what is needed. Then if you are in using Mac:
```bash
source env-macos.sh
```
After that, `cargo build --all` should work.

## Running the CLI

Run:

To compile an Austral file:

```bash
cd bin/austral_bin
cargo r -- <module_file>
```

```bash
cd bin/austral_bin
cargo r -- ../../programs/examples/hello_world.aum
./a.out
# prints Hello world!
```

You can also print the parsed AST and get the MLIR and LLVM representation of the program.

To see all the available options run:
```bash
cargo r -- --help
```

## Status
- [x] Lexer
- [x] Parser
- [ ] Compilation Passes
- [ ] Type checker
- [x] Code generation of simple programs
- [ ] Full code generation

## Docs

[OCaml code analysis](docs/ocaml_code_analysis.md)

## Resources
- https://austral-lang.org/
- https://borretti.me/article/introducing-austral
- https://borretti.me/article/type-systems-memory-safety
- https://borretti.me/article/how-capabilities-work-austral
- https://borretti.me/article/design-austral-compiler
- https://borretti.me/article/how-australs-linear-type-checker-works
- https://borretti.me/article/linear-types-exceptions
- https://borretti.me/article/linear-types-safety
