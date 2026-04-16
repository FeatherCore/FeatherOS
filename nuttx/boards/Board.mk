############################################################################
# boards/Board.mk
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

# Detect genromfs - prefer system genromfs if available
GENROMFS := $(shell which genromfs 2>/dev/null || echo "$(TOPDIR)/tools/genromfs")

# Check if using system genromfs (supports -f -d flags) or NuttX genromfs (positional args)
# System genromfs outputs "Usage: genromfs -f <output> -d <source_dir>" when called without args
GENROMFS_HELP := $(shell $(GENROMFS) 2>&1 || true)
GENROMFS_IS_SYSTEM := $(findstring -f,$(GENROMFS_HELP))

ifneq ($(RCSRCS)$(RCRAWS),)
ETCDIR := etctmp
ETCSRC := $(ETCDIR:%=%.c)

CSRCS += $(ETCSRC)

RCOBJS = $(RCSRCS:%=$(ETCDIR)$(DELIM)%)

$(RCOBJS): $(ETCDIR)$(DELIM)%: %
	$(Q) mkdir -p $(dir $@)
	$(call PREPROCESS, $<, $@)

$(ETCSRC): $(foreach raw,$(RCRAWS), $(if $(wildcard $(BOARD_DIR)$(DELIM)src$(DELIM)$(raw)), $(BOARD_DIR)$(DELIM)src$(DELIM)$(raw), $(if $(wildcard $(BOARD_COMMON_DIR)$(DELIM)$(raw)), $(BOARD_COMMON_DIR)$(DELIM)$(raw), $(BOARD_DIR)$(DELIM)src$(DELIM)$(raw)))) $(RCOBJS)
	$(foreach raw, $(RCRAWS), \
  	  $(shell rm -rf $(ETCDIR)$(DELIM)$(raw)) \
  	  $(shell mkdir -p $(dir $(ETCDIR)$(DELIM)$(raw))) \
  	  $(shell cp -rfp $(if $(wildcard $(BOARD_DIR)$(DELIM)src$(DELIM)$(raw)), $(BOARD_DIR)$(DELIM)src$(DELIM)$(raw), $(if $(wildcard $(BOARD_COMMON_DIR)$(DELIM)$(raw)), $(BOARD_COMMON_DIR)$(DELIM)$(raw), $(BOARD_DIR)$(DELIM)src$(DELIM)$(raw))) $(ETCDIR)$(DELIM)$(raw)))
	$(Q) if $(GENROMFS) --help 2>&1 | grep -q '\-f.*\-d'; then \
		$(GENROMFS) -f romfs.img -d $(ETCDIR)$(DELIM)$(CONFIG_ETC_ROMFSMOUNTPT); \
	else \
		$(GENROMFS) $(ETCDIR)$(DELIM)$(CONFIG_ETC_ROMFSMOUNTPT) romfs.img; \
	fi
	$(Q) echo "#include <nuttx/compiler.h>" > $@
	$(Q) xxd -i romfs.img | sed -e "s/^unsigned char/const unsigned char aligned_data(4)/g" >> $@
	$(Q) rm romfs.img
endif

ifneq ($(ZDSVERSION),)
AOBJS = $(ASRCS:.S=$(OBJEXT))
else
AOBJS = $(ASRCS:$(ASMEXT)=$(OBJEXT))
endif
COBJS = $(CSRCS:.c=$(OBJEXT))
CXXOBJS = $(CXXSRCS:.cxx=$(OBJEXT))

SRCS = $(ASRCS) $(CSRCS)
OBJS = $(AOBJS) $(COBJS)

SCHEDSRCDIR = $(TOPDIR)$(DELIM)sched
ARCHSRCDIR = $(TOPDIR)$(DELIM)arch$(DELIM)$(CONFIG_ARCH)$(DELIM)src
ifneq ($(CONFIG_ARCH_FAMILY),)
  ARCH_FAMILY = $(patsubst "%",%,$(CONFIG_ARCH_FAMILY))
endif

CFLAGS += ${INCDIR_PREFIX}"$(SCHEDSRCDIR)"
CFLAGS += ${INCDIR_PREFIX}"$(ARCHSRCDIR)$(DELIM)chip"
ifneq ($(CONFIG_ARCH_SIM),y)
  CFLAGS += ${INCDIR_PREFIX}"$(ARCHSRCDIR)$(DELIM)common"
endif
ifneq ($(ARCH_FAMILY),)
  CFLAGS += ${INCDIR_PREFIX}"$(ARCHSRCDIR)$(DELIM)$(ARCH_FAMILY)"
endif

all: libboard$(LIBEXT)

ifneq ($(ZDSVERSION),)
$(ASRCS) $(HEAD_ASRC): %$(ASMEXT): %.S
	$(Q) $(CPP) $(CPPFLAGS) $(call CONVERT_PATH,$<) -o $@.tmp
	$(Q) cat $@.tmp | sed -e "s/^#/;/g" > $@
	$(Q) rm $@.tmp
endif

$(AOBJS): %$(OBJEXT): %$(ASMEXT)
	$(call ASSEMBLE, $<, $@)

$(COBJS) $(LINKOBJS): %$(OBJEXT): %.c
	$(call COMPILE, $<, $@)

$(CXXOBJS) $(LINKOBJS): %$(OBJEXT): %.cxx
	$(call COMPILEXX, $<, $@)

libboard$(LIBEXT): $(OBJS) $(CXXOBJS)
	$(call ARCHIVE, $@, $(OBJS) $(CXXOBJS))

.depend: Makefile $(SRCS) $(CXXSRCS) $(RCSRCS) $(TOPDIR)$(DELIM).config
ifneq ($(ZDSVERSION),)
	$(Q) $(MKDEP) $(DEPPATH) "$(CC)" -- $(CFLAGS) -- $(SRCS) >Make.dep
else
	$(Q) $(MKDEP) $(DEPPATH) $(CC) -- $(CFLAGS) -- $(SRCS) >Make.dep
endif
ifneq ($(CXXSRCS),)
	$(Q) $(MKDEP) $(DEPPATH) "$(CXX)" -- $(CXXFLAGS) -- $(CXXSRCS) >>Make.dep
endif
ifneq ($(RCSRCS),)
	$(Q) $(MKDEP) $(DEPPATH) "$(CPP)" --obj-path . -- $(CPPFLAGS) -- $(RCSRCS) >>Make.dep
endif
	$(Q) touch $@

depend: .depend

context::

clean::
	$(call DELFILE, libboard$(LIBEXT))
	$(call DELFILE, $(ETCSRC))
	$(call DELDIR, $(ETCDIR))
	$(call CLEAN)

distclean:: clean
	$(call DELFILE, Make.dep)
	$(call DELFILE, .depend)

-include Make.dep
