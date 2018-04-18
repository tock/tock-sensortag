use power_manager::{PowerManager, Resource, ResourceManager};
use prcm::{Power,PowerDomain};

use kernel::common::regs::ReadWrite;
use aux;
use aon;
use prcm;
use setup::recharge;
use rtc;

pub static mut PM: PowerManager<RegionManager> = PowerManager::new(RegionManager);

pub static mut POWER_REGIONS: [Resource; 2] = [
    Resource::new(PowerDomain::Serial as u32),
    Resource::new(PowerDomain::Peripherals as u32)
];

pub struct RegionManager;

impl ResourceManager for RegionManager {
    fn enable_resource(&self, resource_id: u32) {
        let domain = PowerDomain::from(resource_id);
        Power::enable_domain(domain);
    }

    fn disable_resource(&self, resource_id: u32) {
        let domain = PowerDomain::from(resource_id);
        Power::disable_domain(domain);
    }
}

/// Initialise the power management
/// dependencies and resources.
pub unsafe fn init() {
    for pwr_region in POWER_REGIONS.iter() {
        PM.register_resource(&pwr_region);
    }
}

pub struct SystemControlRegisters {
    scr: ReadWrite<u32, SystemControl::Register>,
}

register_bitfields![
    u32,
    SystemControl [
        SLEEP_ON_EXIT   OFFSET(1)   NUMBITS(1) [], // Go to sleep after ISR
        SLEEP_DEEP      OFFSET(2)   NUMBITS(1) [], // Enable deep sleep
        SEVONPEND       OFFSET(4)   NUMBITS(1) []  // Wake up on all events (even disabled interrupts)
    ]
];

const SYS_CTRL_BASE: u32 = 0xE000ED10;

/// Transition into deep sleep
pub unsafe fn prepare_deep_sleep() {
    // In order to preserve the pins we need to apply an
    // io latch which will freeze the states of each pin during sleep modes
    aon::AON.lock_io_pins(true);

    // We need to allow the aux domain to sleep when we enter sleep mode
    aux::AUX_CTL.wakeup_event(aux::WakeupMode::AllowSleep);

    // Set the ram retention to retain SRAM
    aon::AON.mcu_set_ram_retention(true);

    // Force disable dma & crypto
    // This due to that we can not successfully power down the MCU
    // without them disabled (see p. 496 in the docs)
    prcm::force_disable_dma_and_crypto();

    // Disable all domains except Peripherals & Serial
    // The peripheral & serial domain can be powered on during deep sleep,
    // and is sometimes necessary. This is sometimes also done in
    // the peripheral management, but here we ensure that they are completely
    // disabled if some has been forgotten.
    prcm::Power::disable_domain(prcm::PowerDomain::VIMS);
    prcm::Power::disable_domain(prcm::PowerDomain::RFC);
    prcm::Power::disable_domain(prcm::PowerDomain::CPU);

    // If either Peripherals nor Serial domain is powered on, we need to supply
    // power using the ULDO power supply; which is a low power driver
    if !prcm::Power::is_enabled(prcm::PowerDomain::Peripherals)
        && !prcm::Power::is_enabled(prcm::PowerDomain::Serial) {
        prcm::acquire_uldo();
    }

    // Disable JTAG entirely, otherwise we'll never
    // transition into deep sleep (if a debugger is attached, we still won't).
    aon::AON.jtag_set_enabled(false);

    // Enable power down of the MCU
    aon::AON.mcu_power_down();

    // We need to setup the recharge algorithm by TI, since this
    // will tweak the variables depending on the power & current in order to successfully
    // recharge.
    recharge::before_power_down(0);

    // Sync with the RTC before we are ready to transition into deep sleep
    rtc::RTC.sync();

    // Set the deep sleep bit
    let regs: &SystemControlRegisters = &*(SYS_CTRL_BASE as *const SystemControlRegisters);
    regs.scr.modify(SystemControl::SLEEP_DEEP::SET + SystemControl::SEVONPEND::SET);
}

/// Perform necessary setup once we've woken up from deep sleep
pub unsafe fn prepare_wakeup() {
    // Once we've woken up we need to sync with the RTC to be able
    // to read values which has changed in the AON region during sleep.
    rtc::RTC.sync();

    // We're ready to allow the auxilliary domain to wake up once it's needed.
    aux::AUX_CTL.wakeup_event(aux::WakeupMode::WakeUp);

    // If we were using the ULDO power to supply the peripherals, we can safely
    // disable it now - it is unnecessary if it's started.
    prcm::release_uldo();

    // Enable the CPU power domain once again, or ensure that it is powered (it often are
    // when we transition from sleep).
    prcm::Power::enable_domain(prcm::PowerDomain::CPU);

    // Again, sync with the AON since the ULDO might have been released.
    rtc::RTC.sync();

    // Unlock IO pins and let them be controlled by GPIO
    aon::AON.lock_io_pins(false);

    recharge::after_power_down();

    // Sync with the AON after our recharge calibration
    rtc::RTC.sync();

    // Clear the deep sleep bit
    let regs: &SystemControlRegisters = &*(SYS_CTRL_BASE as *const SystemControlRegisters);
    regs.scr.modify(SystemControl::SLEEP_DEEP::CLEAR);
}