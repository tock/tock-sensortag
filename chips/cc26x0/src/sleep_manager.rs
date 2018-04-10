use kernel::support;
use kernel::common::regs::ReadWrite;

const SYS_CTRL_BASE: u32 = 0xE000ED10;

pub struct SystemControlRegister {
    scr: ReadWrite<u32, SystemControl::Register>,
}

register_bitfields![
    u32,
    SystemControl [
        SLEEP_ON_EXIT   OFFSET(1) NUMBITS(1) [],    // Got to sleep after ISR
        SLEEP_DEEP      OFFSET(2) NUMBITS(1) [],    // Enable deep sleep
        SEVONPEND       OFFSET(4) NUMBITS(1) []     // Wake up on all events (even disabled interrupts)
    ]
];

pub static mut SLEEP_MAN: SleepManager = SleepManager::new();

pub struct SleepManager {
    system_ctrl_register: *const SystemControlRegister,
}

impl SleepManager {
    pub const fn new() -> SleepManager {
        SleepManager {
            system_ctrl_register: SYS_CTRL_BASE as *const SystemControlRegister,
        }
    }

    /// Puts the microcontroller to sleep until an interrupt happens
    pub fn enter_sleep(&self) {
        // Disable deep sleep just to be sure
        self.set_deep_sleep(false);

        self.sleep();
    }

    /// Puts the microcontroller in deep sleep mode and waits for an interrupt
    pub fn enter_deep_sleep(&self) {
        // Enable deep sleep
        self.set_deep_sleep(true);

        self.sleep();
    }

    #[allow(unused)]
    fn prepare_for_deep_sleep(&self) {}

    fn set_deep_sleep(&self, enable: bool) {
        let ctrl_reg = unsafe { &*self.system_ctrl_register };
        if enable {
            ctrl_reg.scr.modify(SystemControl::SLEEP_DEEP::SET);
        }
        else {
            ctrl_reg.scr.modify(SystemControl::SLEEP_DEEP::CLEAR);
        }

    }

    fn sleep(&self) {
        unsafe {
            support::wfi();
        }
    }
}