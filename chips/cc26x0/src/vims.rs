
use kernel::common::VolatileCell;

pub fn disable() {
    const VIMS_BASE: u32 = 0x4003_4000;
    const VIMS_O_CTL: u32 = 0x00000004;

    let vims_ctl: &VolatileCell<u32> = unsafe { &*((VIMS_BASE + VIMS_O_CTL) as *const VolatileCell<u32>) };
    vims_ctl.set(0x00000003); // disable VIMS
}

pub fn enable() {
    const VIMS_BASE: u32 = 0x4003_4000;
    const VIMS_O_CTL: u32 = 0x00000004;

    let vims_ctl: &VolatileCell<u32> = unsafe { &*((VIMS_BASE + VIMS_O_CTL) as *const VolatileCell<u32>) };
    vims_ctl.set(0x00000003); // disable VIMS
}