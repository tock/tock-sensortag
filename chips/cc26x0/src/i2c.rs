use prcm;
use ioc;
use cc26xx::gpio;
use kernel::hil::gpio::Pin;
use core::cell::Cell;
use kernel::common::VolatileCell;

pub const I2C_MCR_MFE: u32 = 0x10;
pub const I2C_MCTRL_RUN: u32 = 0x1;

pub const I2C_MASTER_CMD_SINGLE_SEND: u32 = 0x7;
pub const I2C_MASTER_CMD_BURST_SEND_ERROR_STOP: u32 = 0x4;
pub const I2C_MASTER_CMD_BURST_RECEIVE_START: u32 = 0xb;
pub const I2C_MASTER_CMD_BURST_RECEIVE_CONT: u32 = 0x9;
pub const I2C_MASTER_CMD_BURST_SEND_START: u32 = 0x3;
pub const I2C_MASTER_CMD_BURST_SEND_CONT: u32 = 0x1;
pub const I2C_MASTER_CMD_BURST_SEND_FINISH: u32 = 0x5;
pub const I2C_MASTER_CMD_BURST_RECEIVE_FINISH: u32 = 0x5;

pub const I2C_MSTAT_ERR: u32 = 0x2;
pub const I2C_MSTAT_BUSY: u32 = 0x1;
pub const I2C_MSTAT_BUSBSY: u32 = 0x40;
pub const I2C_MSTAT_ARBLST: u32 = 0x10;
pub const I2C_MSTAT_DATACK_N: u32 = 0x8;
pub const I2C_MSTAT_ADRACK_N: u32 = 0x4;
pub const I2C_MSTAT_DATACK_N_M: u32 = 0x8;
pub const I2C_MSTAT_ADRACK_N_M: u32 = 0x4;

pub const BOARD_IO_SDA: usize = 0x5;
pub const BOARD_IO_SCL: usize = 0x6;
pub const BOARD_IO_SDA_HP: usize = 0x8;
pub const BOARD_IO_SCL_HP: usize = 0x9;

pub const MCU_CLOCK: u32 = 48_000_000;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum I2cInterface {
    Interface0 = 0,
    Interface1 = 1,
    NoInterface = 2,
}

#[repr(C)]
pub struct Registers {
    pub soar: VolatileCell<u32>,
    pub sstat_sctl: VolatileCell<u32>,
    pub sdr: VolatileCell<u32>,
    pub simr: VolatileCell<u32>,
    pub sris: VolatileCell<u32>,
    pub smis: VolatileCell<u32>,
    pub sicr: VolatileCell<u32>,

    _reserved0: [u8; 0x7e4],

    pub msa: VolatileCell<u32>,
    pub mstat_mctrl: VolatileCell<u32>,
    pub mdr: VolatileCell<u32>,
    pub mtpr: VolatileCell<u32>,
    pub mimr: VolatileCell<u32>,
    pub mris: VolatileCell<u32>,
    pub mmis: VolatileCell<u32>,
    pub micr: VolatileCell<u32>,
    pub mcr: VolatileCell<u32>,
}

pub const I2C_BASE: *mut Registers = 0x4000_2000 as *mut Registers;

pub static mut I2C0: I2C = I2C::new();

pub struct I2C {
    regs: *mut Registers,
    slave_addr: Cell<u8>,
    interface: Cell<u8>,
}

impl I2C {
    pub const fn new() -> I2C {
        I2C {
            regs: I2C_BASE as *mut Registers,
            slave_addr: Cell::new(0),
            interface: Cell::new(I2cInterface::NoInterface as u8),
        }
    }

    pub fn wakeup(&self) {
        prcm::Power::enable_domain(prcm::PowerDomain::Serial);
        while !prcm::Power::is_enabled(prcm::PowerDomain::Serial) {}
        prcm::Clock::enable_i2c();

        self.configure(true);
    }

    #[allow(unused)]
    pub fn shutdown(&self) {
        // Not implemented
    }

