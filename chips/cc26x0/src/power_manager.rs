/// Power Manager
///
/// Generalised power management for peripherals. It handles toggling
/// their power on request & release once they've been used.
///
// TODO(cpluss): complete documentation above

use core::cell::Cell;
use kernel::common::{List, ListLink, ListNode};

/// PoweredClient
///     A peripheral which is powered (almost all) and manages
///     its own requests of powers - to be used in conjunction with a
///     PoweredClient.
///
///     NOTE: you need to register all Powered Peripherals
///           during initialisation.
pub trait PoweredClient {
    /// Identifier for this peripheral. Used to distinguish
    /// peripherals between each other.
    fn identifier(&self) -> u32;

    /// Power on this peripheral
    fn power_on(&self);

    /// Power off this peripheral
    fn power_off(&self);

    /// This is invoked before the chip goes into sleep mode, if the
    /// peripheral is powered.
    fn before_sleep(&self);

    /// This is invoked once the chip has woken up from any sleep mode.
    fn after_wakeup(&self);

    /// This is the lowest sleep mode this module will still be able
    /// to function in.
    fn lowest_sleep_mode(&self) -> u32;
}

pub struct PoweredPeripheral<'a> {
    client: Cell<Option<&'a PoweredClient>>,
    next: ListLink<'a, PoweredPeripheral<'a>>,
    usage_count: Cell<u32>,
}

impl<'a> ListNode<'a, PoweredPeripheral<'a>> for PoweredPeripheral<'a> {
    fn next(&self) -> &'a ListLink<PoweredPeripheral<'a>> {
        &self.next
    }
}

impl<'a> PoweredPeripheral<'a> {
    pub const fn new(client: &'a PoweredClient) -> PoweredPeripheral<'a> {
        PoweredPeripheral {
            client: Cell::new(Some(client)),
            next: ListLink::empty(),
            usage_count: Cell::new(0),
        }
    }

    pub fn client(&self) -> &'a PoweredClient {
        self.client.get().expect("")
    }

    pub fn usage_map<F>(&self, closure: F)
    where
        F: FnOnce(u32) -> u32,
    {
        let val = self.usage_count.get();
        self.usage_count.set(closure(val))
    }

    pub fn usage(&self) -> u32 {
        self.usage_count.get()
    }
}

/// Power manager
///     Responsible to keep track of what peripherals
///     is used, and powered on. As well as to power
///     on and off specific peripherals.
///
///     It also has the functionality to power off
///     all used peripherals once the MCU is ready
///     to go to sleep mode, as it's needed in many
///     cases.
pub struct PowerManager<'a> {
    /// A list of IDs for powered peripherals
    powered_peripherals: List<'a, PoweredPeripheral<'a>>,
}

impl<'a> PowerManager<'a> {
    pub const fn new() -> PowerManager<'a> {
        PowerManager {
            powered_peripherals: List::new(),
            //_chip: Cell::new(None),
        }
    }

    /// Register a powered peripheral to hook up and be notified
    /// when specific events occur (sleep, etc).
    pub fn register(&self, peripheral: &'a PoweredPeripheral<'a>) {
        self.powered_peripherals.push_head(peripheral);
    }

    /// Request access for a specific peripheral to be used
    pub fn request(&self, identifier: u32) {
        let powered_peripheral = self.powered_peripherals
            .iter()
            .find(|p| p.client().identifier() == identifier)
            .expect("peripheral requested that has not been registered.");

        if powered_peripheral.usage() == 0 {
            powered_peripheral.client().power_on();
        }

        powered_peripheral.usage_map(|usage: u32| usage + 1);
    }

    /// Release a specific peripheral as no longer being used
    pub fn release(&self, identifier: u32) {
        let powered_peripheral = self.powered_peripherals
            .iter()
            .find(|p| p.client().identifier() == identifier)
            .expect("peripheral requested that has not been registered.");

        powered_peripheral.usage_map(|usage: u32| {
            if usage > 0 {
                usage - 1
            } else {
                0
            }
        });

        if powered_peripheral.usage() == 0 {
            powered_peripheral.client().power_off();
        }
    }

    pub fn prepare_sleep(&self) {
        for peripheral in self.powered_peripherals.iter() {
            if peripheral.usage() > 0 {
                peripheral.client().before_sleep();
                peripheral.client().power_off();
            }
        }
    }

    pub fn after_wakeup(&self) {
        for peripheral in self.powered_peripherals.iter() {
            if peripheral.usage() > 0 {
                peripheral.client().power_on();
                peripheral.client().after_wakeup();
            }
        }
    }


}
