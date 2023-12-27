.PHONY: usage build test deps-macos check-llvm

usage:
	@echo "Usage:"
	@echo "    build"
	@echo "    test"
	@echo "    deps-macos, for installing the dependencies on macOS."
	@echo "    check-llvm, for checking the LLVM installation and required environment variables."

check-llvm:
ifndef MLIR_SYS_170_PREFIX
	$(error Could not find a suitable LLVM 17 toolchain (mlir), please set MLIR_SYS_170_PREFIX env pointing to the LLVM 17 dir)
endif
ifndef TABLEGEN_170_PREFIX
	$(error Could not find a suitable LLVM 17 toolchain (tablegen), please set TABLEGEN_170_PREFIX env pointing to the LLVM 17 dir)
endif
	@echo "LLVM is correctly set at $(MLIR_SYS_170_PREFIX)."

build: check-llvm
	cargo build --all

test: build
	cargo test
	
deps-macos:
	-brew install llvm@17 --quiet
	@echo "You can execute the env-macos.sh script to setup the needed env variables."
