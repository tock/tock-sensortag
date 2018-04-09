//! Power, Clock, and Reset Management (PRCM)
//!
//! For details see p.411 in the cc2650 technical reference manual.
//!
//! PRCM manages different power domains on the boards, specifically:
//!
//!     * RF Power domain
//!     * Serial Power domain
//!     * Peripheral Power domain
//!
//! It also manages the clocks attached to almost every peripheral, which needs to
//! be enabled before usage.

use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};

#[repr(C)]
struct PrcmRegisters {
    _reserved0: [ReadOnly<u8>; 0x28],

    // Write 1 in order to load settings
    pub clk_load_ctl: ReadWrite<u32, ClockLoad::Register>,

    pub rfc_clk_gate: ReadWrite<u32, ClockGate::Register>,

    _reserved1: [ReadOnly<u8>; 0xC],

    // TRNG, Crypto, and UDMA
    pub sec_dma_clk_run: ReadWrite<u32, SECDMAClockGate::Register>,
    pub sec_dma_clk_sleep: ReadWrite<u32, SECDMAClockGate::Register>,
    pub sec_dma_clk_deep_sleep: ReadWrite<u32, SECDMAClockGate::Register>,

    pub gpio_clk_gate_run: ReadWrite<u32, ClockGate::Register>,
    pub gpio_clk_gate_sleep: ReadWrite<u32, ClockGate::Register>,
    pub gpio_clk_gate_deep_sleep: ReadWrite<u32, ClockGate::Register>,

    pub gpt_clk_gate_run: ReadWrite<u32, ClockGate::Register>,
    pub gpt_clk_gate_sleep: ReadWrite<u32, ClockGate::Register>,
    pub gpt_clk_gate_deep_sleep: ReadWrite<u32, ClockGate::Register>,

    pub i2c_clk_gate_run: ReadWrite<u32, ClockGate::Register>,
    pub i2c_clk_gate_sleep: ReadWrite<u32, ClockGate::Register>,
    pub i2c_clk_gate_deep_sleep: ReadWrite<u32, ClockGate::Register>,

    pub uart_clk_gate_run: ReadWrite<u32, ClockGate::Register>,
    pub uart_clk_gate_sleep: ReadWrite<u32, ClockGate::Register>,
    pub uart_clk_gate_deep_sleep: ReadWrite<u32, ClockGate::Register>,

    _reserved4: [ReadOnly<u8>; 0xB4],

    // Power domain control 0
    pub pd_ctl0: ReadWrite<u32, PowerDomain0::Register>,
    pub pd_ctl0_rfc: WriteOnly<u32, PowerDomainSingle::Register>,
    pub pd_ctl0_serial: WriteOnly<u32, PowerDomainSingle::Register>,
    pub pd_ctl0_peripheral: WriteOnly<u32, PowerDomainSingle::Register>,

    _reserved5: [ReadOnly<u8>; 0x04],

    // Power domain status 0
    pub pd_stat0: ReadOnly<u32, PowerDomainStatus0::Register>,
    pub pd_stat0_rfc: ReadOnly<u32, PowerDomainSingle::Register>,
    pub pd_stat0_serial: ReadOnly<u32, PowerDomainSingle::Register>,
    pub pd_stat0_periph: ReadOnly<u32, PowerDomainSingle::Register>,

    _reserved7: [ReadOnly<u8>; 0x2C],

    pub pd_ctl1: ReadWrite<u32, PowerDomain1::Register>,

    _reserved8: [ReadOnly<u8>; 0x14],

    pub pd_stat1: ReadOnly<u32, PowerDomainStatus1::Register>,

    _reserved9: [ReadOnly<u8>; 0x38],

    pub rfc_mode_sel: ReadWrite<u32>,
}

register_bitfields![
    u32,
    ClockLoad [
        LOAD_DONE   OFFSET(1) NUMBITS(1) [],
        LOAD        OFFSET(0) NUMBITS(1) []
    ],
    SECDMAClockGate [
        DMA_CLK_EN      OFFSET(8) NUMBITS(1) [],
        TRNG_CLK_EN     OFFSET(1) NUMBITS(1) [],
        CRYPTO_CLK_EN   OFFSET(0) NUMBITS(1) []
    ],
    ClockGate [
        CLK_EN  OFFSET(0) NUMBITS(1) []
    ],
    PowerDomain0 [
        PERIPH_ON   OFFSET(2) NUMBITS(1) [],
        SERIAL_ON   OFFSET(1) NUMBITS(1) [],
        RFC_ON      OFFSET(0) NUMBITS(1) []
    ],
    PowerDomain1 [
        RFC_ON      OFFSET(2) NUMBITS(1) []
    ],
    PowerDomainSingle [
        ON  OFFSET(0) NUMBITS(1) []
    ],
    PowerDomainStatus0 [
        PERIPH_ON   OFFSET(2) NUMBITS(1) [],
        SERIAL_ON   OFFSET(1) NUMBITS(1) [],
        RFC_ON      OFFSET(0) NUMBITS(1) []
    ],
    PowerDomainStatus1 [
        RFC_ON      OFFSET(2) NUMBITS(1) []
    ]
];

const PRCM_BASE: *mut PrcmRegisters = 0x4008_2000 as *mut PrcmRegisters;

