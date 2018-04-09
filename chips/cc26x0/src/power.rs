use power_manager;
use prcm;

use gpio;
use uart;

/// The power region manager specific
/// for the CC26X0 chip, used to enable & disable power
/// regions on demand.
pub struct PowerRegionManager(());

impl power_manager::PowerRegionManager for PowerRegionManager {
    fn enable(&self, region: u32) {
        let domain = prcm::PowerDomain::from(region);
        prcm::Power::enable_domain(domain);
        while !self.is_enabled(region) {}
    }

    fn disable(&self, region: u32) {
        let domain = prcm::PowerDomain::from(region);
        prcm::Power::disable_domain(domain);
        while self.is_enabled(region) {}
    }

    fn is_enabled(&self, region: u32) -> bool {
        let domain = prcm::PowerDomain::from(region);
        prcm::Power::is_enabled(domain)
    }

    fn sleep(&self, _mode: u32) {
        // TODO(cpluss): implement sleep modes
        unimplemented!()
    }
}

// Low Power Manager
pub static LPM: power_manager::PowerManager<PowerRegionManager>
= power_manager::PowerManager::new(PowerRegionManager(()), &[
    &gpio::GPIO_POWER_DEPENDENCY,
    &uart::UART_POWER_DEPENDENCY,
]);
