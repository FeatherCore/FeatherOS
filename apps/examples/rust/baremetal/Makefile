############################################################################
# apps/examples/rust/baremetal/Makefile
#
# SPDX-License-Identifier: Apache-2.0
#
# Licensed to the Apache Software Foundation (ASF) under one or more
# contributor license agreements.  See the NOTICE file distributed with
# this work for additional information regarding copyright ownership.  The
# ASF licenses this file to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance with the
# License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.  See the
# License for the specific language governing permissions and limitations
# under the License.
#
############################################################################

include $(APPDIR)/Make.defs

# Hello, Rust! built-in application info

PROGNAME  = $(CONFIG_EXAMPLES_HELLO_RUST_PROGNAME)
PRIORITY  = $(CONFIG_EXAMPLES_HELLO_RUST_PRIORITY)
STACKSIZE = $(CONFIG_EXAMPLES_HELLO_RUST_STACKSIZE)
MODULE    = $(CONFIG_EXAMPLES_HELLO_RUST)

# Rust build configuration
RUST_CRATE_NAME := hello_rust
RUST_CARGO_PATH := $(APPDIR)/examples/rust/baremetal

# Get target triple from NuttX configuration
# This supports multiple architectures: thumbv6m, thumbv7m, thumbv8m, riscv32, riscv64, x86_64, etc.
RUST_TARGET := $(strip $(call RUST_TARGET_TRIPLE))

# Generate .cargo/config.toml dynamically based on target
CARGO_CONFIG := $(RUST_CARGO_PATH)/.cargo/config.toml

context:: $(CARGO_CONFIG)
	@echo "Building Rust code with cargo..."
	@echo "Target: $(RUST_TARGET)"
	NUTTX_INCLUDE_DIR=$(TOPDIR)/include:$(TOPDIR)/include/arch \
	RUST_TARGET=$(RUST_TARGET) \
	cargo build \
		--manifest-path $(RUST_CARGO_PATH)/Cargo.toml \
		--target $(RUST_TARGET)

# Dynamically generate Cargo config based on NuttX target
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