/*
    In order to save changes to the PRCM, we need to
    trigger
*/
fn prcm_commit() {
    let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };
    regs.clk_load_ctl.write(ClockLoad::LOAD::SET);
    // Wait for the settings to take effect
    while !regs.clk_load_ctl.is_set(ClockLoad::LOAD_DONE) {}
}

pub enum PowerDomain {
    // Note: when RFC is to be enabled, you are required to use both
    // power domains (i.e enable RFC on both PowerDomain0 and PowerDomain1)
    RFC = 0,
    Serial = 1,
    Peripherals = 2,
    VIMS = 3,
}

impl From<u32> for PowerDomain {
    fn from(n: u32) -> Self {
        match n {
            0 => PowerDomain::RFC,
            1 => PowerDomain::Serial,
            2 => PowerDomain::Peripherals,
            3 => PowerDomain::VIMS,
            _ => unimplemented!()
        }
    }
}

pub struct Power(());

impl Power {
    pub fn enable_domain(domain: PowerDomain) {
        let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };

        match domain {
            PowerDomain::Peripherals => {
                regs.pd_ctl0.modify(PowerDomain0::PERIPH_ON::SET);
            }
            PowerDomain::Serial => {
                regs.pd_ctl0.modify(PowerDomain0::SERIAL_ON::SET);
            }
            PowerDomain::RFC => {
                regs.pd_ctl0.modify(PowerDomain0::RFC_ON::SET);
                regs.pd_ctl1.modify(PowerDomain1::RFC_ON::SET);
            }
            _ => {
                panic!("Tried to turn on a power domain not yet specified!");
            }
        }
    }

    pub fn disable_domain(domain: PowerDomain) {
        let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };

        match domain {
            PowerDomain::Peripherals => {
                regs.pd_ctl0.modify(PowerDomain0::PERIPH_ON::CLEAR);
            }
            PowerDomain::Serial => {
                regs.pd_ctl0.modify(PowerDomain0::SERIAL_ON::CLEAR);
            },
            PowerDomain::RFC => {
                regs.pd_ctl0.modify(PowerDomain0::RFC_ON::CLEAR);
                regs.pd_ctl1.modify(PowerDomain1::RFC_ON::CLEAR);
            },
            _ => {
                panic!("Tried to turn on a power domain not yet specified!");
            }
        }
    }

    pub fn is_enabled(domain: PowerDomain) -> bool {
        let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };
        match domain {
            PowerDomain::Peripherals => regs.pd_stat0_periph.is_set(PowerDomainSingle::ON),
            PowerDomain::Serial => regs.pd_stat0_serial.is_set(PowerDomainSingle::ON),
            PowerDomain::RFC => {
                regs.pd_stat1.is_set(PowerDomainStatus1::RFC_ON)
                    && regs.pd_stat0.is_set(PowerDomainStatus0::RFC_ON)
            }
            _ => false,
        }
    }
}

pub struct Clock(());

impl Clock {
    pub fn enable_gpio() {
        let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };
        regs.gpio_clk_gate_run.write(ClockGate::CLK_EN::SET);
        regs.gpio_clk_gate_sleep.write(ClockGate::CLK_EN::SET);
        regs.gpio_clk_gate_deep_sleep.write(ClockGate::CLK_EN::SET);

        prcm_commit();
    }

    pub fn enable_uart_run() {
        let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };
        regs.uart_clk_gate_run.write(ClockGate::CLK_EN::SET);
        regs.uart_clk_gate_sleep.write(ClockGate::CLK_EN::SET);
        regs.uart_clk_gate_deep_sleep.write(ClockGate::CLK_EN::SET);

        prcm_commit();
    }

    pub fn enable_trng() {
        let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };
        regs.sec_dma_clk_run
            .write(SECDMAClockGate::TRNG_CLK_EN::SET);
        regs.sec_dma_clk_sleep
            .write(SECDMAClockGate::TRNG_CLK_EN::SET);
        regs.sec_dma_clk_deep_sleep
            .write(SECDMAClockGate::TRNG_CLK_EN::SET);

        prcm_commit();
    }

    pub fn enable_rfc() {
        let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };
        regs.rfc_clk_gate.write(ClockGate::CLK_EN::SET);

        prcm_commit();
    }

    pub fn enable_i2c() {
        let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };
        regs.i2c_clk_gate_run.write(ClockGate::CLK_EN::SET);
        regs.i2c_clk_gate_sleep.write(ClockGate::CLK_EN::SET);
        regs.i2c_clk_gate_deep_sleep.write(ClockGate::CLK_EN::SET);

        prcm_commit();
    }

    pub fn enable_gpt() {
        let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };
        regs.gpt_clk_gate_run.write(ClockGate::CLK_EN::SET);
        regs.gpt_clk_gate_sleep.write(ClockGate::CLK_EN::SET);
        regs.gpt_clk_gate_deep_sleep.write(ClockGate::CLK_EN::SET);

        prcm_commit();
    }

    pub fn i2c_run_clk_enabled() -> bool {
        let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };
        regs.i2c_clk_gate_run.is_set(ClockGate::CLK_EN)
    }
}

pub fn rf_mode_sel(mode: u32) {
    let regs: &PrcmRegisters = unsafe { &*PRCM_BASE };
    regs.rfc_mode_sel.set(mode);
}
