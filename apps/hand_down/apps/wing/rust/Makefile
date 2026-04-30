############################################################################
# apps/wing/rust/Makefile
############################################################################

include $(APPDIR)/Make.defs

PROGNAME  = $(CONFIG_WING_RUST_PROGNAME)
PRIORITY  = $(CONFIG_WING_RUST_PRIORITY)
STACKSIZE = $(CONFIG_WING_RUST_STACKSIZE)
MODULE    = $(CONFIG_WING_RUST)

RUST_CARGO_PATH := $(APPDIR)/wing/rust

ifeq ($(CONFIG_ARCH_SIM),y)
RUST_NATIVE := y
endif

ifeq ($(RUST_NATIVE),y)
context::
	@echo "Building clean Wing with cargo (native)..."
	WING_DISPLAY_WIDTH=$(CONFIG_SIM_FBWIDTH) \
	WING_DISPLAY_HEIGHT=$(CONFIG_SIM_FBHEIGHT) \
	cargo build \
		--manifest-path $(RUST_CARGO_PATH)/Cargo.toml \
		--no-default-features
else
RUST_TARGET := $(strip $(call RUST_TARGET_TRIPLE))
CARGO_CONFIG := $(RUST_CARGO_PATH)/.cargo/config.toml

context:: $(CARGO_CONFIG)
	@echo "Building clean Wing with cargo..."
	@echo "Target: $(RUST_TARGET)"
	RUST_TARGET=$(RUST_TARGET) \
	WING_DISPLAY_WIDTH=$(CONFIG_SIM_FBWIDTH) \
	WING_DISPLAY_HEIGHT=$(CONFIG_SIM_FBHEIGHT) \
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
	@echo '  "-C", "link-arg=--undefined=rust_eh_personality",' >> $@
	@echo ']' >> $@
endif

clean::
	cargo clean --manifest-path $(RUST_CARGO_PATH)/Cargo.toml

distclean::
	$(call DELFILE, $(CARGO_CONFIG))

include $(APPDIR)/Application.mk
