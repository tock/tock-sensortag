//! RF Core
//!
//! Provides communication with the core module of the radio.
//!     See page 1586 in the datasheet for more details.

use kernel::common::VolatileCell;

use cc26xx::prcm;
use cc26xx::osc;

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

bitfield!{
    #[derive(Copy, Clone)]
    struct RfcTrigger(u8);
    impl Debug;
    pub _trigger_type, _set_trigger_type  : 3, 0;
    pub _enable_cmd, _set_enable_cmd      : 4;
    pub _trigger_no, _set_trigger_no      : 6, 5;
    pub _past_trigger, _set_past_trigger  : 7;
}

bitfield!{
    #[derive(Copy, Clone)]
    struct RfcCondition(u8);
    impl Debug;
    pub _rule, set_rule : 3, 0;
    pub _skip, _set_skip : 7, 4;
}

pub trait RfcCommand {
    fn command(&self) -> u32;
}

#[repr(C)]
pub struct RfcRadioOperation {
    command_no: u16,
    status: u16,
    next_op: *const RfcRadioOperation,
    start_time: u32,
    start_trigger: u8,
    condition: RfcCondition,
}

impl RfcRadioOperation {
    pub fn new(command: u16, status: u16) -> RfcRadioOperation {
        RfcRadioOperation {
            command_no: command,
            status: status,
            next_op: 0 as *const RfcRadioOperation,
            start_time: 0,
            start_trigger: 0,
            condition: {
                let mut cond = RfcCondition(0);
                cond.set_rule(0x01); // COND_NEVER
                cond
            },
        }
    }
}

impl RfcCommand for RfcRadioOperation {
    fn command(&self) -> u32 {
        /*165:#define CMDR_DIR_CMD(cmdId) (((cmdId) << 16) | 1)
        168:#define CMDR_DIR_CMD_1BYTE(cmdId, par) (((cmdId) << 16) | ((par) << 8) | 1)
        171:#define CMDR_DIR_CMD_2BYTE(cmdId, par) (((cmdId) << 16) | ((par) & 0xFFFC) | 1)*/

        // Default to 2BYTE
        let cmd = self.command_no as u32;
        let par = self.status as u32;
        (cmd << 16) | (par & 0xFFFC) | 1
    }
}

#[repr(C)]
struct RfcCommandRadioSetup {
    command_no: u16,
    status: u16,
    p_nextop: u32,
    ratmr: u32,
    start_trigger: u8,
    condition: RfcCondition,
    mode: u8,
    lo_divider: u8,
    config: RfcSetupConfig,
    tx_power: u16,
    reg_override: u32, //*const [u32],
}

impl RfcCommand for RfcCommandRadioSetup {
    fn command(&self) -> u32 {
        (self as *const RfcCommandRadioSetup) as u32
    }
}

bitfield!{
    #[derive(Copy, Clone)]
    struct RfcSetupConfig(u16);
    impl Debug;
    pub _frontend_mode, set_frontend_mode: 2, 0;
    pub _bias_mode, set_bias_mode: 3;
    pub _analog_cfg_mode, _set_analog_config_mode: 10, 4;
    pub _no_fs_powerup, _set_no_fs_powerup: 11;
}

const BLE_OVERRIDES: [u32; 7] = [
    0x00364038, /* Synth: Set RTRIM (POTAILRESTRIM) to 6 */
    0x000784A3, /* Synth: Set FREF = 3.43 MHz (24 MHz / 7) */
    0xA47E0583, /* Synth: Set loop bandwidth after lock to 80 kHz (K2) */
    0xEAE00603, /* Synth: Set loop bandwidth after lock to 80 kHz (K3, LSB) */
    0x00010623, /* Synth: Set loop bandwidth after lock to 80 kHz (K3, MSB) */
    0x00456088, /* Adjust AGC reference level */
    0xFFFFFFFF, /* End of override list */
];

const RFC_DBELL_BASE: *mut RfcBellRegisters = 0x4004_1000 as *mut RfcBellRegisters;
const RFC_PWR_BASE: *mut VolatileCell<u32> = 0x4004_0000 as *mut VolatileCell<u32>;

