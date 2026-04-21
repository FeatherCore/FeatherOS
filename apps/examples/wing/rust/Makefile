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
RUST_TARGET := $(strip $(call RUST_TARGET_TRIPLE))
CARGO_CONFIG := $(RUST_CARGO_PATH)/.cargo/config.toml
FHRE_LIB := $(APPDIR)/fhre/rust/target/$(RUST_TARGET)/$(if $(CONFIG_DEBUG_FULLOPT),release,debug)/libfhre.a

context:: $(CARGO_CONFIG) $(FHRE_LIB)
	@echo "Building Wing Rust demo with cargo..."
	@echo "Target: $(RUST_TARGET)"
	NUTTX_INCLUDE_DIR=$(TOPDIR)/include:$(TOPDIR)/include/arch \
	RUST_TARGET=$(RUST_TARGET) \
	cargo build \
		--manifest-path $(RUST_CARGO_PATH)/Cargo.toml \
		--target $(RUST_TARGET) \
		--no-default-features

$(FHRE_LIB):
	@echo "Building FHRE library..."
	$(MAKE) -C $(APPDIR)/fhre/rust context

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

clean::
	cargo clean --manifest-path $(RUST_CARGO_PATH)/Cargo.toml

distclean::
	$(call DELFILE, $(CARGO_CONFIG))

include $(APPDIR)/Application.mk
