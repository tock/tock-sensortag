//! SETUP Code from Texas Instruments.
//! This file was converted using corrode (https://github.com/jameysharp/corrode)
//! into rust from c.
//!
//! It copies and trims several values from the factory & customer configuration
//! areas into their appropriate places (e.g trims the auxiliary voltages).
//!
//! Source:
//!     - https://github.com/contiki-os/cc26xxware/blob/e816e3508b87744186acae2c5f792ad378836ae3/driverlib/setup_rom.c
//!     - https://github.com/contiki-os/cc26xxware/blob/e816e3508b87744186acae2c5f792ad378836ae3/driverlib/setup.c

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

#[allow(unused_variables, unused_mut)]
pub mod oscfh;

#[allow(unused_variables, unused_mut)]
mod ddi;

#[allow(unused_variables, unused_mut)]
mod adi;

pub mod recharge;

pub fn perform() {
    unsafe { setup_trim_device() }
}

pub unsafe extern "C" fn setup_trim_device() {
    let mut ui32fcfg1revision: u32;
    let mut ui32aon_sys_resetctl: u32;
    ui32fcfg1revision = *((0x50001000i32 + 0x31ci32) as (*mut usize)) as (u32);
    if ui32fcfg1revision == 0xffffffffu32 {
        ui32fcfg1revision = 0u32;
    }

    *(((0x40030000i32 + 0x24i32) as (usize) & 0xf0000000usize | 0x2000000usize
        | ((0x40030000i32 + 0x24i32) as (usize) & 0xfffffusize) << 5i32
        | (1i32 << 2i32) as (usize)) as (*mut usize)) = 0usize;
    *((0x400c6000i32 + 0x5ci32) as (*mut usize)) = 0x1usize;
    *(((0x40082000i32 + 0x110i32) as (usize) & 0xf0000000usize | 0x2000000usize
        | ((0x40082000i32 + 0x110i32) as (usize) & 0xfffffusize) << 5i32
        | (2i32 << 2i32) as (usize)) as (*mut usize)) = 1usize;
    setup_set_cache_mode_according_to_ccfg_setting();
    if *(((0x40094000i32 + 0xci32) as (usize) & 0xf0000000usize | 0x2000000usize
        | ((0x40094000i32 + 0xci32) as (usize) & 0xfffffusize) << 5i32
        | (0i32 << 2i32) as (usize)) as (*mut usize)) == 0
    {
        trim_after_cold_reset_wakeup_from_shut_down_wakeup_from_power_down();
    } else if *(((0x40090000i32 + 0x8i32) as (usize) & 0xf0000000usize | 0x2000000usize
        | ((0x40090000i32 + 0x8i32) as (usize) & 0xfffffusize) << 5i32
        | (0i32 << 2i32) as (usize)) as (*mut usize)) == 0
    {
        trim_after_cold_reset_wakeup_from_shut_down(ui32fcfg1revision);
        trim_after_cold_reset_wakeup_from_shut_down_wakeup_from_power_down();
    } else {
        trim_after_cold_reset();
        trim_after_cold_reset_wakeup_from_shut_down(ui32fcfg1revision);
        trim_after_cold_reset_wakeup_from_shut_down_wakeup_from_power_down();
    }
    *((0x40082000i32 + 0x18ci32) as (*mut usize)) = 0usize;
    *((0x40030000i32 + 0x2048i32) as (*mut usize)) = *((0x40030000i32 + 0x2048i32) as (*mut usize))
        & !0xfff0000i32 as (usize)
        | (0x139i32 << 16i32) as (usize);
    if (*((0x40090000i32 + 0x4i32) as (*mut usize)) & (0x2000i32 | 0x1000i32) as (usize)) >> 12i32
        == 1usize
    {
        ui32aon_sys_resetctl = (*((0x40090000i32 + 0x4i32) as (*mut usize))
            & !(0x2000000i32 | 0x1000000i32 | 0x20000i32 | 0x10000i32) as (usize))
            as (u32);
        *((0x40090000i32 + 0x4i32) as (*mut usize)) =
            (ui32aon_sys_resetctl | 0x20000u32) as (usize);
        *((0x40090000i32 + 0x4i32) as (*mut usize)) = ui32aon_sys_resetctl as (usize);
    }
    'loop9: loop {
        if *(((0x40034000i32 + 0x0i32) as (usize) & 0xf0000000usize | 0x2000000usize
            | ((0x40034000i32 + 0x0i32) as (usize) & 0xfffffusize) << 5i32
            | (3i32 << 2i32) as (usize)) as (*mut usize)) == 0
        {
            break;
        }
    }
}

