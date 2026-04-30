############################################################################
# apps/fhre/rust/Makefile
############################################################################

include $(APPDIR)/Make.defs
include $(APPDIR)/tools/Rust.mk

MODULE = $(CONFIG_FHRE_RUST)

RUST_CARGO_PATH := $(APPDIR)/fhre/rust
RUST_TARGET := $(strip $(call RUST_TARGET_TRIPLE))
FHRE_RESOURCE := $(APPDIR)/fhre/resource
FHRE_SIM_RESOURCE := $(TOPDIR)/boards/sim/sim/sim/src/etc/fhre/resource

ifeq ($(CONFIG_ARCH_SIM),y)
context::
	@echo "Checking FHRE Rust core (native, no_std)..."
	cargo build --manifest-path $(RUST_CARGO_PATH)/Cargo.toml --no-default-features
	@echo "Installing FHRE demo resources into sim ROMFS..."
	@rm -rf $(FHRE_SIM_RESOURCE)
	@mkdir -p $(FHRE_SIM_RESOURCE)
	@cp -R $(FHRE_RESOURCE)/. $(FHRE_SIM_RESOURCE)/
else
context::
	@echo "Checking FHRE Rust core..."
	@echo "Target: $(RUST_TARGET)"
	cargo build --manifest-path $(RUST_CARGO_PATH)/Cargo.toml --target $(RUST_TARGET) --no-default-features
endif

clean::
	cargo clean --manifest-path $(RUST_CARGO_PATH)/Cargo.toml

include $(APPDIR)/Application.mk
