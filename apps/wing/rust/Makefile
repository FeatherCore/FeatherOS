############################################################################
# apps/wing/rust/Makefile
############################################################################

include $(APPDIR)/Make.defs

MODULE = $(CONFIG_WING_RUST)

RUST_CRATE_NAME := wing
RUST_CARGO_PATH := $(APPDIR)/wing/rust

# Check if we're building for NuttX simulator (native compilation)
ifeq ($(CONFIG_ARCH_SIM),y)
RUST_NATIVE := y
endif

# FHRE library path differs between SIM and embedded builds
ifeq ($(RUST_NATIVE),y)
FHRE_LIB := $(APPDIR)/fhre/rust/target/$(if $(CONFIG_DEBUG_FULLOPT),release,debug)/libfhre.a
else
RUST_TARGET := $(strip $(call RUST_TARGET_TRIPLE))
FHRE_LIB := $(APPDIR)/fhre/rust/target/$(RUST_TARGET)/$(if $(CONFIG_DEBUG_FULLOPT),release,debug)/libfhre.a
endif

# For SIM (native compilation), we don't need --target flag
ifeq ($(RUST_NATIVE),y)
context:: $(FHRE_LIB)
	@echo "Building Wing library with cargo (native)..."
	cargo build \
		--manifest-path $(RUST_CARGO_PATH)/Cargo.toml
else
RUST_TARGET := $(strip $(call RUST_TARGET_TRIPLE))

# Generate .cargo/config.toml dynamically based on target
CARGO_CONFIG := $(RUST_CARGO_PATH)/.cargo/config.toml

context:: $(CARGO_CONFIG) $(FHRE_LIB)
	@echo "Building Wing library with cargo..."
	@echo "Target: $(RUST_TARGET)"
	NUTTX_INCLUDE_DIR=$(TOPDIR)/include:$(TOPDIR)/include/arch \
	RUST_TARGET=$(RUST_TARGET) \
	cargo build \
		--manifest-path $(RUST_CARGO_PATH)/Cargo.toml \
		--target $(RUST_TARGET)

$(CARGO_CONFIG):
	@echo "Generating Cargo config for target: $(RUST_TARGET)"
	@mkdir -p $(dir $@)
	@echo '[build]' > $@
	@echo 'target = "$(RUST_TARGET)"' >> $@
	@echo '' >> $@
	@echo '[target.$(RUST_TARGET)]' >> $@
	@echo 'rustflags = [' >> $@
	@echo '  "-C", "link-arg=-Tlink.x",' >> $@
	@echo '  "-C", "linker=rust-lld",' >> $@
	@echo '  "-C", "link-arg=--undefined=rust_eh_personality",' >> $@
	@echo ']' >> $@
endif

$(FHRE_LIB):
	@echo "Building FHRE library..."
	$(MAKE) -C $(APPDIR)/fhre/rust context

clean::
	cargo clean --manifest-path $(RUST_CARGO_PATH)/Cargo.toml

distclean::
	$(call DELFILE, $(CARGO_CONFIG))

include $(APPDIR)/Application.mk