unsafe extern "C" fn trim_after_cold_reset_wakeup_from_shut_down_wakeup_from_power_down() {}

unsafe extern "C" fn aonwucjtag_power_off() {
    *((0x40091000i32 + 0x40i32) as (*mut usize)) = 0usize;
}

unsafe extern "C" fn trim_after_cold_reset_wakeup_from_shut_down(mut ui32fcfg1revision: u32) {
    let mut ccfg_mode_conf_reg: u32;
    let mut mp1rev: u32;
    *((0x40091000i32 + 0x10i32) as (*mut usize)) = 0x1usize;
    'loop1: loop {
        if !(*(((0x40091000i32 + 0x14i32) as (usize) & 0xf0000000usize | 0x2000000usize
            | ((0x40091000i32 + 0x14i32) as (usize) & 0xfffffusize) << 5i32
            | (5i32 << 2i32) as (usize)) as (*mut usize)) == 0)
        {
            break;
        }
    }
    *((0x400c6000i32 + 0x0i32) as (*mut usize)) = (0x40i32 | 0x80i32) as (usize);
    if *((0x50003000i32 + 0xfb0i32) as (*mut usize)) & 0x2usize == 0usize {
        *((0x40086200i32 + 0x40i32 + 0xbi32 * 2i32) as (*mut u8)) =
            (0xf0usize | *((0x50003000i32 + 0xfaci32) as (*mut usize)) >> 16i32) as (u8);
    } else {
        *((0x40086200i32 + 0x40i32 + 0xbi32 * 2i32) as (*mut u8)) = 0x72u8;
    }
    aonwucjtag_power_off();
    ccfg_mode_conf_reg = *((0x50003000i32 + 0xfb4i32) as (*mut usize)) as (u32);
    setup_after_cold_reset_wakeup_from_shut_down_cfg1(ccfg_mode_conf_reg);
    setup_after_cold_reset_wakeup_from_shut_down_cfg2(ui32fcfg1revision, ccfg_mode_conf_reg);
    mp1rev = (*((0x50001000i32 + 0x314i32) as (*mut usize)) & 0xffffusize) as (u32);
    if mp1rev < 527u32 {
        let mut vtrim_bod: u32 =
            (*((0x50001000i32 + 0x2bci32) as (*mut usize)) >> 24i32 & 0xfusize) as (u32);
        let mut vtrim_udig: u32 =
            (*((0x50001000i32 + 0x2bci32) as (*mut usize)) >> 16i32 & 0xfusize) as (u32);
        if vtrim_bod > 0u32 {
            vtrim_bod = vtrim_bod.wrapping_sub(1u32);
        }
        if vtrim_udig != 7u32 {
            if vtrim_udig == 6u32 {
                vtrim_udig = 7u32;
            } else {
                vtrim_udig = vtrim_udig.wrapping_add(2u32) & 0xfu32;
            }
        }
        *((0x40086000i32 + 0x2i32) as (*mut u8)) = (vtrim_udig << 4i32 | vtrim_bod << 0i32) as (u8);
    }
    setup_after_cold_reset_wakeup_from_shut_down_cfg3(ccfg_mode_conf_reg);

    let adr: u32 = *(*(0x10000180i32 as (*mut u32)).offset(8isize) as (*mut u32)).offset(3isize);
    (::core::mem::transmute::<*const (), unsafe extern "C" fn(u32) -> ()>(adr as *const ()))(
        0x2u32,
    );

    *((0x400c6000i32 + 0x0i32) as (*mut usize)) = 0x40usize;
    *(((0x40030000i32 + 0x24i32) as (usize) & 0xf0000000usize | 0x2000000usize
        | ((0x40030000i32 + 0x24i32) as (usize) & 0xfffffusize) << 5i32
        | (5i32 << 2i32) as (usize)) as (*mut usize)) = 1usize;
}

unsafe extern "C" fn trim_after_cold_reset() {}

unsafe extern "C" fn setup_sign_extend_vddr_trim_value(mut ui32vddr_trim_val: u32) -> i32 {
    let mut i32signed_vddr_val: i32 = ui32vddr_trim_val as (i32);
    if i32signed_vddr_val > 0x15i32 {
        i32signed_vddr_val = i32signed_vddr_val - 0x20i32;
    }
    i32signed_vddr_val
}

