PACKAGE_MANAGER = cargo
RUN_COMMAND = run
BUILD_COMMAND = build
TEST_COMMAND = test
FMT_COMMAND = fmt
CLIPPY_COMMAND = clippy

# Compile example and run
TPL_FILE = example.tpl
OUTPUT_FILE = out

run:
	$(PACKAGE_MANAGER) $(RUN_COMMAND) -- $(TPL_FILE) $(OUTPUT_FILE)
	./$(OUTPUT_FILE)

# Release build
build:
	$(PACKAGE_MANAGER) $(BUILD_COMMAND) --release

# Unit tests
test:
	$(PACKAGE_MANAGER) $(TEST_COMMAND)

# Formatting
FMT_ARGS = --emit=files

fmt:
	$(PACKAGE_MANAGER) $(FMT_COMMAND) -- $(FMT_ARGS)

# Clippy
CLIPPY_ARGS = --fix --allow-dirty -- -D warnings

clippy:
	$(PACKAGE_MANAGER) $(CLIPPY_COMMAND) $(CLIPPY_ARGS)
