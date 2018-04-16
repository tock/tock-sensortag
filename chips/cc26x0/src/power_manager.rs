//! Power Manager
//!
//! This construct facilitates the management of different hardware resources on
//! on a chip related to power. By keeping track of the number of references to a certain resource,
//! we can decide if the resource needs to be powered on or not.
//!
//! For a resource to be tracked it needs to be registered with the power manager.
//! The power manager then controls all registered resources through a resource manager.
//! The resource manager knows the hardware specific details of how to enable/disable the resources.
//!
//! Following is an example from the cc26xx family of microcontrollers that shows how different
//! power regions can be controlled through the power manager.
//!
//! ``` rust
//! /// All requests to use a certain power region goes through this power manager.
//! pub static mut PM: PowerManager<RegionManager> = PowerManager::new(RegionManager);
//!
//! /// Used to power on/off different power regions.
//! pub struct RegionManager;
//!
//! impl ResourceManager for RegionManager {
//!    fn enable_resource(&self, resource_id: u32) {
//!        let domain = PowerDomain::from(resource_id);
//!        Power::enable_domain(domain);
//!    }
//!
//!    fn disable_resource(&self, resource_id: u32) {
//!        let domain = PowerDomain::from(resource_id);
//!        Power::disable_domain(domain);
//!    }
//! }
//!
//! /// Registers all resources we want the power manager to keep track off.
//! pub unsafe fn init() {
//!    for pwr_region in POWER_REGIONS.iter() {
//!        // pwer_region is of type Resource and has a unique id associated with it.
//!        PM.register_resource(&pwr_region);
//!    }
//! }
//! ```
//!
//! A peripheral that wants to use a certain power region then simply requests the resource through
//! the power manager and releases it once it is done.
//!
//! ``` rust
//! PM.request_resource(power_region_id);
//!
//! // Do some work which requires the power region to be on.
//!
//! PM.release_resource(power_region_id);
//! ```

use core::cell::Cell;
use kernel::common::{List, ListLink, ListNode};

/// Manages the details of how resources are enabled and disabled.
pub trait ResourceManager {
    fn enable_resource(&self, resource_id: u32);
    fn disable_resource(&self, resource_id: u32);
}

/// A resource is a hardware component that is shared between several system components.
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

    /// Increment the number of references to this resource.
    pub fn inc_ref_count(&self) {
        self.ref_count.set(self.ref_count.get() + 1);
    }

    /// Decrement the number of references to this resource.
    pub fn dec_ref_count(&self) {
        self.ref_count.set(self.ref_count.get() - 1);
    }
}

/// Keeps track of different resources and controls wheter they should be powered on or not.
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

    /// Registers a resource with the power manager to keep track of how many times it is referenced.
    pub fn register_resource(&self, resource: &'a Resource<'a>) {
        self.resources.push_head(resource);
    }

    /// Tells the power manager that we need a certain resource to be enabled.
    ///
    /// A resource is powered on when we have at least one active request.
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

    /// Tells the power manager that we are done with a resource.
    ///
    /// A resource is powered off when no one needs it services anymore.
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
}