/*
    Used to enable certain clocks through the status
    command to the RFC.
*/
const RFC_PWR_CLK_MDMRAM: u16 = 0x10;
const RFC_PWR_CLK_RFERAM: u16 = 0x40;

/*
    RFC Immediate commands
*/
const RFC_CMD0: u16 = 0x607;
const RFC_PING: u16 = 0x0406;
const RFC_START_RAT_TIMER: u16 = 0x080A;

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

pub struct RFCore {
    bell_regs: *mut RfcBellRegisters,
    pwr_ctl: *mut VolatileCell<u32>,
}

impl RFCore {
    pub const fn new() -> RFCore {
        RFCore {
            bell_regs: RFC_DBELL_BASE,
            pwr_ctl: RFC_PWR_BASE,
        }
    }

    pub fn enable(&self) {
        osc::OSCILLATOR_CONTROL.switch_to_hf_xosc();

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
        /*bell_regs.command.set(RFC_CMD0 << 16
            | (RFC_PWR_CLK_MDMRAM | RFC_PWR_CLK_RFERAM) & 0xFFFC | 1);*/
        //self.send_command(RfcRadioOperation::new(RFC_CMD0, RFC_PWR_CLK_MDMRAM | RFC_PWR_CLK_RFERAM));

        // Send a ping command to verify that the core is ready and alive
        let status = self.send_command(&RfcRadioOperation::new(RFC_PING, 0));
        match status {
            RfcResult::Error(errno) => panic!("Tried to enable RFC but an error occurred, status: {:x}", errno),
            _ => debug_verbose!("Radio successfully enabled\r")
        }
    }

    #[inline(never)]
    #[no_mangle]
    pub fn setup(&self) {
        let setup_cmd: RfcCommandRadioSetup = RfcCommandRadioSetup {
            command_no: 0x0802,
            status: 0,
            p_nextop: 0,
            ratmr: 0,
            start_trigger: 0,
            condition: {
                let mut cond = RfcCondition(0);
                cond.set_rule(0x01); // COND_NEVER
                cond
            },
            mode: 0,
            lo_divider: 0,
            config: {
                let mut cfg = RfcSetupConfig(0);
                cfg.set_frontend_mode(0x01);
                cfg.set_bias_mode(true); // External - might need to change
                cfg
            },
            tx_power: 0x9330,
            reg_override: 0, //&BLE_OVERRIDES as *const [u32],
        };

        match self.send_command(&setup_cmd) {
            RfcResult::Error(status) => panic!("Could not send setup to radio, status=0x{:x}\r", status),
            _ => debug_verbose!("Sent setup to the radio\r")
        }

        // Wait for the cmd to be done
        debug_verbose!("Waiting for it to complete.\r");
        let mut timeout = 0;
        loop {
            timeout += 1;

            if (setup_cmd.status & 0x0C00) == 0x0400 {
                debug_verbose!("Got a CMD_DONE!!!!\r");
                break;
            }

            if timeout > 0x2FFFFFF {
                panic!("Timeout cmd.status=0x{:x}\r", setup_cmd.status);
            }
        }
    }

    pub fn send_command<C: RfcCommand>(&self, cmd: &C) -> RfcResult {
        let command = cmd.command();

        // Check if the radio is accessible or not
        if !prcm::Power::is_enabled(prcm::PowerDomain::RFC) {
            debug_verbose!("RFC power domain is off.\r");
            return RfcResult::Error(0);
        }

        let bell_regs: &RfcBellRegisters = unsafe { &*self.bell_regs };

        while bell_regs.command.get() != 0 { }

        // Set the command
        bell_regs.command.set(command);

        debug_verbose!("Radio command pending: 0x{:x}\r", command);

        let mut timeout = 0;
        loop {
            let status = bell_regs.command_status.get();
            timeout = timeout + 1;

            if (status & 0xFF) == 0x01 {
                debug_verbose!("Status=0x{:x}\r", status);
                break;
            }

            if timeout > 0x2FFFFFF {
                debug_verbose!("TIMED OUT WITH STATUS 0x{:x}\r", status);
                return RfcResult::Error(status)
            }
        }

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