pub unsafe extern "C" fn setup_after_cold_reset_wakeup_from_shut_down_cfg1(
    mut ccfg_mode_conf_reg: u32,
) {
    let mut i32vddr_sleep_trim: i32;
    let mut i32vddr_sleep_delta: i32;
    i32vddr_sleep_trim = setup_sign_extend_vddr_trim_value(
        ((*((0x50001000i32 + 0x2b8i32) as (*mut usize)) & 0x1f000000usize) >> 24i32) as (u32),
    );
    i32vddr_sleep_delta = ccfg_mode_conf_reg as (i32) << 32i32 - 4i32 - 28i32 >> 32i32 - 4i32;
    i32vddr_sleep_trim = i32vddr_sleep_trim + i32vddr_sleep_delta + 1i32;
    if i32vddr_sleep_trim > 21i32 {
        i32vddr_sleep_trim = 21i32;
    }
    if i32vddr_sleep_trim < -10i32 {
        i32vddr_sleep_trim = -10i32;
    }
    *((0x40086200i32 + 0x60i32 + 0x7i32 * 2i32) as (*mut u16)) =
        (0x1fi32 << 8i32 | i32vddr_sleep_trim << 0i32 & 0x1fi32) as (u16);
    if *((0x40090000i32 + 0x0i32) as (*mut usize)) & 0x2usize != 0 {
        ccfg_mode_conf_reg = ccfg_mode_conf_reg | (0x8000000i32 | 0x4000000i32) as (u32);
    } else {
        *(((0x40095000i32 + 0x24i32) as (usize) & 0xf0000000usize | 0x2000000usize
            | ((0x40095000i32 + 0x24i32) as (usize) & 0xfffffusize) << 5i32
            | (5i32 << 2i32) as (usize)) as (*mut usize)) = 0usize;
    }
    *(((0x40090000i32 + 0x0i32) as (usize) & 0xf0000000usize | 0x2000000usize
        | ((0x40090000i32 + 0x0i32) as (usize) & 0xfffffusize) << 5i32
        | (0i32 << 2i32) as (usize)) as (*mut usize)) =
        (ccfg_mode_conf_reg >> 27i32 & 1u32 ^ 1u32) as (usize);
    *(((0x40090000i32 + 0x0i32) as (usize) & 0xf0000000usize | 0x2000000usize
        | ((0x40090000i32 + 0x0i32) as (usize) & 0xfffffusize) << 5i32
        | (2i32 << 2i32) as (usize)) as (*mut usize)) =
        (ccfg_mode_conf_reg >> 26i32 & 1u32 ^ 1u32) as (usize);
}

pub unsafe extern "C" fn setup_after_cold_reset_wakeup_from_shut_down_cfg2(
    mut ui32fcfg1revision: u32,
    mut ccfg_mode_conf_reg: u32,
) {
    let mut ui32trim: u32;
    ui32trim = setup_get_trim_for_anabypass_value1(ccfg_mode_conf_reg);
    ddi::ddi32reg_write(0x400ca000u32, 0x18u32, ui32trim);
    ui32trim = setup_get_trim_for_rc_osc_lf_rtune_ctune_trim();
    ddi::ddi16bitfield_write(
        0x400ca000u32,
        0x2cu32,
        (0xffi32 | 0x300i32) as (u32),
        0u32,
        ui32trim as (u16),
    );
    ui32trim = setup_get_trim_for_xosc_hf_ibiastherm();
    ddi::ddi32reg_write(0x400ca000u32, 0x1cu32, ui32trim << 0i32);
    ui32trim = setup_get_trim_for_ampcomp_th2();
    ddi::ddi32reg_write(0x400ca000u32, 0x14u32, ui32trim);
    ui32trim = setup_get_trim_for_ampcomp_th1();
    ddi::ddi32reg_write(0x400ca000u32, 0x10u32, ui32trim);
    ui32trim = setup_get_trim_for_ampcomp_ctrl(ui32fcfg1revision);
    ddi::ddi32reg_write(0x400ca000u32, 0xcu32, ui32trim);
    ui32trim = setup_get_trim_for_adc_sh_mode_en(ui32fcfg1revision);
    *((0x400ca000i32 + 0x100i32 + 0x24i32 * 2i32 + 1i32) as (*mut u8)) =
        (0x20u32 | ui32trim << 1i32) as (u8);
    ui32trim = setup_get_trim_for_adc_sh_vbuf_en(ui32fcfg1revision);
    *((0x400ca000i32 + 0x100i32 + 0x24i32 * 2i32 + 1i32) as (*mut u8)) =
        (0x10u32 | ui32trim) as (u8);
    ui32trim = setup_get_trim_for_xosc_hf_ctl(ui32fcfg1revision);
    ddi::ddi32reg_write(0x400ca000u32, 0x28u32, ui32trim);
    ui32trim = setup_get_trim_for_dblr_loop_filter_reset_voltage(ui32fcfg1revision);
    *((0x400ca000i32 + 0x100i32 + 0x24i32 * 2i32 + 4i32) as (*mut u8)) =
        (0x60u32 | ui32trim << 1i32) as (u8);
    ui32trim = setup_get_trim_for_rc_osc_lf_ibias_trim(ui32fcfg1revision);
    *((0x400ca000i32 + 0x100i32 + 0x20i32 * 2i32 + 1i32) as (*mut u8)) =
        (0x80u32 | ui32trim << 3i32) as (u8);
    ui32trim = setup_get_trim_for_xosc_lf_regulator_and_cmirrwr_ratio(ui32fcfg1revision);
    *((0x400ca000i32 + 0x180i32 + 0x2ci32 * 2i32 + 4i32) as (*mut u16)) =
        (0xfc00u32 | ui32trim << 2i32) as (u16);
    ui32trim = setup_get_trim_for_radc_ext_cfg(ui32fcfg1revision);
    ddi::ddi32reg_write(0x400ca000u32, 0x8u32, ui32trim);
    *((0x400ca000i32 + 0x40i32 + 0x0i32) as (*mut usize)) = 0x400000usize;
}

