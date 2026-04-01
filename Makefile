# Cross-compile for Linux Ubuntu from macOS
TARGET = x86_64-unknown-linux-musl
RELEASE_DIR = target/$(TARGET)/release
BINARY_NAME = sing-box-config-gate

.PHONY: all build clean install-toolchain

all: build

# Install cross-compilation toolchain (run once on macOS)
install-toolchain:
	@echo "Installing Rust target for $(TARGET)..."
	rustup target add $(TARGET)
	@echo "Installing musl-cross toolchain..."
	brew install FiloSottile/musl-cross/musl-cross
	@echo "Toolchain installation complete!"

# Build Linux Ubuntu release binary
build:
	@echo "Building release for $(TARGET)..."
	CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=$(shell brew --prefix musl-cross)/bin/x86_64-linux-musl-gcc \
	cargo build --release --target $(TARGET)
	mkdir -p bin
	cp $(RELEASE_DIR)/$(BINARY_NAME) bin/$(BINARY_NAME)
	@echo "Build complete: bin/$(BINARY_NAME)"

# Clean build artifacts
clean:
	cargo clean

# Show binary info
info:
	@echo "Target: $(TARGET)"
	@echo "Release dir: $(RELEASE_DIR)"
	@echo "Binary: $(RELEASE_DIR)/$(BINARY_NAME)"
	@if [ -f "$(RELEASE_DIR)/$(BINARY_NAME)" ]; then \
		file $(RELEASE_DIR)/$(BINARY_NAME); \
	else \
		echo "Binary not found. Run 'make build' first."; \
	fi
