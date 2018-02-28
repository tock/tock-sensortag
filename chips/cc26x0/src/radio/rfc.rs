//! RF Core
//!
//! Provides communication with the core module of the radio.
//!     See page 1586 in the datasheet for more details.
//!
//! The radio is managed by an external Cortex-M0 running prioprietary code in order to manage
//! and set everything up. All stacks is implemented on this external MCU, and interaction
//! with it enables the radio to do all kinds of communication.
//!
//! In order to communicate, we send commands to the Cortex-M0 through something called
//! "Radio Doorbell", which has some fancy features attached to it.
//!
//! This abstraction allows us to perform complex radio instructions with a set of simple
//! packages and commands.
//!
//!

// RFC Commands are located at the bottom
use self::rfc_commands::*;
use cc26xx::prcm;
use cc26xx::rtc;
use kernel::common::VolatileCell;

/*
    RFC commands can be of two types:
        * Direct Commands
        * Radio operation / Immediate
*/
pub enum RfcCommandType {
    Direct,
    Immediate,
}

/*
    Trait to implement custom RFC commands.
*/
pub trait RfcCommand {
    fn command_id(&self) -> &u16;
    fn command_status(&self) -> &u16;
    fn command_type(&self) -> RfcCommandType;

    /*
        A direct command structure of CMDR:
        bit  31                    16               8               2    0
            ----------------------------------------------------------------
            | Command ID (16 bits) | Opt. param     | Opt. ext      | 0  1 |
            ----------------------------------------------------------------
    */
    fn direct_command(&self) -> u32 {
        let cmd = *self.command_id() as u32;
        let par = *self.command_status() as u32;
        (cmd << 16) | (par & 0xFFFC) | 1
    }

    /*
        A radio op / immediate command structure of CMDR:
        bit  31                    16               8               2    0
            ----------------------------------------------------------------
            | Command ID (16 bits) | Opt. param     | Opt. ext      | 0  1 |
            ----------------------------------------------------------------
    */
    fn immediate_command(&self) -> *const Self {
        self as *const Self
    }
}

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

/*const BLE_OVERRIDES: [u32; 7] = [
    0x00364038, /* Synth: Set RTRIM (POTAILRESTRIM) to 6 */
    0x000784A3, /* Synth: Set FREF = 3.43 MHz (24 MHz / 7) */
    0xA47E0583, /* Synth: Set loop bandwidth after lock to 80 kHz (K2) */
    0xEAE00603, /* Synth: Set loop bandwidth after lock to 80 kHz (K3, LSB) */
    0x00010623, /* Synth: Set loop bandwidth after lock to 80 kHz (K3, MSB) */
    0x00456088, /* Adjust AGC reference level */
    0xFFFFFFFF, /* End of override list */
];*/

const RFC_DBELL_BASE: *mut RfcBellRegisters = 0x4004_1000 as *mut RfcBellRegisters;
const RFC_PWR_BASE: *mut VolatileCell<u32> = 0x4004_0000 as *mut VolatileCell<u32>;

