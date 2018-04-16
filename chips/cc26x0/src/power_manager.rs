use core::cell::Cell;
use kernel::common::{List, ListLink, ListNode};

/*
pub trait ResourceClient<'a> {
    fn before_sleep(&self);
    fn after_wakeup(&self);
    fn lowest_sleep_mode(&self) -> u32;
}
*/

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
    resource_manager: T,
}

impl<'a, T: ResourceManager> PowerManager<'a, T> {
    pub const fn new(resource_manager: T) -> PowerManager<'a, T> {
        PowerManager {
            resources: List::new(),
            resource_manager,
        }
    }

    pub fn add_resource(&self, resource: &'a Resource<'a>) {
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

    /*
    pub fn prepare_for_sleep(&self, clients: &[PowerClient]) {
        clients.into_iter().map(|c| { c.before_sleep(); });
    }

    pub fn after_wakeup(&self, clients: &[PowerClient]) {
        clients.into_iter().map(|c| { c.after_wakeup(); });
    }
    */
}
