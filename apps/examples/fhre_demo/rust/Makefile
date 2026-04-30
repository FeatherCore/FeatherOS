############################################################################
# apps/examples/fhre_demo/rust/Makefile
############################################################################

include $(APPDIR)/Make.defs
include $(APPDIR)/tools/Rust.mk

PROGNAME  = $(CONFIG_EXAMPLES_FHRE_DEMO_RUST_PROGNAME)
PRIORITY  = $(CONFIG_EXAMPLES_FHRE_DEMO_RUST_PRIORITY)
STACKSIZE = $(CONFIG_EXAMPLES_FHRE_DEMO_RUST_STACKSIZE)
MODULE    = $(CONFIG_EXAMPLES_FHRE_DEMO_RUST)

RUST_CARGO_PATH := $(APPDIR)/examples/fhre_demo/rust
RUST_TARGET := $(strip $(call RUST_TARGET_TRIPLE))

ifeq ($(CONFIG_ARCH_SIM),y)
context::
	@echo "Building FHRE Rust demo (native, no_std)..."
	cargo build --manifest-path $(RUST_CARGO_PATH)/Cargo.toml --no-default-features
else
context::
	@echo "Building FHRE Rust demo..."
	@echo "Target: $(RUST_TARGET)"
	cargo build --manifest-path $(RUST_CARGO_PATH)/Cargo.toml --target $(RUST_TARGET) --no-default-features
endif

clean::
	cargo clean --manifest-path $(RUST_CARGO_PATH)/Cargo.toml

include $(APPDIR)/Application.mk

