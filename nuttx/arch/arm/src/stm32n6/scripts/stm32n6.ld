/****************************************************************************
 * Linker script for STM32N6 (Cortex-M55)
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 ****************************************************************************/

OUTPUT_ARCH(arm)
ENTRY(__start)

MEMORY
{
    flash (rx)  : ORIGIN = 0x08000000, LENGTH = 16M
    sram (rwx)  : ORIGIN = 0x34000000, LENGTH = 512K
    axisram (rwx) : ORIGIN = 0x34180000, LENGTH = 512K
}

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
        _etext = ABSOLUTE(.);
    } > flash

    .init_section :
    {
        _sinit = ABSOLUTE(.);
        KEEP(*(SORT_BY_INIT_PRIORITY(.init_array.*)))
        KEEP(*(.init_array))
        _einit = ABSOLUTE(.);
    } > flash

    .ARM.extab :
    {
        *(.ARM.extab* .gnu.linkonce.armextab.*)
    } > flash

    .ARM.exidx :
    {
        *(.ARM.exidx* .gnu.linkonce.armexidx.*)
    } > flash

    .data :
    {
        _sdata = ABSOLUTE(.);
        *(.data .data.*)
        *(.gnu.linkonce.d.*)
        CONSTRUCTORS
        _edata = ABSOLUTE(.);
    } > sram AT > flash

    _data_init = LOADADDR(.data);

    .bss :
    {
        _sbss = ABSOLUTE(.);
        *(.bss .bss.*)
        *(.gnu.linkonce.b.*)
        *(COMMON)
        _ebss = ABSOLUTE(.);
    } > sram

    .noinit :
    {
        _snoinit = ABSOLUTE(.);
        *(.noinit*)
        _enoinit = ABSOLUTE(.);
    } > sram

    .stack :
    {
        _sstack = ABSOLUTE(.);
        *(.stack)
        . = ALIGN(8);
        _estack = ABSOLUTE(.);
    } > sram

    .heap :
    {
        _sheap = ABSOLUTE(.);
        . = ORIGIN(sram) + LENGTH(sram) - ABSOLUTE(.);
        _eheap = ABSOLUTE(.);
    } > sram

    _START_BSS = _sbss;
    _END_BSS = _ebss;
    _START_DATA = _sdata;
    _END_DATA = _edata;
    _DATA_INIT = _data_init;

    /DISCARD/ :
    {
        *(.comment)
        *(.note*)
    }
}