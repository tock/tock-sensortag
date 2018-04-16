use core::cell::Cell;
use kernel::common::{List, ListLink, ListNode};

pub trait PowerClient {
    fn before_sleep(&self, sleep_mode: u32);
    fn after_wakeup(&self, sleep_mode: u32);
    fn lowest_sleep_mode(&self) -> u32;
}

pub struct Peripheral<'a> {
    client: Cell<&'a PowerClient>,
    next: ListLink <'a, Peripheral<'a>>
}

impl<'a> Peripheral<'a> {
    pub fn new(client: &'a PowerClient) -> Peripheral {
        Peripheral {
            client: Cell::new(client),
            next: ListLink::empty(),
        }
    }

    pub fn lowest_sleep_mode(&self) -> u32 {
        self.client.get().lowest_sleep_mode()
    }

    pub fn before_sleep(&self, sleep_mode: u32) {
        self.client.get().before_sleep(sleep_mode);
    }

    pub fn after_wakeup(&self, sleep_mode: u32) {
        self.client.get().after_wakeup(sleep_mode);
    }
}

impl<'a> ListNode<'a, Peripheral<'a>> for Peripheral<'a> {
    fn next(&self) -> &'a ListLink<Peripheral<'a>> { &self.next }
}

pub struct PeripheralManager<'a> {
    peripherals: List<'a, Peripheral<'a>>,
}

impl<'a> PeripheralManager<'a> {
    pub const fn new() -> PeripheralManager<'a> {
        PeripheralManager {
            peripherals: List::new(),
        }
    }

    pub fn register_peripheral(&self, peripheral: &'a Peripheral<'a>) {
        self.peripherals.push_head(peripheral);
    }

    pub fn before_sleep(&self, sleep_mode: u32) {
        self.peripherals.iter().map(|p| p.before_sleep(sleep_mode));
    }

    pub fn after_wakeup(&self, sleep_mode: u32) {
        self.peripherals.iter().map(|p| p.after_wakeup(sleep_mode));
    }

    pub fn lowest_sleep_mode(&self) -> u32 {
        self.peripherals.iter().fold(0, |prev, peripheral| {
            prev.max(peripheral.lowest_sleep_mode())
        })
    }
}