unsafe extern "C" fn sys_ctrl_aon_sync() {
    *((0x40092000i32 + 0x2ci32) as (*mut usize));
}

pub unsafe extern "C" fn setup_after_cold_reset_wakeup_from_shut_down_cfg3(
    mut ccfg_mode_conf_reg: u32,
) {
    let mut _current_block;
    let mut fcfg1osc_conf: u32;
    let mut ui32trim: u32;
    let mut current_hf_clock: u32;
    let mut ccfg_ext_lf_clk: u32;
    let switch1 = (ccfg_mode_conf_reg & 0xc0000u32) >> 18i32;
    if !(switch1 == 2u32) {
        if switch1 == 1u32 {
            fcfg1osc_conf = *((0x50001000i32 + 0x38ci32) as (*mut usize)) as (u32);
            if fcfg1osc_conf & 0x20000u32 == 0u32 {
                *((0x400ca000i32 + 0x40i32 + 0x0i32) as (*mut usize)) = 0x4000usize;
                *((0x40086000i32 + 0xci32) as (*mut usize)) =
                    *((0x40086000i32 + 0xci32) as (*mut usize)) & !(0x80i32 | 0xfi32) as (usize)
                        | ((fcfg1osc_conf & 0x10000u32) >> 16i32 << 7i32) as (usize)
                        | ((fcfg1osc_conf & 0xf000u32) >> 12i32 << 0i32) as (usize);
                *((0x40086000i32 + 0xbi32) as (*mut usize)) =
                    *((0x40086000i32 + 0xbi32) as (*mut usize)) & !0xfi32 as (usize)
                        | ((fcfg1osc_conf & 0xf00u32) >> 8i32 << 0i32) as (usize);
                *((0x40086000i32 + 0xai32) as (*mut usize)) = *((0x40086000i32 + 0xai32)
                    as (*mut usize))
                    & !(0x80i32 | 0x60i32 | 0x6i32 | 0x1i32) as (usize)
                    | ((fcfg1osc_conf & 0x80u32) >> 7i32 << 7i32) as (usize)
                    | ((fcfg1osc_conf & 0x60u32) >> 5i32 << 5i32) as (usize)
                    | ((fcfg1osc_conf & 0x6u32) >> 1i32 << 1i32) as (usize)
                    | ((fcfg1osc_conf & 0x1u32) >> 0i32 << 0i32) as (usize);
                _current_block = 6;
            } else {
                _current_block = 4;
            }
        } else {
            _current_block = 4;
        }
        if _current_block == 6 {
        } else {
            *((0x400ca000i32 + 0x40i32 + 0x0i32) as (*mut usize)) = 0x80000000usize;
        }
    }
    if *((0x50003000i32 + 0xfb0i32) as (*mut usize)) & 0x8usize == 0usize {
        *((0x400ca000i32 + 0x40i32 + 0x28i32) as (*mut usize)) = 0x40usize;
    }
    *((0x400ca000i32 + 0x80i32 + 0x0i32) as (*mut usize)) = 0x200usize;
    ui32trim = setup_get_trim_for_xosc_hf_fast_start();
    *((0x400ca000i32 + 0x100i32 + 0x4i32 * 2i32) as (*mut u8)) = (0x30u32 | ui32trim) as (u8);
    let switch2 = (ccfg_mode_conf_reg & 0xc00000u32) >> 22i32;
    if switch2 == 2u32 {
        _current_block = 17;
    } else if switch2 == 1u32 {
        current_hf_clock = oscfh::clock_source_get(0x1u32);
        oscfh::clock_source_set(0x4u32, current_hf_clock);
        'loop15: loop {
            if !(oscfh::clock_source_get(0x4u32) != current_hf_clock) {
                break;
            }
        }
        ccfg_ext_lf_clk = *((0x50003000i32 + 0xfa8i32) as (*mut usize)) as (u32);
        setup_set_aon_rtc_sub_sec_inc((ccfg_ext_lf_clk & 0xffffffu32) >> 0i32);
        let adr: u32 =
            *(*(0x10000180i32 as (*mut u32)).offset(13isize) as (*mut u32)).offset(0isize);
        (::core::mem::transmute::<*const (), unsafe extern "C" fn(u32, u32, u32) -> ()>(
            adr as *const (),
        ))(
            (ccfg_ext_lf_clk & 0xff000000u32) >> 24i32,
            0x7u32,
            (0x0i32 | 0x0i32 | 0x6000i32 | 0x0i32 | 0x0i32 | 0x0i32 | 0x0i32 | 0x0i32 | 0x0i32
                | 0x20000000i32 | 0x40000000i32) as (u32),
        );

        *((0x400ca000i32 + 0x40i32 + 0x0i32) as (*mut usize)) = 0x400usize;
        _current_block = 17;
    } else {
        if switch2 == 0u32 {
            oscfh::clock_source_set(0x4u32, 0x1u32);
            setup_set_aon_rtc_sub_sec_inc(0x8637bdu32);
        } else {
            oscfh::clock_source_set(0x4u32, 0x2u32);
        }
        _current_block = 18;
    }
    if _current_block == 17 {
        oscfh::clock_source_set(0x4u32, 0x3u32);
    }
    *((0x400cb000i32 + 0xbi32) as (*mut u8)) =
        (*((0x50001000i32 + 0x36ci32) as (*mut usize)) >> 0i32 << 0i32 & 0x3fusize) as (u8);
    *((0x400cb000i32 + 0x60i32 + 0x8i32 * 2i32) as (*mut u16)) =
        (0x78i32 << 8i32 | 3i32 << 3i32) as (u16);
    sys_ctrl_aon_sync();
}

