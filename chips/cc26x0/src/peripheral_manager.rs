use core::cell::Cell;
use kernel::common::{List, ListLink, ListNode};

/// A PowerClient implements ways to get notified when the chip changes its power mode.
pub trait PowerClient {
    fn before_sleep(&self, sleep_mode: u32);
    fn after_wakeup(&self, sleep_mode: u32);
    fn lowest_sleep_mode(&self) -> u32;
}

/// Wrapper around PowerClient to be used in a linked list.
pub struct Peripheral<'a> {
    client: Cell<Option<&'a PowerClient>>,
    next: ListLink<'a, Peripheral<'a>>,
}

impl<'a> Peripheral<'a> {
    pub const fn new(client: &'a PowerClient) -> Peripheral {
        Peripheral {
            client: Cell::new(Some(client)),
            next: ListLink::empty(),
        }
    }

    /// Returns the lowest possible power mode this peripheral can enter at the moment.
    pub fn lowest_sleep_mode(&self) -> u32 {
        self.client
            .get()
            .map(|c| c.lowest_sleep_mode())
            .expect("No power client for a peripheral is set.")
    }

    /// Prepares the peripheral before going into sleep mode.
    pub fn before_sleep(&self, sleep_mode: u32) {
        self.client.get().map(|c| c.before_sleep(sleep_mode));
    }

    /// Initializes the peripheral after waking up from sleep mode.
    pub fn after_wakeup(&self, sleep_mode: u32) {
        self.client.get().map(|c| c.after_wakeup(sleep_mode));
    }
}

impl<'a> ListNode<'a, Peripheral<'a>> for Peripheral<'a> {
    fn next(&self) -> &'a ListLink<Peripheral<'a>> {
        &self.next
    }
}

/// Manages peripherals wanting to get notified when changing power modes.
pub struct PeripheralManager<'a> {
    peripherals: List<'a, Peripheral<'a>>,
}

impl<'a> PeripheralManager<'a> {
    pub const fn new() -> PeripheralManager<'a> {
        PeripheralManager {
            peripherals: List::new(),
        }
    }

    /// Registers a new peripheral to be managed by the PeripheralManager.
    pub fn register_peripheral(&self, peripheral: &'a Peripheral<'a>) {
        self.peripherals.push_head(peripheral);
    }

    /// Prepares all registered clients for entering sleep mode.
    pub fn before_sleep(&self, sleep_mode: u32) {
        for peripheral in self.peripherals.iter() {
            peripheral.before_sleep(sleep_mode);
        }
    }

    /// Starts all registered clients after waking up from sleep mode.
    pub fn after_wakeup(&self, sleep_mode: u32) {
        for peripheral in self.peripherals.iter() {
            peripheral.after_wakeup(sleep_mode);
        }
    }

    /// Returns the lowest possible power mode allowed by the registered clients.
    pub fn lowest_sleep_mode(&self) -> u32 {
        self.peripherals.iter().fold(0, |prev, peripheral| {
            prev.max(peripheral.lowest_sleep_mode())
        })
    }
}
