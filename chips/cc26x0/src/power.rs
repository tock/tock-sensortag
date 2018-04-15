use power_manager::{PowerManager, PowerResource, ResourceHandler};
use prcm::{Power,PowerDomain};

pub static mut PM: PowerManager<'static, RegionManager> = PowerManager::new(RegionManager);

pub static mut PWR_REGIONS: [PowerResource; 2] = [
    PowerResource::new(PowerDomain::Serial as u32),
    PowerResource::new(PowerDomain::Peripherals as u32)
];

pub struct RegionManager;

impl ResourceHandler for RegionManager {
    fn power_on_resource(&self, resource_id: u32) {
        let domain = PowerDomain::from(resource_id);
        match domain {
            PowerDomain::Serial => Power::enable_domain(PowerDomain::Serial),
            _ => {}
        }
    }

    fn power_off_resource(&self, resource_id: u32) {
        let domain = PowerDomain::from(resource_id);
        match domain {
            PowerDomain::Serial => Power::disable_domain(PowerDomain::Serial),
            _ => {}
        }
    }
}

pub unsafe fn init_power_management() {
    for pwr_region in PWR_REGIONS.iter() {
        PM.add_resource(&pwr_region);
    }
}
