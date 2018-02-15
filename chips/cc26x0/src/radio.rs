//! Radio module for CC2650 SensorTag
//!
//! The radio works by communication to an external Cortex-M0 Radio MCU which handles
//! all logic and transmissions. The radio mcu has the capability to wake-up the main chip
//! when traffic is detected, among other things.

use kernel::common::VolatileCell;

use cc26xx::prcm;

#[repr(C)]
pub struct RfcBellRegisters {
    command: VolatileCell<u32>,
    command_status: VolatileCell<u32>,
    _rf_hw_interrupt_flags: VolatileCell<u32>,
    _rf_hw_interrupt_enable: VolatileCell<u32>,

    rf_cpe_interrupt_flags: VolatileCell<u32>,
    rf_cpe_interrupt_enable: VolatileCell<u32>,
    _rf_cpe_interrupt_vector_sel: VolatileCell<u32>,
    rf_ack_interrupt_flag: VolatileCell<u32>,

    _sys_gpo_control: VolatileCell<u32>,
}

const RFC_DBELL_BASE: *mut RfcBellRegisters = 0x4004_1000 as *mut RfcBellRegisters;
const RFC_PWR_BASE: *mut VolatileCell<u32> = 0x4004_0000 as *mut VolatileCell<u32>;

pub const RFC: RFCore = RFCore::new();

pub struct RFCore {
    bell_regs: *mut RfcBellRegisters,
    pwr_ctl: *mut VolatileCell<u32>,
}

/*
    Used to enable certain clocks through the status
    command to the RFC.
*/
const RFC_PWR_CLK_MDMRAM: u32 = 0x10;
const RFC_PWR_CLK_RFERAM: u32 = 0x40;

/*
    Specific commands to the RFC
*/
const RFC_CMD0: u32 = 0x607;
const RFC_CMD_PING: u32 = 0x0406;

/*
    Power masks in order to enable certain clocks in the RFC
*/
const RFC_PWR_RFC: u32 = 0x01; // Main module
// Command and Packet Engine (CPE)
const RFC_PWR_CPE: u32 = 0x02;
const RFC_PWR_CPERAM: u32 = 0x04;
// Modem module
const RFC_PWR_MDM: u32 = 0x08;
const RFC_PWR_MDMRAM: u32 = 0x10;
// RF Engine (RFE)
const RFC_PWR_RFE: u32 = 0x20;
const RFC_PWR_RFERAM: u32 = 0x40;
// Radio Timer (RAT)
const RFC_PWR_RAT: u32 = 0x80;
// Packet Handling Accelerator (PHA)
const RFC_PWR_PHA: u32 = 0x100;
// Frequence Synthesizer Calibration Accelerator (FCSCA)
const RFC_PWR_FSCA: u32 = 0x200;

pub enum RfcResult {
    Ok,
    Error(u32),
}

pub enum RfcInterrupt {
    Cpe0,
    Cpe1,
    CmdAck,
    Hardware,
}

impl RFCore {
    const fn new() -> RFCore {
        RFCore {
            bell_regs: RFC_DBELL_BASE,
            pwr_ctl: RFC_PWR_BASE,
        }
    }

    pub fn enable(&self) {
        // Enable power & clock
        prcm::Power::enable_domain(prcm::PowerDomain::RFC);
        prcm::Clock::enable_rfc();

        // Wait for the power domain to be up
        while !prcm::Power::is_enabled(prcm::PowerDomain::RFC) { }

        let bell_regs: &RfcBellRegisters = unsafe { &*self.bell_regs };

        // Enable CPE
        bell_regs.rf_cpe_interrupt_enable.set(0);
        bell_regs.rf_cpe_interrupt_flags.set(0);

        // Setup clocks and allow CPE to boot
        let pwr_ctl: &VolatileCell<u32> = unsafe { &*self.pwr_ctl };
        pwr_ctl.set(
            RFC_PWR_RFC
            | RFC_PWR_CPE | RFC_PWR_CPERAM
            | RFC_PWR_FSCA
            | RFC_PWR_PHA
            | RFC_PWR_RAT
            | RFC_PWR_RFE | RFC_PWR_RFERAM
            | RFC_PWR_MDM | RFC_PWR_MDMRAM
        );

        // Turn on additional clocks
        bell_regs.rf_ack_interrupt_flag.set(0);

        // Wait for CMDR to be writeable
        //while bell_regs.command.get() != 0 { }
        bell_regs.command.set(RFC_CMD0 << 16
            | (RFC_PWR_CLK_MDMRAM | RFC_PWR_CLK_RFERAM) & 0xFFFC | 1);

        // Wait until CMD0 has been ACKed
        //while bell_regs.command_status.get() != 1 { }

        // Send a ping command to verify that the core is ready and alive
        let status = self.send_command(RFC_CMD_PING);
        match status {
            RfcResult::Error(errno) => panic!("Tried to enable RFC but an error occurred, status: {:x}", errno),
            _ => debug_verbose!("Radio successfully enabled\r")
        }
    }

    pub fn send_command(&self, command: u32) -> RfcResult {
        // Check if the radio is accessible or not
        if !prcm::Power::is_enabled(prcm::PowerDomain::RFC) {
            debug_verbose!("RFC power domain is off.\r");
            return RfcResult::Error(0);
        }

        let bell_regs: &RfcBellRegisters = unsafe { &*self.bell_regs };

        while bell_regs.command.get() != 0 { }

        // Set the command
        bell_regs.command.set(command);

        loop {
            let status = bell_regs.command_status.get();

            if (status & 0xFF) != 0 {
                break;
            }
        }

        debug_verbose!("Radio command sent: {:x}\r", command);
        RfcResult::Ok
    }

    pub fn handle_interrupt(&self, int: RfcInterrupt) {
        let bell_regs: &RfcBellRegisters = unsafe { &*self.bell_regs };
        match int {
            RfcInterrupt::CmdAck => {
                // Clear the interrupt
                bell_regs.rf_ack_interrupt_flag.set(0);

                debug_verbose!("CmdAck handled\r");
            }
            _ => ()
        }
    }
}
