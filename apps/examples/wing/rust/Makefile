############################################################################
# apps/examples/wing/rust/Makefile
############################################################################

include $(APPDIR)/Make.defs

PROGNAME  = $(CONFIG_EXAMPLES_WING_RUST_PROGNAME)
PRIORITY  = $(CONFIG_EXAMPLES_WING_RUST_PRIORITY)
STACKSIZE = $(CONFIG_EXAMPLES_WING_RUST_STACKSIZE)
MODULE    = $(CONFIG_EXAMPLES_WING_RUST)

RUST_CRATE_NAME := wing_rs_demo
RUST_CARGO_PATH := $(APPDIR)/examples/wing/rust

# Check if we're building for NuttX simulator (native compilation)
ifeq ($(CONFIG_ARCH_SIM),y)
RUST_NATIVE := y
endif

# Library paths differ between SIM and embedded builds
ifeq ($(RUST_NATIVE),y)
FHRE_LIB := $(APPDIR)/fhre/rust/target/$(if $(CONFIG_DEBUG_FULLOPT),release,debug)/libfhre.a
WING_LIB := $(APPDIR)/wing/rust/target/$(if $(CONFIG_DEBUG_FULLOPT),release,debug)/libwing.a
else
RUST_TARGET := $(strip $(call RUST_TARGET_TRIPLE))
FHRE_LIB := $(APPDIR)/fhre/rust/target/$(RUST_TARGET)/$(if $(CONFIG_DEBUG_FULLOPT),release,debug)/libfhre.a
WING_LIB := $(APPDIR)/wing/rust/target/$(RUST_TARGET)/$(if $(CONFIG_DEBUG_FULLOPT),release,debug)/libwing.a
endif

# For SIM (native compilation), we don't need --target flag
ifeq ($(RUST_NATIVE),y)
context:: $(FHRE_LIB) $(WING_LIB)
	@echo "Building Wing Rust demo with cargo (native)..."
	cargo build \
		--manifest-path $(RUST_CARGO_PATH)/Cargo.toml \
		--no-default-features
else
RUST_TARGET := $(strip $(call RUST_TARGET_TRIPLE))

# Generate .cargo/config.toml dynamically based on target
CARGO_CONFIG := $(RUST_CARGO_PATH)/.cargo/config.toml

context:: $(CARGO_CONFIG) $(FHRE_LIB) $(WING_LIB)
	@echo "Building Wing Rust demo with cargo..."
	@echo "Target: $(RUST_TARGET)"
	NUTTX_INCLUDE_DIR=$(TOPDIR)/include:$(TOPDIR)/include/arch \
	RUST_TARGET=$(RUST_TARGET) \
	cargo build \
		--manifest-path $(RUST_CARGO_PATH)/Cargo.toml \
		--target $(RUST_TARGET) \
		--no-default-features

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
	@echo ']' >> $@
endif

$(FHRE_LIB):
	@echo "Building FHRE library..."
	$(MAKE) -C $(APPDIR)/fhre/rust context

$(WING_LIB):
	@echo "Building Wing library..."
	$(MAKE) -C $(APPDIR)/wing/rust context

clean::
	cargo clean --manifest-path $(RUST_CARGO_PATH)/Cargo.toml

distclean::
	$(call DELFILE, $(CARGO_CONFIG))

include $(APPDIR)/Application.mk
