//! SETUP Code from Texas Instruments.
//! This file was converted using corrode (https://github.com/jameysharp/corrode)
//! into rust from c.
//!
//! It sets up the recharge algorithm which is necessary when
//! progressing between deep sleep and regular sleep/active modes.
//!
//! Source:
//!     - https://github.com/contiki-os/cc26xxware/blob/e816e3508b87744186acae2c5f792ad378836ae3/driverlib/sys_ctrl.c

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

#[no_mangle]
pub unsafe extern fn norom_aonbat_mon_temperature_get_deg_c() -> i32 {
    let signed_temp = *((0x40095000i32 + 0x30i32) as (*mut usize)) as (i32) << 32i32 - 9i32 - 8i32 >> 32i32 - 9i32 - 8i32;
    let voltage_slope = *((0x50001000i32 + 0x30ci32) as (*mut u8)) as (i8);
    let temp_correction = voltage_slope as (i32) * (*((0x40095000i32 + 0x28i32) as (*mut usize)) as (i32) - 0x300i32) >> 4i32;
    signed_temp - temp_correction + 0x80i32 >> 8i32
}

#[derive(Copy)]
#[repr(C)]
pub struct Struct1 {
    pub pd_time: u32,
    pub pd_recharge_period: u16,
    pub pd_state: u8,
    pub pd_temp: i8,
}

impl Clone for Struct1 {
    fn clone(&self) -> Self { *self }
}

static mut POWER_QUAL_GLOBALS
: Struct1
= Struct1 {
    pd_time: 0u32,
    pd_recharge_period: 0u16,
    pd_state: 0u8,
    pd_temp: 0i8
};

unsafe extern fn setup_sign_extend_vddr_trim_value(
    ui32vddr_trim_val: u32
) -> i32 {
    let mut i32signed_vddr_val: i32 = ui32vddr_trim_val as (i32);
    if i32signed_vddr_val > 0x15i32 {
        i32signed_vddr_val = i32signed_vddr_val - 0x20i32;
    }
    i32signed_vddr_val
}

