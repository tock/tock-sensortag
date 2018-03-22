# Makefile for building the tock kernel for the sensortag development kit

TOCK_ARCH=cortex-m3
TOCK_BOARD=sensortag
TARGET=thumbv7m-none-eabi
PLATFORM=sensortag

DSLITE=$(UNIFLASH_CLI_BASE)/ccs_base/DebugServer/bin/DSLite
PROG_BASE=0x10000

include ../../tock/boards/Makefile.common

apps/%/build/$(TOCK_ARCH)/$(TOCK_ARCH).bin:
	@make -C apps/$*

flash: guard-UNIFLASH_CLI_BASE target/$(TARGET)/release/sensortag
	@echo "Flashing target/$(TARGET)/release/sensortag"
	@$(DSLITE) flash -c jtag/cc2650.ccxml -e -f target/$(TARGET)/release/sensortag

flash-%: apps/%/build/$(TOCK_ARCH)/$(TOCK_ARCH).bin guard-DSLITE
	@$(DSLITE) flash -c jtag/cc2650.ccxml -e -f $<,$(PROG_BASE)

guard-%:
	@ if [ "${${*}}" = "" ]; then \
		echo "Environment variable $* not set, see readme for more details."; \
		exit 1; \
	fi
