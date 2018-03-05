//! BLE Controller
//!     Manages bluetooth.

use self::ble_commands::*;
use cc26xx::{osc,prcm};
use radio::rfc::{self, rfc_commands};

static mut BLE_OVERRIDES: [u32; 7] = [
    0x00364038 /* Synth: Set RTRIM (POTAILRESTRIM) to 6 */,
    0x000784A3 /* Synth: Set FREF = 3.43 MHz (24 MHz / 7) */,
    0xA47E0583 /* Synth: Set loop bandwidth after lock to 80 kHz (K2) */,
    0xEAE00603 /* Synth: Set loop bandwidth after lock to 80 kHz (K3, LSB) */,
    0x00010623 /* Synth: Set loop bandwidth after lock to 80 kHz (K3, MSB) */,
    0x00456088 /* Adjust AGC reference level */,
    0xFFFFFFFF /* End of override list */,
];

static mut BLE_PARAMS_BUF: [u32; 32] = [0; 32];
static mut PAYLOAD: [u8; 64] = [0; 64];

// TODO(cpluss): change this to randomised generated
static mut DEVICE_ADDRESS: [u8; 6] = [0xf0, 0x10, 0x20, 0x34, 0x56, 0xf0];

pub struct Ble<'a> {
    rfc: &'a rfc::RFCore,
}

/* BLE RFC Commands */
const RFC_BLE_ADVERTISE: u16 = 0x1805;

impl<'a> Ble<'a> {
    pub const fn new(rfc: &'a rfc::RFCore) -> Ble<'a> {
        Ble {
            rfc: rfc,
        }
    }

    pub fn power_up(&self) {
        /*
            The BLE communication is synchronous, so we need to be synchronized to the same
            clock frequency. The best accuracy is achieved when using the XTAL Oscillator.

            However, it takes a while for it to pulse correctly, so we enable it
            before switching to it.
        */
        osc::OSCILLATOR_CONTROL.request_switch_to_hf_xosc();

        prcm::rf_mode_sel(0x01);

        self.rfc.enable();
        self.rfc.start_rat();

        osc::OSCILLATOR_CONTROL.switch_to_hf_xosc();

        unsafe {
            let reg_overrides: u32 = (&BLE_OVERRIDES[0] as *const u32) as u32;
            //(&self.ble_overrides as *const BleOverrides) as u32;
            self.rfc.setup(0x00, reg_overrides); // Mode 0 = BLE
        }
    }

    pub fn advertise(&self) {
        let name = "lol det funkar";

        unsafe {
            for i in 0..PAYLOAD.len() {
                PAYLOAD[i] = 0;
            }

            PAYLOAD[0] = 0x02; // 2 bytes
            PAYLOAD[1] = 0x01; // ADV TYPE DEVINFO
            PAYLOAD[2] = 0x02; // LE General Discoverable Mode
            //PAYLOAD[2] = 0x1A; // LE general discoverable + BR/EDR
            PAYLOAD[3] = (name.len() + 1) as u8;

            PAYLOAD[4] = 0x09; // ADV TYPE NAME
            let mut p = 5;
            for c in name.chars() {
                PAYLOAD[p] = c as u8;
                p = p + 1;
            }

            for channel in 37..40 {
                self.advertise_on(channel, &mut PAYLOAD, p as u8);
            }
        }
    }

    #[inline(never)]
    #[no_mangle]
    pub unsafe fn advertise_on(&self, channel: u8, payload: &mut [u8], payload_len: u8) {
        for i in 0..BLE_PARAMS_BUF.len() {
            BLE_PARAMS_BUF[i] = 0;
        }

        let params: &mut BleAdvertiseParams =
            &mut *(::core::mem::transmute::<*mut u32, *mut BleAdvertiseParams>
                (&mut BLE_PARAMS_BUF[0] as *mut u32));

        params.device_address = &mut DEVICE_ADDRESS[0] as *mut u8;
        params.adv_len = payload_len;
        params.adv_data = payload[0] as *const u8;
        params.end_time = 1;
        params.end_trigger = 1;

        /*let params = BleAdvertiseParams {
            rx_queue: 0, // pointer to receive queue
            rx_config: 0,
            adv_config: 0,

            adv_len: payload_len,
            scan_rsp_len: 0,

            adv_data: payload as *mut [u8],
            scan_rsp_data: 0,
            device_address: &DEVICE_ADDRESS as *mut [u8],

            white_list: 0,

            __dummy0: 0,
            __dummy1: 0,

            end_trigger: 0,
            end_time: 0,
        };*/

        let cmd = BleAdvertise {
            command_no: RFC_BLE_ADVERTISE,
            status: 0,
            p_nextop: 0,
            ratmr: 0,
            start_trigger: 0,
            condition: {
                let mut cnd = rfc_commands::RfcCondition(0);
                cnd.set_rule(1); // COND_NEVER
                cnd
            },

            channel: channel,
            whitening: {
                let mut wht = BleWhitening(0);
                wht.set_override(false);
                wht.set_init(0);
                wht
            },

            params: params as *const BleAdvertiseParams,
            output: 0,
        };

        debug_verbose!("Advertising with payload 0x{:x}, len={}\r", (&payload[0] as *const u8) as u32, payload_len);

        // Queue the advertisement command
        self.rfc.send(&cmd);

        match self.rfc.wait_for(&cmd) {
            rfc::RfcResult::Error(status) => panic!(
                "Error during advertisement on channel={}, status=0x{:x}, cmdsta=0x{:x}, cmdr=0x{:x}\r",
                channel, status, self.rfc.get_status(), self.rfc.get_command()
            ),
            rfc::RfcResult::Ok => debug_verbose!("Sent advertisement on channel {}\r", channel),
        }
    }
}

pub mod ble_commands {
    use radio::rfc::*;
    use radio::rfc::rfc_commands::*;

    #[repr(C)]
    pub struct BleAdvertise {
        pub command_no: u16,
        pub status: u16,
        pub p_nextop: u32,
        pub ratmr: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,

        pub channel: u8,
        pub whitening: BleWhitening,

        pub params: *const BleAdvertiseParams,
        pub output: u32,
    }

    #[repr(C)]
    pub struct BleAdvertiseParams {
        pub rx_queue: u32, // pointer to receive queue
        pub rx_config: u8,
        pub adv_config: u8,

        pub adv_len: u8,
        pub scan_rsp_len: u8,

        pub adv_data: *const u8,
        pub scan_rsp_data: u32,
        pub device_address: *const u8,

        pub white_list: u32,

        pub __dummy0: u16,
        pub __dummy1: u8,

        pub end_trigger: u8,
        pub end_time: u32,
    }

    impl RfcCommand for BleAdvertise {
        fn command_type(&self) -> RfcCommandType {
            RfcCommandType::Immediate
        }
        fn command_id(&self) -> &u16 {
            &self.command_no
        }
        fn command_status(&self) -> &u16 {
            &self.status
        }
    }

    bitfield!{
        #[derive(Copy, Clone)]
        pub struct BleWhitening(u8);
        impl Debug;
        pub _init, set_init: 6, 0;
        pub _override, set_override: 1;
    }
}