    fn configure(&self, fast: bool) {
        self.master_enable();

        let freq;
        if fast {
            freq = 400_000;
        } else {
            freq = 100_000;
        }

        // Compute SCL (serial clock) period
        let tpr = ((MCU_CLOCK + (2 * 10 * freq) - 1) / (2 * 10 * freq)) - 1;
        let regs: &Registers = unsafe { &*self.regs };
        regs.mtpr.set(tpr);
    }

    fn master_enable(&self) {
        let regs: &Registers = unsafe { &*self.regs };
        // Set as master
        regs.mcr.set(regs.mcr.get() | I2C_MCR_MFE);
        // Enable master to transfer/receive data
        regs.mstat_mctrl.set(I2C_MCTRL_RUN);
    }

    fn master_disable(&self) {
        let regs: &Registers = unsafe { &*self.regs };
        regs.mstat_mctrl.set(0);
        regs.mcr.set(regs.mcr.get() & !I2C_MCR_MFE);
    }

    pub fn write_single(&self, data: u8) -> bool {
        self.set_master_slave_address(self.slave_addr.get(), false);
        self.master_put_data(data);

        if !self.busy_wait_master_bus() {
            return false;
        }

        self.master_control(I2C_MASTER_CMD_SINGLE_SEND);
        if !self.busy_wait_master() {
            return false;
        }

        self.status()
    }

    pub fn read(&self, data: &mut [u8], len: u8) -> bool {
        self.set_master_slave_address(self.slave_addr.get(), true);

        self.busy_wait_master_bus();

        self.master_control(I2C_MASTER_CMD_BURST_RECEIVE_START);

        let mut i = 0;
        let mut success = true;
        while i < (len - 1) && success {
            self.busy_wait_master();
            success = self.status();
            if success {
                data[i as usize] = self.master_get_data() as u8;
                self.master_control(I2C_MASTER_CMD_BURST_RECEIVE_CONT);
                i += 1;
            }
        }

        if success {
            self.master_control(I2C_MASTER_CMD_BURST_RECEIVE_FINISH);
            self.busy_wait_master();
            success = self.status();
            if success {
                data[(len - 1) as usize] = self.master_get_data() as u8;
                self.busy_wait_master_bus();
            }
        }

        success
    }

    pub fn write(&self, data: &[u8], len: u8) -> bool {
        self.set_master_slave_address(self.slave_addr.get(), false);

        self.master_put_data(data[0]);

        self.busy_wait_master_bus();

        self.master_control(I2C_MASTER_CMD_BURST_SEND_START);
        self.busy_wait_master();
        let mut success = self.status();

        for i in 1..len {
            if !success {
                break;
            }
            self.master_put_data(data[i as usize]);
            if i < len - 1 {
                self.master_control(I2C_MASTER_CMD_BURST_SEND_CONT);
                self.busy_wait_master();
                success = self.status();
            }
        }

        if success {
            self.master_control(I2C_MASTER_CMD_BURST_SEND_FINISH);
            self.busy_wait_master();
            success = self.status();
            self.busy_wait_master_bus();
        }

        success
    }

    pub fn write_read(&self, data: &mut [u8], write_len: u8, read_len: u8) -> bool {
        self.set_master_slave_address(self.slave_addr.get(), false);

        self.master_put_data(data[0]);

        self.busy_wait_master_bus();

        self.master_control(I2C_MASTER_CMD_BURST_SEND_START);
        self.busy_wait_master();
        let mut success = self.status();

        for i in 1..write_len {
            if !success {
                break;
            }

            self.master_put_data(data[i as usize]);

            self.master_control(I2C_MASTER_CMD_BURST_SEND_CONT);
            self.busy_wait_master();
            success = self.status();
        }

        if !success {
            return false;
        }

        self.set_master_slave_address(self.slave_addr.get(), true);

        self.master_control(I2C_MASTER_CMD_BURST_RECEIVE_START);

        let mut i = 0;
        while i < (read_len - 1) && success {
            self.busy_wait_master();
            success = self.status();
            if success {
                data[i as usize] = self.master_get_data() as u8;
                self.master_control(I2C_MASTER_CMD_BURST_RECEIVE_CONT);
                i += 1;
            }
        }

        if success {
            self.master_control(I2C_MASTER_CMD_BURST_RECEIVE_FINISH);
            self.busy_wait_master();
            success = self.status();
            if success {
                data[(read_len - 1) as usize] = self.master_get_data() as u8;
                self.busy_wait_master_bus();
            }
        }

        success
    }

