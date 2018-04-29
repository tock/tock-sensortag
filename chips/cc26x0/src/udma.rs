///
/// # Micro Direct Memory Access for the TI CC26x0 Microcontroller
///

use kernel;
use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};

pub const UDMA_BASE: usize = 0x4002_0000;
pub const UDMA_CONFIG_BASE: usize = 0x2000_0400;

#[repr(C)]
struct Registers {
    status: ReadWrite<u32>,
    cfg: ReadWrite<u32>,
    ctrl: ReadWrite<u32>,
    alt_ctrl: ReadWrite<u32>,
    wait_on_req: ReadWrite<u32>,
    soft_req: ReadWrite<u32>,
    set_burst: ReadWrite<u32>,
    clear_burst: ReadWrite<u32>,
    set_req_mask: ReadWrite<u32>,
    clear_req_mask: ReadWrite<u32>,
    set_channel_en: ReadWrite<u32>,
    clear_channel_en: ReadWrite<u32>,
    set_chnl_pri_alt: ReadWrite<u32>,
    clear_chnl_pri_alt: ReadWrite<u32>,
    set_chnl_priority: ReadWrite<u32>,
    clear_chnl_priority: ReadWrite<u32>,
    error: ReadWrite<u32>,
    req_done: ReadWrite<u32>,
    done_mask: ReadWrite<u32>
}