pub unsafe extern "C" fn setup_get_trim_for_anabypass_value1(mut ccfg_mode_conf_reg: u32) -> u32 {
    let mut ui32fcfg1value: u32;
    let mut ui32xosc_hf_row: u32;
    let mut ui32xosc_hf_col: u32;
    let mut i32customer_delta_adjust: i32;
    let mut ui32trim_value: u32;
    ui32fcfg1value = *((0x50001000i32 + 0x350i32) as (*mut usize)) as (u32);
    ui32xosc_hf_row = (ui32fcfg1value & 0x3c000000u32) >> 26i32;
    ui32xosc_hf_col = (ui32fcfg1value & 0x3fffc00u32) >> 10i32;
    //i32customer_delta_adjust = 0i32;
    if ccfg_mode_conf_reg & 0x20000u32 == 0u32 {
        i32customer_delta_adjust = ccfg_mode_conf_reg as (i32) << 16i32 >> 24i32;
        'loop2: loop {
            if !(i32customer_delta_adjust < 0i32) {
                break;
            }
            ui32xosc_hf_col = ui32xosc_hf_col >> 1i32;
            if ui32xosc_hf_col == 0u32 {
                ui32xosc_hf_col = 0xffffu32;
                ui32xosc_hf_row = ui32xosc_hf_row >> 1i32;
                if ui32xosc_hf_row == 0u32 {
                    ui32xosc_hf_row = 1u32;
                    ui32xosc_hf_col = 1u32;
                }
            }
            i32customer_delta_adjust = i32customer_delta_adjust + 1;
        }
        'loop3: loop {
            if !(i32customer_delta_adjust > 0i32) {
                break;
            }
            ui32xosc_hf_col = ui32xosc_hf_col << 1i32 | 1u32;
            if ui32xosc_hf_col > 0xffffu32 {
                ui32xosc_hf_col = 1u32;
                ui32xosc_hf_row = ui32xosc_hf_row << 1i32 | 1u32;
                if ui32xosc_hf_row > 0xfu32 {
                    ui32xosc_hf_row = 0xfu32;
                    ui32xosc_hf_col = 0xffffu32;
                }
            }
            i32customer_delta_adjust = i32customer_delta_adjust - 1;
        }
    }
    ui32trim_value = ui32xosc_hf_row << 16i32 | ui32xosc_hf_col << 0i32;
    ui32trim_value
}

