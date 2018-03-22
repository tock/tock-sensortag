# Makefile for building the tock kernel for the sensortag development kit

TOCK_ARCH=cortex-m3
TOCK_BOARD=sensortag
TARGET=thumbv7m-none-eabi
PLATFORM=sensortag

PROG_BASE=0x10000

include ../../tock/boards/Makefile.common

flash: target/$(TARGET)/release/sensortag
	$(call check_defined, UNIFLASH_CLI_BASE, See launchxl README for details)
	@echo "Flashing target/$(TARGET)/release/sensortag"
	@$(UNIFLASH_CLI_BASE)/ccs_base/DebugServer/bin/DSLite flash -c jtag/cc2650.ccxml -e -f target/$(TARGET)/release/sensortag
