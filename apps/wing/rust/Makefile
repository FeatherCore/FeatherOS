############################################################################
# apps/wing/rust/Makefile
############################################################################

include $(APPDIR)/Make.defs
include $(APPDIR)/tools/Rust.mk

MODULE = $(CONFIG_WING_RUST)

RUST_CARGO_PATH := $(APPDIR)/wing/rust
RUST_TARGET := $(strip $(call RUST_TARGET_TRIPLE))
WING_W10M_RESOURCE := $(APPDIR)/wing/resource/windows10_mobile
WING_SIM_RESOURCE := $(TOPDIR)/boards/sim/sim/sim/src/etc/wing/resource/windows10_mobile

ifeq ($(CONFIG_ARCH_SIM),y)
context::
	@echo "Checking Wing Rust core (native, no_std)..."
	cargo build --manifest-path $(RUST_CARGO_PATH)/Cargo.toml --no-default-features
	@echo "Installing Wing Windows 10 Mobile resources into sim ROMFS..."
	@rm -rf $(WING_SIM_RESOURCE)
	@mkdir -p $(WING_SIM_RESOURCE)
	@cp -R $(WING_W10M_RESOURCE)/. $(WING_SIM_RESOURCE)/
else
context::
	@echo "Checking Wing Rust core..."
	@echo "Target: $(RUST_TARGET)"
	cargo build --manifest-path $(RUST_CARGO_PATH)/Cargo.toml --target $(RUST_TARGET) --no-default-features
endif

clean::
	cargo clean --manifest-path $(RUST_CARGO_PATH)/Cargo.toml

include $(APPDIR)/Application.mk
