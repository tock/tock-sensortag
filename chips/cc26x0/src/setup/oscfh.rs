use setup::adi;
use setup::ddi;

pub unsafe extern "C" fn clock_source_set(mut ui32src_clk: u32, mut ui32osc: u32) {
    if ui32src_clk & 0x1u32 != 0 {
        ddi::ddi16bitfield_write(0x400ca000u32, 0x0u32, 0x1u32, 0u32, ui32osc as (u16));
    }
    if ui32src_clk & 0x2u32 != 0 {
        ddi::ddi16bitfield_write(0x400ca000u32, 0x0u32, 0x2u32, 1u32, ui32osc as (u16));
    }
    if ui32src_clk & 0x4u32 != 0 {
        ddi::ddi16bitfield_write(0x400ca000u32, 0x0u32, 0xcu32, 2u32, ui32osc as (u16));
    }
}

pub unsafe extern "C" fn clock_source_get(mut ui32src_clk: u32) -> u32 {
    let mut ui32clock_source: u32;
    if ui32src_clk == 0x4u32 {
        ui32clock_source = ddi::ddi16bitfield_read(0x400ca000u32, 0x34u32, 0x60000000u32, 29u32) as (u32);
    } else {
        ui32clock_source = ddi::ddi16bitfield_read(0x400ca000u32, 0x34u32, 0x10000000u32, 28u32) as (u32);
    }
    ui32clock_source
}

#[allow(unused)]
unsafe extern "C" fn source_ready() -> bool {
    (if ddi::ddi16bitfield_read(0x400ca000u32, 0x34u32, 0x1u32, 0u32) != 0 {
        1i32
    } else {
        0i32
    }) != 0
}

#[derive(Copy)]
#[repr(C)]
pub struct RomFuncTable {
    pub _crc32: unsafe extern "C" fn(*mut u8, u32, u32) -> u32,
    pub _flash_get_size: unsafe extern "C" fn() -> u32,
    pub _get_chip_id: unsafe extern "C" fn() -> u32,
    pub _reserved_location1: unsafe extern "C" fn(u32) -> u32,
    pub _reserved_location2: unsafe extern "C" fn() -> u32,
    pub _reserved_location3: unsafe extern "C" fn(*mut u8, u32, u32) -> u32,
    pub _reset_device: unsafe extern "C" fn(),
    pub _fletcher32: unsafe extern "C" fn(*mut u16, u16, u16) -> u32,
    pub _min_value: unsafe extern "C" fn(*mut u32, u32) -> u32,
    pub _max_value: unsafe extern "C" fn(*mut u32, u32) -> u32,
    pub _mean_value: unsafe extern "C" fn(*mut u32, u32) -> u32,
    pub _stand_deviation_value: unsafe extern "C" fn(*mut u32, u32) -> u32,
    pub _reserved_location4: unsafe extern "C" fn(u32),
    pub _reserved_location5: unsafe extern "C" fn(u32),
    pub hfsource_safe_switch: unsafe extern "C" fn(),
    pub _select_comp_ainput: unsafe extern "C" fn(u8),
    pub _select_comp_aref: unsafe extern "C" fn(u8),
    pub _select_adccomp_binput: unsafe extern "C" fn(u8),
    pub _select_comp_bref: unsafe extern "C" fn(u8),
}

impl Clone for RomFuncTable {
    fn clone(&self) -> Self {
        *self
    }
}

/*
    In order to switch oscillator sources we need to call a ROM
    function (proprietary), due to a set of undocumented restrictions.
*/
pub unsafe extern "C" fn source_switch() {
    adi::safe_hapi_void((*(0x10000048i32 as (*mut RomFuncTable))).hfsource_safe_switch);
}