pub unsafe extern "C" fn setup_get_trim_for_rc_osc_lf_rtune_ctune_trim() -> u32 {
    let mut ui32trim_value: u32;
    ui32trim_value =
        ((*((0x50001000i32 + 0x350i32) as (*mut usize)) & 0x3fcusize) >> 2i32 << 0i32) as (u32);
    ui32trim_value = (ui32trim_value as (usize)
        | (*((0x50001000i32 + 0x350i32) as (*mut usize)) & 0x3usize) >> 0i32 << 8i32)
        as (u32);
    ui32trim_value
}

pub unsafe extern "C" fn setup_get_trim_for_xosc_hf_ibiastherm() -> u32 {
    let mut ui32trim_value: u32;
    ui32trim_value =
        ((*((0x50001000i32 + 0x37ci32) as (*mut usize)) & 0x3fffusize) >> 0i32) as (u32);
    ui32trim_value
}

pub unsafe extern "C" fn setup_get_trim_for_ampcomp_th2() -> u32 {
    let mut ui32trim_value: u32;
    let mut ui32fcfg1value: u32;
    ui32fcfg1value = *((0x50001000i32 + 0x374i32) as (*mut usize)) as (u32);
    ui32trim_value = (ui32fcfg1value & 0xfc000000u32) >> 26i32 << 26i32;
    ui32trim_value = ui32trim_value | (ui32fcfg1value & 0xfc0000u32) >> 18i32 << 18i32;
    ui32trim_value = ui32trim_value | (ui32fcfg1value & 0xfc00u32) >> 10i32 << 10i32;
    ui32trim_value = ui32trim_value | (ui32fcfg1value & 0xfcu32) >> 2i32 << 2i32;
    ui32trim_value
}

pub unsafe extern "C" fn setup_get_trim_for_ampcomp_th1() -> u32 {
    let mut ui32trim_value: u32;
    let mut ui32fcfg1value: u32;
    ui32fcfg1value = *((0x50001000i32 + 0x370i32) as (*mut usize)) as (u32);
    ui32trim_value = (ui32fcfg1value & 0xfc0000u32) >> 18i32 << 18i32;
    ui32trim_value = ui32trim_value | (ui32fcfg1value & 0xfc00u32) >> 10i32 << 10i32;
    ui32trim_value = ui32trim_value | (ui32fcfg1value & 0x3c0u32) >> 6i32 << 6i32;
    ui32trim_value = ui32trim_value | (ui32fcfg1value & 0x3fu32) >> 0i32 << 0i32;
    ui32trim_value
}

pub unsafe extern "C" fn setup_get_trim_for_ampcomp_ctrl(mut ui32fcfg1revision: u32) -> u32 {
    let mut ui32trim_value: u32;
    let mut ui32fcfg1value: u32;
    let mut ibias_offset: u32;
    let mut ibias_init: u32;
    let mut mode_conf1: u32;
    let mut delta_adjust: i32;
    ui32fcfg1value = *((0x50001000i32 + 0x378i32) as (*mut usize)) as (u32);
    ibias_offset = (ui32fcfg1value & 0xf00000u32) >> 20i32;
    ibias_init = (ui32fcfg1value & 0xf0000u32) >> 16i32;
    if *((0x50003000i32 + 0xfb0i32) as (*mut usize)) & 0x1usize == 0usize {
        mode_conf1 = *((0x50003000i32 + 0xfaci32) as (*mut usize)) as (u32);
        delta_adjust = mode_conf1 as (i32) << 32i32 - 8i32 - 4i32 >> 28i32;
        delta_adjust = delta_adjust + ibias_offset as (i32);
        if delta_adjust < 0i32 {
            delta_adjust = 0i32;
        }
        if delta_adjust > 0xf00000i32 >> 20i32 {
            delta_adjust = 0xf00000i32 >> 20i32;
        }
        ibias_offset = delta_adjust as (u32);
        delta_adjust = mode_conf1 as (i32) << 32i32 - 12i32 - 4i32 >> 28i32;
        delta_adjust = delta_adjust + ibias_init as (i32);
        if delta_adjust < 0i32 {
            delta_adjust = 0i32;
        }
        if delta_adjust > 0xf0000i32 >> 16i32 {
            delta_adjust = 0xf0000i32 >> 16i32;
        }
        ibias_init = delta_adjust as (u32);
    }
    ui32trim_value = ibias_offset << 20i32 | ibias_init << 16i32;
    ui32trim_value = ui32trim_value | (ui32fcfg1value & 0xff00u32) >> 8i32 << 8i32;
    ui32trim_value = ui32trim_value | (ui32fcfg1value & 0xf0u32) >> 4i32 << 4i32;
    ui32trim_value = ui32trim_value | (ui32fcfg1value & 0xfu32) >> 0i32 << 0i32;
    if ui32fcfg1revision >= 0x22u32 {
        ui32trim_value = ui32trim_value | (ui32fcfg1value & 0x40000000u32) >> 30i32 << 30i32;
    }
    ui32trim_value
}