/*
    RFC Immediate commands
*/
const RFC_CMD0: u16 = 0x607;
const RFC_PING: u16 = 0x406;
const RFC_BUS_REQUEST: u16 = 0x40E;
//const RFC_START_RAT_TIMER: u16 = 0x080A;

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
        // Enable power & clock
        prcm::Power::enable_domain(prcm::PowerDomain::RFC);
        prcm::Clock::enable_rfc();

        // Wait for the power domain to be up
        while !prcm::Power::is_enabled(prcm::PowerDomain::RFC) {}

        let bell_regs: &RfcBellRegisters = unsafe { &*self.bell_regs };

        // Enable CPE
        bell_regs.rf_cpe_interrupt_enable.set(0);
        bell_regs.rf_cpe_interrupt_flags.set(0);

        // Setup clocks and allow CPE to boot
        let pwr_ctl: &VolatileCell<u32> = unsafe { &*self.pwr_ctl };
        pwr_ctl.set(
            RFC_PWR_RFC | RFC_PWR_CPE | RFC_PWR_CPERAM | RFC_PWR_FSCA | RFC_PWR_PHA | RFC_PWR_RAT
                | RFC_PWR_RFE | RFC_PWR_RFERAM | RFC_PWR_MDM | RFC_PWR_MDMRAM,
        );

        // Turn on additional clocks
        bell_regs.rf_ack_interrupt_flag.set(0);
        self.send_and_wait(&DirectCommand::new(RFC_CMD0, 0x10 | 0x40));

        // Request the bus
        self.send_and_wait(&DirectCommand::new(RFC_BUS_REQUEST, 1));

        // Send a ping command to verify that the core is ready and alive
        self.send_and_wait(&DirectCommand::new(RFC_PING, 0));
    }

    pub fn setup(&self, mode: u8, reg_override: u32) {
        let setup_cmd = RfcCommandRadioSetup {
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
            mode: mode,
            lo_divider: 0,
            config: {
                let mut cfg = RfcSetupConfig(0);
                cfg.set_frontend_mode(0); // Differential mode
                cfg.set_bias_mode(false); // Internal bias
                cfg
            },
            tx_power: 0x9330,
            reg_override: reg_override,
        };

        self.send(&setup_cmd);

        // Wait for the cmd to be done
        match self.wait_for(&setup_cmd) {
            RfcResult::Error(status) => panic!("Error occurred during setup: 0x{:x}\r", status),
            RfcResult::Ok => debug_verbose!("Setup successful!\r"),
        }
    }

    pub fn start_rat(&self) {
        unsafe {
            rtc::RTC.set_upd_en(true);
        }

        let cmd = RfcCommandStartRat {
            command_no: 0x080A,
            status: 0,
            next_op: 0,
            start_time: 0,
            start_trigger: 0,
            condition: {
                let mut cond = RfcCondition(0);
                cond.set_rule(0x01); // COND_NEVER
                cond
            },
            _reserved: 0,
            rat0: 0,
        };

        self.send(&cmd);

        // Wait for the cmd to be done
        match self.wait_for(&cmd) {
            RfcResult::Error(status) => panic!("Error occurred during RAT start: 0x{:x}\r", status),
            RfcResult::Ok => debug_verbose!("RAT started.\r"),
        }
    }

    pub fn send_and_wait<C: RfcCommand>(&self, cmd: &C) {
        self.send(cmd);
        match self.wait_for(cmd) {
            RfcResult::Error(status) => panic!("Error occurred during send_and_wait cmdsta=0x{:x}\r", status),
            RfcResult::Ok => (),
        }
    }

    pub fn send<C: RfcCommand>(&self, cmd: &C) -> RfcResult {
        let command: u32 = match cmd.command_type() {
            RfcCommandType::Direct => cmd.direct_command(),
            RfcCommandType::Immediate => cmd.immediate_command() as u32,
        };

        // Check if the radio is accessible or not
        if !prcm::Power::is_enabled(prcm::PowerDomain::RFC) {
            panic!("RFC power domain is off.\r");
        }

        let bell_regs: &RfcBellRegisters = unsafe { &*self.bell_regs };

        while bell_regs.command.get() != 0 {}

        // Set the command
        bell_regs.command.set(command);

        debug_verbose!("Radio command pending: 0x{:x}\r", command);

        // Wait for the ACK
        //while(!HWREG(RFC_DBELL_BASE + RFC_DBELL_O_RFACKIFG));
        while bell_regs.rf_ack_interrupt_flag.get() == 0 { }
        bell_regs.rf_ack_interrupt_flag.set(0);

        RfcResult::Ok
    }

    pub fn wait_for<C: RfcCommand>(&self, cmd: &C) -> RfcResult {
        let mut timeout: u32 = 0;
        let mut status = 0;
        const MAX_TIMEOUT: u32 = 0x2FFFFFF;

        match cmd.command_type() {
            /*
                Direct commands return directly with a result (if communication is enabled).
                    CMD_DONE = 0x01
                And is read from CMDSTA in the DBELL registers.
            */
            RfcCommandType::Direct => {
                let bell_regs: &RfcBellRegisters = unsafe { &*self.bell_regs };
                while timeout < MAX_TIMEOUT {
                    status = bell_regs.command_status.get();
                    if (status & 0xFF) == 0x01 {
                        return RfcResult::Ok;
                    }

                    timeout += 1;
                }
            }

            /*
                Immediate/Radio operations does not return directly, and can take a while to
                complete depending on the complexity of the command. The result is then directly
                written to the status register of the command sent.
            */
            RfcCommandType::Immediate => while timeout < MAX_TIMEOUT {
                status = *cmd.command_status() as u32;
                if (status & 0x0C00) == 0x0400 {
                    debug_verbose!("Got status 0x{:x}\r", status);
                    return RfcResult::Ok;
                }

                timeout += 1;
            },
        }

        // If we arrive here, an error occurred above (timed out)
        return RfcResult::Error(status);
    }

    pub fn get_status(&self) -> u32 {
        let bell_regs: &RfcBellRegisters = unsafe { &*self.bell_regs };
        bell_regs.command_status.get()
    }

    pub fn get_command(&self) -> u32 {
        let bell_regs: &RfcBellRegisters = unsafe { &*self.bell_regs };
        bell_regs.command.get()
    }

    pub fn handle_interrupt(&self, int: RfcInterrupt) {
        let bell_regs: &RfcBellRegisters = unsafe { &*self.bell_regs };
        match int {
            RfcInterrupt::CmdAck => {
                // Clear the interrupt
                bell_regs.rf_ack_interrupt_flag.set(0);

                debug_verbose!("CmdAck handled\r");
            }
            _ => (),
        }
    }
}

