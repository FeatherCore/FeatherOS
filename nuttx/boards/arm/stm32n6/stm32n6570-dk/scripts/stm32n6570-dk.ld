/****************************************************************************
 * boards/arm/stm32n6/stm32n6570-dk/scripts/stm32n6570-dk.ld
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Linker script for STM32N6570-DK Cortex-M55 AXISRAM execution.
 *
 ****************************************************************************/

OUTPUT_FORMAT("elf32-littlearm", "elf32-bigarm", "elf32-littlearm")
OUTPUT_ARCH(arm)
ENTRY(__start)

MEMORY
{
  ram (rwx)  : ORIGIN = 0x34000000, LENGTH = 1024K
  xspi (rx)  : ORIGIN = 0x70000000, LENGTH = 64M
  psram (rw) : ORIGIN = 0x90000000, LENGTH = 64M
}

_stack_size = DEFINED(stack_size) ? stack_size : 2048;
_idle_stack_size = DEFINED(idle_stack_size) ? idle_stack_size : 2048;

SECTIONS
{
  .text :
  {
    _stext = ABSOLUTE(.);
    *(.vectors)
    *(.text .text.*)
    *(.fixup)
    *(.gnu.warning)
    *(.rodata .rodata.*)
    *(.gnu.linkonce.t.*)
    *(.glue_7)
    *(.glue_7t)
    *(.got)
    *(.gcc_except_table)
    *(.gnu.linkonce.r.*)
    . = ALIGN(4);
    _etext = ABSOLUTE(.);
  } > ram

  .init_section :
  {
    _sinit = ABSOLUTE(.);
    KEEP(*(SORT_BY_INIT_PRIORITY(.init_array.*)))
    KEEP(*(.init_array))
    _einit = ABSOLUTE(.);
  } > ram

  .ARM.extab :
  {
    *(.ARM.extab* .gnu.linkonce.armextab.*)
  } > ram

  .ARM.exidx :
  {
    *(.ARM.exidx* .gnu.linkonce.armexidx.*)
  } > ram

  _eronly = ABSOLUTE(.);

  .data :
  {
    _sdata = ABSOLUTE(.);
    *(.data .data.*)
    *(.gnu.linkonce.d.*)
    CONSTRUCTORS
    . = ALIGN(4);
    _edata = ABSOLUTE(.);
  } > ram

  _data_loadaddr = LOADADDR(.data);

  .bss :
  {
    _sbss = ABSOLUTE(.);
    *(.bss .bss.*)
    *(.gnu.linkonce.b.*)
    *(COMMON)
    . = ALIGN(4);
    _ebss = ABSOLUTE(.);
  } > ram

  .noinit :
  {
    _snoinit = ABSOLUTE(.);
    *(.noinit*)
    . = ALIGN(4);
    _enoinit = ABSOLUTE(.);
  } > ram

  .ramfunc :
  {
    _sramfuncs = ABSOLUTE(.);
    *(.ramfunc  .ramfunc.*)
    . = ALIGN(4);
    _eramfuncs = ABSOLUTE(.);
  } > ram

  _framfuncs = LOADADDR(.ramfunc);

  .heap :
  {
    _sheap = ABSOLUTE(.);
    . = ORIGIN(ram) + LENGTH(ram) - _stack_size - _idle_stack_size;
    _eheap = ABSOLUTE(.);
  } > ram

  .stack :
  {
    _estack = ABSOLUTE(.);
    . = . + _stack_size;
    _sstack = ABSOLUTE(.);
  } > ram

  _START_BSS = _sbss;
  _END_BSS = _ebss;
  _START_DATA = _sdata;
  _END_DATA = _edata;
  _DATA_INIT = _data_loadaddr;

  /DISCARD/ :
  {
    *(.comment)
    *(.note*)
  }
}
