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

pub trait ResourceManager {
    fn enable_resource(&self, resource_id: u32);

    fn disable_resource(&self, resource_id: u32);
}

pub struct Resource<'a> {
    id: Cell<u32>,
    next: ListLink<'a, Resource<'a>>,
    ref_count: Cell<u32>,
}

impl<'a> ListNode<'a, Resource<'a>> for Resource<'a> {
    fn next(&self) -> &'a ListLink<Resource<'a>> { &self.next }
}

impl<'a> Resource<'a> {
    pub const fn new(id: u32) -> Resource<'a> {
        Resource {
            id: Cell::new(id),
            next: ListLink::empty(),
            ref_count: Cell::new(0),
        }
    }

    pub fn inc_ref_count(&self) {
        self.ref_count.set(self.ref_count.get() + 1);
    }

    pub fn dec_ref_count(&self) {
        self.ref_count.set(self.ref_count.get() - 1);
    }
}

pub struct PowerManager<'a, T: ResourceManager> {
    resources: List<'a, Resource<'a>>,
    peripherals: List<'a, Peripheral<'a>>,
    resource_manager: T,
}

impl<'a, T: ResourceManager> PowerManager<'a, T> {
    pub const fn new(resource_manager: T) -> PowerManager<'a, T> {
        PowerManager {
            resources: List::new(),
            peripherals: List::new(),
            resource_manager,
        }
    }

    pub fn register_resource(&self, resource: &'a Resource<'a>) {
        self.resources.push_head(resource);
    }

    pub fn request_resource(&self, resource_id: u32) {
        let resource = self.resources
            .iter()
            .find(| r| r.id.get() == resource_id)
            .expect("Resource not found.");

        if resource.ref_count.get() == 0 {
            self.resource_manager.enable_resource(resource_id);
        }

        resource.inc_ref_count();
    }

    pub fn release_resource(&self, resource_id: u32) {
        let resource = self.resources
            .iter()
            .find(|r | r.id.get() == resource_id)
            .expect("Resource not found.");

        if resource.ref_count.get() > 0 {
            resource.dec_ref_count()
        }

        if resource.ref_count.get() == 0 {
            self.resource_manager.disable_resource(resource_id);
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
