use core::cell::Cell;

/// PowerRegionManager
///     Used to enable or disable specific power
///     regions on the chip. Provide an enum
///     of available regions.
pub trait PowerRegionManager {
    /// Enable a certain power region on the chip
    fn enable(&self, region: u32);

    /// Disable a certain power region on the chip
    fn disable(&self, region: u32);

    /// Check to see whether a specific power region
    /// is already enabled or not.
    fn is_enabled(&self, region: u32) -> bool;

    /// Go to a specific sleep mode
    fn sleep(&self, mode: u32);
}

/// PowerModule
///     A power module is a contract of what power region
///     a specific peripheral requires in order to function.
pub trait PowerModule {
    /// A unique ID identifying this power module.
    /// This is used when comparing power modules:
    fn id(&self) -> u32;

    /// This is the region which this peripheral
    /// requires in order to function properly (eg. Peripheral power domain).
    fn regions(&self) -> &[u32];

    /// This is the lowest sleep mode this module will still be able
    /// to function in.
    fn lowest_sleep_mode(&self) -> u32;

    /// This is invoked before the chip goes into sleep mode, if the
    /// peripheral is powered.
    fn prepare_for_sleep(&self);

    /// This is invoked once the chip has woken up from any sleep mode.
    fn wakeup(&self);
}

/// Power dependency
///     Sets up a power constraint for a module, and includes
///     the number of times it has been registered.
pub struct PowerDependency {
    // Once we unregister a power dependency
    // we don't delete it, just set it to unused. There won't
    // be that many power modules, and they are all statically
    // for each board.
    used: Cell<bool>,
    module: &'static PowerModule,
}

impl PowerDependency {
    pub const fn new<M: PowerModule>(module: &'static M) -> PowerDependency {
        PowerDependency {
            used: Cell::new(false),
            module,
        }
    }
}

use core::marker::Sync;
unsafe impl Sync for PowerDependency {
}

/// Power manager
///     Responsible to keep track of what power regions
///     is needed, and powered on. As well as to power
///     on and off specific regions once they are no longer
///     required.
///
///     It also determines if the chip is ready to go into sleep mode,
///     by determining whether any region is powered on and can still
///     function in a lower sleep mode.
pub struct PowerManager<Prm: PowerRegionManager> {
    dependencies: &'static [&'static PowerDependency],
    region_manager: Prm,
}

impl<Prm: PowerRegionManager> PowerManager<Prm> {
    pub const fn new(prm: Prm, dependencies: &'static [&'static PowerDependency]) -> PowerManager<Prm> {
        PowerManager {
            dependencies,
            region_manager: prm,
        }
    }

    /// Registers a module to be used - it works multiple
    /// times for each module, and will only power on a region
    /// if it isn't already powered.
    pub fn register<T: PowerModule>(&self, module: &'static T) {
        let existing_module =
            self.dependencies
                .iter()
                .find(|dep| dep.module.id() == module.id());

        match existing_module {
            None => {
                panic!("Tried to enable a power dependency which was not registered.\r");
            },

            Some(m) => {
                // In case it is already registered, we check if its used or discarded
                if m.used.get() {
                    return;
                } else {
                    m.used.set(true);
                }
            }
        }

        for region in module.regions().iter() {
            if !self.region_manager.is_enabled(*region) {
                self.region_manager.enable(*region);
            }
        }
    }

    /// Unregister a module which is in use - it works multiple times
    /// for each module, depending on how many drivers are using the
    /// module in a specific power region.
    pub fn unregister<T: PowerModule>(&self, module: &'static T) {
        let existing_module =
            self.dependencies
                .iter()
                .find(|dep| dep.module.id() == module.id());

        match existing_module {
            None => (),
            Some(m) => {
                m.used.set(false);

                for region in module.regions().iter() {
                    let used_by_other =
                        self.dependencies
                            .iter()
                            .find(|dep| {
                                dep.module.regions()
                                    .iter()
                                    .any(| r | *r == *region)
                            });

                    // Skip this if it's being used by another module
                    if used_by_other.is_some() {
                        continue;
                    }

                    debug_verbose!("Disabling region {}\r", *region);
                    self.region_manager.disable(*region);
                }
            }
        }
    }

    pub fn sleep(&self) {}
}