#[no_mangle]
pub unsafe extern fn before_power_down(
    xosc_power_mode: u32
) {
    let mut cur_temp: i32;
    let mut shifted_temp: i32;
    let mut delta_vddr_sleep_trim: i32;
    let mut vddr_trim_sleep: i32;
    //let mut vddr_trim_actve: i32;
    let mut diff_vddr_active_sleep: i32;
    //let mut ccfg_mode_conf_reg: u32;
    let mut cur_state: u32;
    //let mut prcm_ram_retention: u32;
    let mut di : u32;
    let mut dii : u32;
    let mut ti : u32;
    //let mut cd : u32;
    //let mut cl : u32;
    let mut load : u32;
    //let mut k : u32;
    //let mut vddr_cap: u32;
    let mut new_recharge_period: u32;
    let mut per_e: u32;
    let mut per_m: u32;
    //let mut p_lookup_table: *const u32;

    if *((0x40090000i32 + 0x0i32) as (*mut usize)) & 0x2usize != 0 {
        POWER_QUAL_GLOBALS.pd_state = 8u8;
        *((0x40091000i32 + 0x30i32) as (*mut usize)) = 0xa4fdfdusize;
    } else {
        cur_temp = norom_aonbat_mon_temperature_get_deg_c();
        cur_state = 0u32;
        let ccfg_mode_conf_reg = *((0x50003000i32 + 0xfb4i32) as (*mut usize)) as (u32);
        delta_vddr_sleep_trim = (ccfg_mode_conf_reg as (i32) << 32i32 - 4i32 - 28i32 >> 32i32 - 4i32) + 1i32;
        if ccfg_mode_conf_reg & 0x200000u32 == 0u32 {
            let mut tc_delta: i32 = 62i32 - cur_temp >> 3i32;
            if tc_delta > 8i32 {
                tc_delta = 8i32;
            }
            if tc_delta > delta_vddr_sleep_trim {
                delta_vddr_sleep_trim = tc_delta;
            }
        }
        vddr_trim_sleep = setup_sign_extend_vddr_trim_value(
            ((*((0x50001000i32 + 0x2b8i32) as (*mut usize)) & 0x1f000000usize) >> 24i32) as (u32)
        );
        let vddr_trim_actve = setup_sign_extend_vddr_trim_value(
            ((*((0x50001000i32 + 0x13ci32) as (*mut usize)) & 0x1f0000usize) >> 16i32) as (u32)
        );
        vddr_trim_sleep = vddr_trim_sleep + delta_vddr_sleep_trim;
        if vddr_trim_sleep > 21i32 {
            vddr_trim_sleep = 21i32;
        }
        if vddr_trim_sleep < -10i32 {
            vddr_trim_sleep = -10i32;
        }
        *((0x40086200i32 + 0x60i32 + 0x7i32 * 2i32) as (*mut u16)) = (0x1fi32 << 8i32 | vddr_trim_sleep << 0i32 & 0x1fi32) as (u16);
        let prcm_ram_retention = *((0x40082000i32 + 0x224i32) as (*mut usize)) as (u32);
        if prcm_ram_retention & 0x3u32 != 0 {
            cur_state = cur_state | 1u32;
        }
        if prcm_ram_retention & 0x4u32 != 0 {
            cur_state = cur_state | 2u32;
        }
        if xosc_power_mode != 0u32 {
            cur_state = cur_state | 4u32;
        }
        if cur_temp - POWER_QUAL_GLOBALS.pd_temp as (i32) >= 5i32 || cur_state != POWER_QUAL_GLOBALS.pd_state as (u32) {
            shifted_temp = cur_temp - 15i32;
            let p_lookup_table = (0x50001000i32 + 0x39ci32) as (*mut u32) as (*const u32);
            di = 0u32;
            ti = 0u32;
            if shifted_temp >= 0i32 {
                shifted_temp = shifted_temp + (shifted_temp << 4i32);
                ti = (shifted_temp >> 8i32) as (u32);
                if ti > 7u32 {
                    ti = 7u32;
                }
                dii = ti;
                if dii > 6u32 {
                    dii = 6u32;
                }
                let cd = (*p_lookup_table.offset(
                    dii.wrapping_add(1u32) as (isize)
                )).wrapping_sub(
                    *p_lookup_table.offset(dii as (isize))
                );
                di = cd & 0xffu32;
                if cur_state & 4u32 != 0 {
                    di = di.wrapping_add(cd >> 8i32 & 0xffu32);
                }
                if cur_state & 2u32 != 0 {
                    di = di.wrapping_add(cd >> 16i32 & 0xffu32);
                }
                if cur_state & 1u32 != 0 {
                    di = di.wrapping_add(cd >> 24i32 & 0xffu32);
                }
            }
            let cl = *p_lookup_table.offset(ti as (isize));
            load = cl & 0xffu32;
            if cur_state & 4u32 != 0 {
                load = load.wrapping_add(cl >> 8i32 & 0xffu32);
            }
            if cur_state & 2u32 != 0 {
                load = load.wrapping_add(cl >> 16i32 & 0xffu32);
            }
            if cur_state & 1u32 != 0 {
                load = load.wrapping_add(cl >> 24i32 & 0xffu32);
            }
            load = load.wrapping_add(
                di.wrapping_mul(
                    (shifted_temp as (u32)).wrapping_sub(ti << 8i32)
                ).wrapping_add(
                    128u32
                ) >> 8i32
            );
            diff_vddr_active_sleep = vddr_trim_actve - vddr_trim_sleep;
            if diff_vddr_active_sleep < 1i32 {
                diff_vddr_active_sleep = 1i32;
            }
            let k = (diff_vddr_active_sleep * 52i32) as (u32);
            let vddr_cap = (ccfg_mode_conf_reg & 0xffu32) >> 0i32;
            new_recharge_period = vddr_cap.wrapping_mul(k).wrapping_div(load);
            if new_recharge_period > 0xffffu32 {
                new_recharge_period = 0xffffu32;
            }
            POWER_QUAL_GLOBALS.pd_recharge_period = new_recharge_period as (u16);
            if cur_temp > 127i32 {
                cur_temp = 127i32;
            }
            if cur_temp < -128i32 {
                cur_temp = -128i32;
            }
            POWER_QUAL_GLOBALS.pd_temp = cur_temp as (i8);
            POWER_QUAL_GLOBALS.pd_state = cur_state as (u8);
        }
        POWER_QUAL_GLOBALS.pd_time = *((0x40092000i32 + 0x8i32) as (*mut usize)) as (u32);
        per_e = 0u32;
        per_m = POWER_QUAL_GLOBALS.pd_recharge_period as (u32);
        if per_m < 31u32 {
            per_m = 31u32;
            POWER_QUAL_GLOBALS.pd_recharge_period = 31u16;
        }
        'loop45: loop {
            if !(per_m > 511u32) {
                break;
            }
            per_m = per_m >> 1i32;
            per_e = per_e.wrapping_add(1u32);
        }
        per_m = per_m.wrapping_sub(15u32) >> 4i32;
        *((0x40091000i32 + 0x30i32) as (*mut usize)) = (0x80a4e700u32 | per_m << 3i32 | per_e << 0i32) as (usize);
        *((0x40091000i32 + 0x34i32) as (*mut usize)) = 0usize;
    }
}

#[no_mangle]
pub unsafe extern fn after_power_down() {
    let mut cur_temp: i32;
    // let mut longest_recharge_period: u32;
    let mut delta_time: u32;
    let mut new_recharge_period: u32;

    let longest_recharge_period = ((*((0x40091000i32 + 0x34i32) as (*mut usize)) & 0xffffusize) >> 0i32) as (u32);
    if longest_recharge_period != 0u32 {
        cur_temp = norom_aonbat_mon_temperature_get_deg_c();
        if cur_temp < POWER_QUAL_GLOBALS.pd_temp as (i32) {
            if cur_temp < -128i32 {
                cur_temp = -128i32;
            }
            POWER_QUAL_GLOBALS.pd_temp = cur_temp as (i8);
        }
        if longest_recharge_period < POWER_QUAL_GLOBALS.pd_recharge_period as (u32) {
            POWER_QUAL_GLOBALS.pd_recharge_period = longest_recharge_period as (u16);
        } else {
            delta_time = (*((0x40092000i32 + 0x8i32) as (*mut usize))).wrapping_sub(
                POWER_QUAL_GLOBALS.pd_time as (usize)
            ).wrapping_add(
                2usize
            ) as (u32);
            if delta_time > 31u32 {
                delta_time = 31u32;
            }
            new_recharge_period = (POWER_QUAL_GLOBALS.pd_recharge_period as (u32)).wrapping_add(
                longest_recharge_period.wrapping_sub(
                    POWER_QUAL_GLOBALS.pd_recharge_period as (u32)
                ) >> (delta_time >> 1i32)
            );
            if new_recharge_period > 0xffffu32 {
                new_recharge_period = 0xffffu32;
            }
            POWER_QUAL_GLOBALS.pd_recharge_period = new_recharge_period as (u16);
        }
    }
}
