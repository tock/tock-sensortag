/*
 * This is the generic linker script for Tock. For most developers, it should
 * be sufficient to define {ROM/PROG/RAM}_{ORIGIN/LENGTH} (6 variables, the
 * start and length for each) and MPU_MIN_ALIGN (the minimum alignment
 * granularity supported by the MPU).
 *
 * --------------------------------------------------------------------------
 *
 * If you wish to create your own linker script from scratch, you must define
 * the following symbols:
 *
 * `_etext`, `_srelocate`, `_erelocate`
 *    The `_etext` symbol marks the end of data stored in flash that should
 *    stay in flash. `_srelocate` and `_erelocate` mark the address range in
 *    SRAM that mutable program data is copied to.
 *
 *    Tock will copy `_erelocate` - `_srelocate` bytes of data from the
 *    `_etext` pointer to the `_srelocate` pointer.
 *
 * `_szero`, `_ezero`
 *
 *    The `_szero` and `_ezero` symbols define the range of the BSS, SRAM that
 *    Tock will zero on boot.
 *
 * `_sapps`
 *
 *    The `_sapps` symbol marks the beginning of application memory in flash.
 */

ROM_ORIGIN   =   0x00000000;
ROM_LENGTH   =   0x00010000;
PROG_ORIGIN  =   0x00010000;
PROG_LENGTH  =   0x0000F000;
CCFG_ORIGIN  =   0x0001FFA8;
CCFG_LENGTH  =   0x00000058;
RAM_ORIGIN   =   0x20000000;
RAM_LENGTH   =   20K;

MPU_MIN_ALIGN = 4;

INCLUDE ../../tock/boards/kernel_layout.ld

