//! SETUP Code from Texas Instruments.
//! This file was converted using corrode (https://github.com/jameysharp/corrode)
//! into rust from c.
//!
//! Source:
//!     - https://github.com/contiki-os/cc26xxware/blob/e816e3508b87744186acae2c5f792ad378836ae3/driverlib/ddi.c

/*
 * Copyright (c) 2015, Texas Instruments Incorporated - http://www.ti.com/
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 * 3. Neither the name of the copyright holder nor the names of its
 *    contributors may be used to endorse or promote products derived
 *    from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * ``AS IS'' AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS
 * FOR A PARTICULAR PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL THE
 * COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT,
 * INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
 * STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED
 * OF THE POSSIBILITY OF SUCH DAMAGE.
*/

unsafe extern "C" fn aux_adi_ddi_safe_write(n_addr: u32, n_data: u32, n_size: u32) {
    //let mut bIrqEnabled : bool = CPUcpsid() == 0;

    'loop1: loop {
        if !(*((0x400c8000i32 + 0x0i32) as (*mut usize)) == 0) {
            break;
        }
    }
    if n_size == 2u32 {
        *(n_addr as (*mut u16)) = n_data as (u16);
    } else if n_size == 1u32 {
        *(n_addr as (*mut u8)) = n_data as (u8);
    } else {
        *(n_addr as (*mut usize)) = n_data as (usize);
    }
    *((0x400c8000i32 + 0x0i32) as (*mut usize)) = 1usize;

    /*if bIrqEnabled {
        CPUcpsie();
    }*/
}

unsafe extern "C" fn aux_adi_ddi_safe_read(n_addr: u32, n_size: u32) -> u32 {
    let mut ret: u32;
    //let mut bIrqEnabled: bool = CPUcpsid() == 0;
    'loop1: loop {
        if !(*((0x400c8000i32 + 0x0i32) as (*mut usize)) == 0) {
            break;
        }
    }
    if n_size == 2u32 {
        ret = *(n_addr as (*mut u16)) as (u32);
    } else if n_size == 1u32 {
        ret = *(n_addr as (*mut u8)) as (u32);
    } else {
        ret = *(n_addr as (*mut usize)) as (u32);
    }
    *((0x400c8000i32 + 0x0i32) as (*mut usize)) = 1usize;
    /*if bIrqEnabled {
        CPUcpsie();
    }*/
    ret
}

pub unsafe extern "C" fn ddi32reg_write(ui32base: u32, ui32reg: u32, ui32val: u32) {
    aux_adi_ddi_safe_write(ui32base.wrapping_add(ui32reg), ui32val, 4u32);
}

#[allow(unused)]
pub unsafe extern "C" fn ddi16bit_write(
    ui32base: u32,
    ui32reg: u32,
    mut ui32mask: u32,
    ui32wr_data: u32,
) {
    let mut ui32reg_addr: u32;
    ui32reg_addr = ui32base
        .wrapping_add(ui32reg << 1i32)
        .wrapping_add(0x200u32);
    if ui32mask & 0xffff0000u32 != 0 {
        ui32reg_addr = ui32reg_addr.wrapping_add(4u32);
        ui32mask = ui32mask >> 16i32;
    }
    let ui32data: u32 = if ui32wr_data != 0 { ui32mask } else { 0x0u32 };
    aux_adi_ddi_safe_write(ui32reg_addr, ui32mask << 16i32 | ui32data, 4u32);
}

pub unsafe extern "C" fn ddi16bitfield_write(
    ui32base: u32,
    ui32reg: u32,
    mut ui32mask: u32,
    mut ui32shift: u32,
    ui32data: u16,
) {
    let mut ui32reg_addr: u32;
    ui32reg_addr = ui32base
        .wrapping_add(ui32reg << 1i32)
        .wrapping_add(0x200u32);
    if ui32shift >= 16u32 {
        ui32shift = ui32shift.wrapping_sub(16u32);
        ui32reg_addr = ui32reg_addr.wrapping_add(4u32);
        ui32mask = ui32mask >> 16i32;
    }
    let ui32wr_data: u32 = (ui32data as (i32) << ui32shift) as (u32);
    aux_adi_ddi_safe_write(ui32reg_addr, ui32mask << 16i32 | ui32wr_data, 4u32);
}

#[allow(unused)]
pub unsafe extern "C" fn ddi16bit_read(
    mut ui32base: u32,
    mut ui32reg: u32,
    mut ui32mask: u32,
) -> u16 {
    let mut ui32reg_addr: u32;
    let mut ui16data: u16;
    ui32reg_addr = ui32base.wrapping_add(ui32reg).wrapping_add(0x0u32);
    if ui32mask & 0xffff0000u32 != 0 {
        ui32reg_addr = ui32reg_addr.wrapping_add(2u32);
        ui32mask = ui32mask >> 16i32;
    }
    ui16data = aux_adi_ddi_safe_read(ui32reg_addr, 2u32) as (u16);
    ui16data = (ui16data as (u32) & ui32mask) as (u16);
    ui16data
}

pub unsafe extern "C" fn ddi16bitfield_read(
    ui32base: u32,
    ui32reg: u32,
    mut ui32mask: u32,
    mut ui32shift: u32,
) -> u16 {
    let mut ui32reg_addr: u32;
    let mut ui16data: u16;
    ui32reg_addr = ui32base.wrapping_add(ui32reg).wrapping_add(0x0u32);
    if ui32shift >= 16u32 {
        ui32shift = ui32shift.wrapping_sub(16u32);
        ui32reg_addr = ui32reg_addr.wrapping_add(2u32);
        ui32mask = ui32mask >> 16i32;
    }
    ui16data = aux_adi_ddi_safe_read(ui32reg_addr, 2u32) as (u16);
    ui16data = (ui16data as (u32) & ui32mask) as (u16);
    ui16data = (ui16data as (i32) >> ui32shift) as (u16);
    ui16data
}
