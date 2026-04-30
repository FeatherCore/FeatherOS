############################################################################
# apps/fhre/rust/Makefile
############################################################################

include $(APPDIR)/Make.defs
include $(APPDIR)/tools/Rust.mk

MODULE = $(CONFIG_FHRE_RUST)

RUST_CARGO_PATH := $(APPDIR)/fhre/rust
RUST_TARGET := $(strip $(call RUST_TARGET_TRIPLE))
FHRE_RESOURCE := $(APPDIR)/fhre/resource
FHRE_RESOURCE_MANIFEST := $(FHRE_RESOURCE)/romfs_manifest.txt
FHRE_SIM_ROOT := $(TOPDIR)/boards/sim/sim/sim/src/etc/fhre
FHRE_SIM_RESOURCE := $(FHRE_SIM_ROOT)/resource
RESOURCE_BUDGET_CHECK := $(APPDIR)/tools/check_resource_budget.sh

ifeq ($(CONFIG_ARCH_SIM),y)
context::
	@echo "Checking FHRE Rust core (native, no_std)..."
	cargo build --manifest-path $(RUST_CARGO_PATH)/Cargo.toml --no-default-features
	@echo "Checking and installing FHRE demo resources into sim ROMFS..."
	@rm -rf $(FHRE_SIM_ROOT)
	@mkdir -p $(FHRE_SIM_RESOURCE)
	$(RESOURCE_BUDGET_CHECK) $(FHRE_RESOURCE) $(FHRE_RESOURCE_MANIFEST) $(FHRE_SIM_RESOURCE)
else
context::
	@echo "Checking FHRE Rust core..."
	@echo "Target: $(RUST_TARGET)"
	cargo build --manifest-path $(RUST_CARGO_PATH)/Cargo.toml --target $(RUST_TARGET) --no-default-features
endif

clean::
	cargo clean --manifest-path $(RUST_CARGO_PATH)/Cargo.toml

include $(APPDIR)/Application.mk
