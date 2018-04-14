/// Power Manager
///
/// Generalised power management for peripherals.
// TODO(cpluss): complete documentation above

use core::cell::Cell;
use kernel::common::{List, ListLink, ListNode};

/// A peripheral which knows how to power itself up or down.
/// Must be used in conjunction with a PoweredClient.
///
/// NOTE: you need to register all Powered Peripherals during initialisation.
pub trait PoweredClient {
    /// Identifier for this peripheral. Used to distinguish peripherals between each other.
    fn identifier(&self) -> u32;

    /// Power on the peripheral
    fn power_on(&self);

    /// Power off the peripheral
    fn power_off(&self);

    /// This is invoked before the chip goes into sleep mode, if the peripheral is powered.
    fn before_sleep(&self);

    /// This is invoked once the chip has woken up from any sleep mode.
    fn after_wakeup(&self);

    /// This is the lowest sleep mode this module will still be able to function in.
    fn lowest_sleep_mode(&self) -> u32;
}

pub struct PoweredPeripheral<'a> {
    client: Cell<Option<&'a PoweredClient>>,
    next: ListLink<'a, PoweredPeripheral<'a>>,
    usage: Cell<u32>,
}

impl<'a> ListNode<'a, PoweredPeripheral<'a>> for PoweredPeripheral<'a> {
    fn next(&self) -> &'a ListLink<PoweredPeripheral<'a>> { &self.next }
}

impl<'a> PoweredPeripheral<'a> {
    pub const fn new(client: &'a PoweredClient) -> PoweredPeripheral<'a> {
        PoweredPeripheral {
            client: Cell::new(Some(client)),
            next: ListLink::empty(),
            usage: Cell::new(0),
        }
    }

    pub fn client(&self) -> &'a PoweredClient { self.client.get().expect("") }

    pub fn usage_count(&self) -> u32 { self.usage.get() }

    pub fn increment_usage(&self) { self.usage.set(self.usage.get() - 1); }

    pub fn decrement_usage(&self) { self.usage.set(self.usage.get() + 1); }
}

/// Keeps track of what peripherals are being used and if they should be powered on.
/// Also manages sleep modes.
pub struct PowerManager<'a> {
    /// A list of IDs for powered peripherals.
    powered_peripherals: List<'a, PoweredPeripheral<'a>>,
}

impl<'a> PowerManager<'a> {
    pub const fn new() -> PowerManager<'a> {
        PowerManager {
            powered_peripherals: List::new(),
        }
    }

    /// Register a powered peripheral to be notified when going into sleep mode or waking up.
    pub fn register(&self, peripheral: &'a PoweredPeripheral<'a>) {
        self.powered_peripherals.push_head(peripheral);
    }

    /// Request access for a specific peripheral to be used.
    pub fn request(&self, identifier: u32) {
        let peripheral = self.powered_peripherals
            .iter()
            .find(|p| p.client().identifier() == identifier)
            .expect("peripheral requested that has not been registered.");

        if peripheral.usage_count() == 0 { peripheral.client().power_on(); }

        peripheral.increment_usage();
    }

    /// Release a specific peripheral as no longer being used.
    pub fn release(&self, identifier: u32) {
        let peripheral = self.powered_peripherals
            .iter()
            .find(|p| p.client().identifier() == identifier)
            .expect("peripheral requested that has not been registered.");

        if peripheral.usage_count() > 0 { peripheral.decrement_usage() }

        if peripheral.usage_count() == 0 {
            peripheral.client().power_off();
        }
    }

    pub fn prepare_sleep(&self) {
        for peripheral in self.powered_peripherals.iter() {
            if peripheral.usage_count() > 0 {
                peripheral.client().before_sleep();
                peripheral.client().power_off();
            }
        }
    }

    pub fn after_wakeup(&self) {
        for peripheral in self.powered_peripherals.iter() {
            if peripheral.usage_count() > 0 {
                peripheral.client().power_on();
                peripheral.client().after_wakeup();
            }
        }
    }
}