pub unsafe extern "C" fn setup_get_trim_for_dblr_loop_filter_reset_voltage(
    mut ui32fcfg1revision: u32,
) -> u32 {
    let mut dblr_loop_filter_reset_voltage_value: u32 = 0u32;
    if ui32fcfg1revision >= 0x20u32 {
        dblr_loop_filter_reset_voltage_value =
            ((*((0x50001000i32 + 0x398i32) as (*mut usize)) & 0x300000usize) >> 20i32) as (u32);
    }
    dblr_loop_filter_reset_voltage_value
}

pub unsafe extern "C" fn setup_get_trim_for_adc_sh_mode_en(mut ui32fcfg1revision: u32) -> u32 {
    let mut get_trim_for_adc_sh_mode_en_value: u32 = 1u32;
    if ui32fcfg1revision >= 0x22u32 {
        get_trim_for_adc_sh_mode_en_value =
            ((*((0x50001000i32 + 0x38ci32) as (*mut usize)) & 0x10000000usize) >> 28i32) as (u32);
    }
    get_trim_for_adc_sh_mode_en_value
}

pub unsafe extern "C" fn setup_get_trim_for_adc_sh_vbuf_en(mut ui32fcfg1revision: u32) -> u32 {
    let mut get_trim_for_adc_sh_vbuf_en_value: u32 = 1u32;
    if ui32fcfg1revision >= 0x22u32 {
        get_trim_for_adc_sh_vbuf_en_value =
            ((*((0x50001000i32 + 0x38ci32) as (*mut usize)) & 0x20000000usize) >> 29i32) as (u32);
    }
    get_trim_for_adc_sh_vbuf_en_value
}

pub unsafe extern "C" fn setup_get_trim_for_xosc_hf_ctl(mut ui32fcfg1revision: u32) -> u32 {
    let mut get_trim_for_xoschf_ctl_value: u32 = 0u32;
    let mut fcfg1data: u32;
    if ui32fcfg1revision >= 0x20u32 {
        fcfg1data = *((0x50001000i32 + 0x398i32) as (*mut usize)) as (u32);
        get_trim_for_xoschf_ctl_value = (fcfg1data & 0x18000000u32) >> 27i32 << 8i32;
        get_trim_for_xoschf_ctl_value =
            get_trim_for_xoschf_ctl_value | (fcfg1data & 0x7000000u32) >> 24i32 << 2i32;
        get_trim_for_xoschf_ctl_value =
            get_trim_for_xoschf_ctl_value | (fcfg1data & 0xc00000u32) >> 22i32 << 0i32;
    }
    get_trim_for_xoschf_ctl_value
}

pub unsafe extern "C" fn setup_get_trim_for_xosc_hf_fast_start() -> u32 {
    let mut ui32xosc_hf_fast_start_value: u32;
    ui32xosc_hf_fast_start_value =
        ((*((0x50001000i32 + 0x38ci32) as (*mut usize)) & 0x180000usize) >> 19i32) as (u32);
    ui32xosc_hf_fast_start_value
}