    fn set_master_slave_address(&self, addr: u8, receive: bool) {
        let regs: &Registers = unsafe { &*self.regs };
        regs.msa.set(((addr as u32) << 1) | (receive as u32));
    }

    fn master_put_data(&self, data: u8) {
        let regs: &Registers = unsafe { &*self.regs };
        regs.mdr.set(data as u32);
    }

    fn master_get_data(&self) -> u32 {
        let regs: &Registers = unsafe { &*self.regs };
        regs.mdr.get()
    }

    fn master_bus_busy(&self) -> bool {
        let regs: &Registers = unsafe { &*self.regs };
        (regs.mstat_mctrl.get() & I2C_MSTAT_BUSBSY) != 0
    }

    fn master_busy(&self) -> bool {
        let regs: &Registers = unsafe { &*self.regs };
        (regs.mstat_mctrl.get() & I2C_MSTAT_BUSY) != 0
    }

    // Limited busy wait for the master
    fn busy_wait_master(&self) -> bool {
        let delay = 0xFFFFFF;
        for _ in 0..delay {
            if !self.master_busy() {
                return true;
            }
        }
        false
    }

    // Limited busy wait for the master bus
    fn busy_wait_master_bus(&self) -> bool {
        let delay = 0xFFFFFF;
        for _ in 0..delay {
            if !self.master_bus_busy() {
                return true;
            }
        }
        false
    }

    fn master_control(&self, cmd: u32) {
        let regs: &Registers = unsafe { &*self.regs };
        regs.mstat_mctrl.set(cmd);
    }

    fn status(&self) -> bool {
        let status = self.master_err();

        if (status & (I2C_MSTAT_DATACK_N_M | I2C_MSTAT_ADRACK_N_M)) != 0 {
            self.master_control(I2C_MASTER_CMD_BURST_SEND_ERROR_STOP);
        }

        status == 0
    }

    fn master_err(&self) -> u32 {
        let regs: &Registers = unsafe { &*self.regs };
        let err = regs.mstat_mctrl.get();

        // If the master is busy there is not error to report
        if (err & I2C_MSTAT_BUSY) == 1 {
            return 0;
        }

        // Check for errors
        if err & (I2C_MSTAT_ERR | I2C_MSTAT_ARBLST) != 0 {
            return err & (I2C_MSTAT_ARBLST | I2C_MSTAT_DATACK_N | I2C_MSTAT_ADRACK_N);
        } else {
            return 0;
        }
    }

    fn accessible(&self) -> bool {
        if !prcm::Power::is_enabled(prcm::PowerDomain::Serial) {
            return false;
        }

        if !prcm::Clock::i2c_run_clk_enabled() {
            return false;
        }

        true
    }

    pub fn select(&self, new_interface: I2cInterface, addr: u8) {
        self.slave_addr.set(addr);

        if !self.accessible() {
            self.wakeup();
        }

        let interface = new_interface as u8;
        if interface != self.interface.get() as u8 {
            self.interface.set(interface);

            self.master_disable();

            if interface == I2cInterface::Interface0 as u8 {
                unsafe {
                    ioc::IOCFG[BOARD_IO_SDA].enable_i2c_sda();
                    ioc::IOCFG[BOARD_IO_SCL].enable_i2c_scl();
                    gpio::PORT[BOARD_IO_SDA_HP].make_input();
                    gpio::PORT[BOARD_IO_SCL_HP].make_input();
                }
            } else if interface == I2cInterface::Interface1 as u8 {
                unsafe {
                    ioc::IOCFG[BOARD_IO_SDA_HP].enable_i2c_sda();
                    ioc::IOCFG[BOARD_IO_SCL_HP].enable_i2c_scl();
                    gpio::PORT[BOARD_IO_SDA].make_input();
                    gpio::PORT[BOARD_IO_SCL].make_input();
                }
            }

            self.configure(true);
        }
    }
}
