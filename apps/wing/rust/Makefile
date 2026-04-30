############################################################################
# apps/wing/rust/Makefile
############################################################################

include $(APPDIR)/Make.defs
include $(APPDIR)/tools/Rust.mk

MODULE = $(CONFIG_WING_RUST)

RUST_CARGO_PATH := $(APPDIR)/wing/rust
RUST_TARGET := $(strip $(call RUST_TARGET_TRIPLE))
WING_W10M_RESOURCE := $(APPDIR)/wing/resource/windows10_mobile
WING_W10M_RESOURCE_MANIFEST := $(WING_W10M_RESOURCE)/romfs_manifest.txt
WING_SIM_ROOT := $(TOPDIR)/boards/sim/sim/sim/src/etc/wing
WING_SIM_RESOURCE := $(WING_SIM_ROOT)/resource/windows10_mobile
RESOURCE_BUDGET_CHECK := $(APPDIR)/tools/check_resource_budget.sh

ifeq ($(CONFIG_ARCH_SIM),y)
context::
	@echo "Checking Wing Rust core (native, no_std)..."
	cargo build --manifest-path $(RUST_CARGO_PATH)/Cargo.toml --no-default-features
	@echo "Checking and installing Wing Windows 10 Mobile resources into sim ROMFS..."
	@rm -rf $(WING_SIM_ROOT)
	@mkdir -p $(WING_SIM_RESOURCE)
	$(RESOURCE_BUDGET_CHECK) $(WING_W10M_RESOURCE) $(WING_W10M_RESOURCE_MANIFEST) $(WING_SIM_RESOURCE)
else
context::
	@echo "Checking Wing Rust core..."
	@echo "Target: $(RUST_TARGET)"
	cargo build --manifest-path $(RUST_CARGO_PATH)/Cargo.toml --target $(RUST_TARGET) --no-default-features
endif

clean::
	cargo clean --manifest-path $(RUST_CARGO_PATH)/Cargo.toml

include $(APPDIR)/Application.mk