pub unsafe extern "C" fn setup_get_trim_for_radc_ext_cfg(mut ui32fcfg1revision: u32) -> u32 {
    let mut get_trim_for_radc_ext_cfg_value: u32 = 0x403f8000u32;
    let mut fcfg1data: u32;
    if ui32fcfg1revision >= 0x20u32 {
        fcfg1data = *((0x50001000i32 + 0x398i32) as (*mut usize)) as (u32);
        get_trim_for_radc_ext_cfg_value = (fcfg1data & 0xffc00u32) >> 10i32 << 22i32;
        get_trim_for_radc_ext_cfg_value =
            get_trim_for_radc_ext_cfg_value | (fcfg1data & 0x3f0u32) >> 4i32 << 16i32;
        get_trim_for_radc_ext_cfg_value =
            get_trim_for_radc_ext_cfg_value | (fcfg1data & 0xfu32) >> 0i32 << 12i32;
    }
    get_trim_for_radc_ext_cfg_value
}

pub unsafe extern "C" fn setup_get_trim_for_rc_osc_lf_ibias_trim(
    mut ui32fcfg1revision: u32,
) -> u32 {
    let mut trim_for_rc_osc_lf_ibias_trim_value: u32 = 0u32;
    if ui32fcfg1revision >= 0x22u32 {
        trim_for_rc_osc_lf_ibias_trim_value =
            ((*((0x50001000i32 + 0x38ci32) as (*mut usize)) & 0x8000000usize) >> 27i32) as (u32);
    }
    trim_for_rc_osc_lf_ibias_trim_value
}

pub unsafe extern "C" fn setup_get_trim_for_xosc_lf_regulator_and_cmirrwr_ratio(
    mut ui32fcfg1revision: u32,
) -> u32 {
    let mut trim_for_xosc_lf_regulator_and_cmirrwr_ratio_value: u32 = 0u32;
    if ui32fcfg1revision >= 0x22u32 {
        trim_for_xosc_lf_regulator_and_cmirrwr_ratio_value =
            ((*((0x50001000i32 + 0x38ci32) as (*mut usize))
                & (0x6000000i32 | 0x1e00000i32) as (usize)) >> 21i32) as (u32);
    }
    trim_for_xosc_lf_regulator_and_cmirrwr_ratio_value
}

pub unsafe extern "C" fn setup_set_cache_mode_according_to_ccfg_setting() {
    let mut vims_ctl_mode0: u32;
    'loop1: loop {
        if *(((0x40034000i32 + 0x0i32) as (usize) & 0xf0000000usize | 0x2000000usize
            | ((0x40034000i32 + 0x0i32) as (usize) & 0xfffffusize) << 5i32
            | (3i32 << 2i32) as (usize)) as (*mut usize)) == 0
        {
            break;
        }
    }
    vims_ctl_mode0 = (*((0x40034000i32 + 0x4i32) as (*mut usize)) & !0x3i32 as (usize)
        | 0x20000000usize | 0x4usize) as (u32);
    if *((0x50003000i32 + 0xfb0i32) as (*mut usize)) & 0x4usize != 0 {
        *((0x40034000i32 + 0x4i32) as (*mut usize)) = (vims_ctl_mode0 | 0x1u32) as (usize);
    } else if *((0x40034000i32 + 0x0i32) as (*mut usize)) & 0x3usize != 0x0usize {
        *((0x40034000i32 + 0x4i32) as (*mut usize)) = (vims_ctl_mode0 | 0x3u32) as (usize);
        'loop6: loop {
            if !(*((0x40034000i32 + 0x0i32) as (*mut usize)) & 0x3usize != 0x3usize) {
                break;
            }
        }
        *((0x40034000i32 + 0x4i32) as (*mut usize)) = vims_ctl_mode0 as (usize);
    } else {
        *((0x40034000i32 + 0x4i32) as (*mut usize)) = vims_ctl_mode0 as (usize);
    }
}

pub unsafe extern "C" fn setup_set_aon_rtc_sub_sec_inc(mut sub_sec_inc: u32) {
    *((0x400c6000i32 + 0x3ci32) as (*mut usize)) = (sub_sec_inc & 0xffffu32) as (usize);
    *((0x400c6000i32 + 0x40i32) as (*mut usize)) = (sub_sec_inc >> 16i32 & 0xffu32) as (usize);
    *((0x400c6000i32 + 0x44i32) as (*mut usize)) = 0x1usize;
    'loop1: loop {
        if !(*(((0x400c6000i32 + 0x44i32) as (usize) & 0xf0000000usize | 0x2000000usize
            | ((0x400c6000i32 + 0x44i32) as (usize) & 0xfffffusize) << 5i32
            | (1i32 << 2i32) as (usize)) as (*mut usize)) == 0)
        {
            break;
        }
    }
    *((0x400c6000i32 + 0x44i32) as (*mut usize)) = 0usize;
}
