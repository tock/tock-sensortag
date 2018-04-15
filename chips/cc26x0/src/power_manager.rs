use core::cell::Cell;
use kernel::common::{List, ListLink, ListNode};

/*
pub trait PowerClient<'a> {
    fn before_sleep(&self);

    fn after_wakeup(&self);

    fn lowest_sleep_mode(&self) -> u32;
}
*/

pub trait ResourceHandler {
    fn power_on_resource(&self, resource_id: u32);

    fn power_off_resource(&self, resource_id: u32);
}

pub struct PowerResource<'a> {
    id: Cell<u32>,
    next: ListLink<'a, PowerResource<'a>>,
    ref_count: Cell<u32>,
}

impl<'a> ListNode<'a, PowerResource<'a>> for PowerResource<'a> {
    fn next(&self) -> &'a ListLink<PowerResource<'a>> { &self.next }
}

impl<'a> PowerResource<'a> {
    pub const fn new(id: u32) -> PowerResource<'a> {
        PowerResource {
            id: Cell::new(id),
            next: ListLink::empty(),
            ref_count: Cell::new(0),
        }
    }

    pub fn inc_ref_count(&self) { self.ref_count.set(self.ref_count.get() + 1); }

    pub fn dec_ref_count(&self) { self.ref_count.set(self.ref_count.get() - 1); }
}

pub struct PowerManager<'a, T: ResourceHandler> {
    resources: List<'a, PowerResource<'a>>,
    resource_handler: T,
}

impl<'a, T: ResourceHandler> PowerManager<'a, T> {
    pub const fn new(resource_handler: T) -> PowerManager<'a, T> {
        PowerManager {
            resources: List::new(),
            resource_handler,
        }
    }

    pub fn add_resource(&self, resource: &'a PowerResource<'a>) {
        self.resources.push_head(resource);
    }

    pub fn request_resource(&self, resource_id: u32) {
        let resource = self.resources
            .iter()
            .find(| r| { r.id.get() == resource_id })
            .expect("Resource not found.");

        if resource.ref_count.get() == 0 { self.resource_handler.power_on_resource(resource_id); }

        resource.inc_ref_count();
    }

    pub fn release_resource(&self, resource_id: u32) {
        let resource = self.resources
            .iter()
            .find(|r | { r.id.get() == resource_id })
            .expect("Resource not found.");

        if resource.ref_count.get() > 0 { resource.dec_ref_count() }

        if resource.ref_count.get() == 0 { self.resource_handler.power_off_resource(resource_id); }
    }

    /*
    pub fn prepare_for_sleep(&self, clients: &[PowerClient]) {
        clients.into_iter().map(|c| { c.before_sleep(); });
    }

    pub fn after_wakeup(&self, clients: &[PowerClient]) {
        clients.into_iter().map(|c| { c.after_wakeup(); });
    }
    */
}