pub mod rfc_commands {
    use radio::rfc::{RfcCommand, RfcCommandType};

    /* Basic direct command */
    pub struct DirectCommand {
        pub command_id: u16,
        pub parameters: u16,
    }

    impl DirectCommand {
        pub const fn new(command: u16, param: u16) -> DirectCommand {
            DirectCommand {
                command_id: command,
                parameters: param,
            }
        }
    }

    impl RfcCommand for DirectCommand {
        fn command_type(&self) -> RfcCommandType {
            RfcCommandType::Direct
        }
        fn command_id(&self) -> &u16 {
            &self.command_id
        }
        fn command_status(&self) -> &u16 {
            &self.parameters
        }
    }

    /* Basic immediate command */
    #[repr(C)]
    pub struct ImmediateCommand {
        // These fields below are always the first bytes in any rfc command
        // which is a radio operation or a immediate command.
        pub command_no: u16,
        pub status: u16,
        pub next_op: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
    }

    impl RfcCommand for ImmediateCommand {
        fn command_type(&self) -> RfcCommandType {
            RfcCommandType::Immediate
        }
        fn command_id(&self) -> &u16 {
            &self.command_no
        }
        fn command_status(&self) -> &u16 {
            &self.status
        }
    }

    /* In order to properly setup the radio mode (e.g BLE or IEEE) */
    #[repr(C)]
    pub struct RfcCommandRadioSetup {
        pub command_no: u16,
        pub status: u16,
        pub p_nextop: u32,
        pub ratmr: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub mode: u8,
        pub lo_divider: u8,
        pub config: RfcSetupConfig,
        pub tx_power: u16,
        pub reg_override: u32,
    }

    impl RfcCommand for RfcCommandRadioSetup {
        fn command_type(&self) -> RfcCommandType {
            RfcCommandType::Immediate
        }
        fn command_id(&self) -> &u16 {
            &self.command_no
        }
        fn command_status(&self) -> &u16 {
            &self.status
        }
    }

    #[repr(C)]
    pub struct RfcCommandStartRat {
        pub command_no: u16,
        pub status: u16,
        pub next_op: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub _reserved: u16,
        pub rat0: u32,
    }

    impl RfcCommand for RfcCommandStartRat {
        fn command_type(&self) -> RfcCommandType {
            RfcCommandType::Immediate
        }
        fn command_id(&self) -> &u16 {
            &self.command_no
        }
        fn command_status(&self) -> &u16 {
            &self.status
        }
    }

    /* Bitfields used by many commands */
    bitfield!{
        #[derive(Copy, Clone)]
        pub struct RfcTrigger(u8);
        impl Debug;
        pub _trigger_type, _set_trigger_type  : 3, 0;
        pub _enable_cmd, _set_enable_cmd      : 4;
        pub _trigger_no, _set_trigger_no      : 6, 5;
        pub _past_trigger, _set_past_trigger  : 7;
    }

    bitfield!{
        #[derive(Copy, Clone)]
        pub struct RfcCondition(u8);
        impl Debug;
        pub _rule, set_rule : 3, 0;
        pub _skip, _set_skip : 7, 4;
    }

    bitfield!{
        #[derive(Copy, Clone)]
        pub struct RfcSetupConfig(u16);
        impl Debug;
        pub _frontend_mode, set_frontend_mode: 2, 0;
        pub _bias_mode, set_bias_mode: 3;
        pub _analog_cfg_mode, _set_analog_config_mode: 9, 4;
        pub _no_fs_powerup, _set_no_fs_powerup: 10;
    }
}
