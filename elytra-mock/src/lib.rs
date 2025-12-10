#![feature(macro_metavar_expr)]

use elytra_conf::elytra;
use elytra_conf::prelude::{*};
use chrono_tz::TZ_VARIANTS;

pub mod handler;

#[derive(Debug)]
struct TimeZoneOpts{}
impl OptionValueProvider for TimeZoneOpts {
    fn get(&self, index: usize) -> Option<&'static str> {
        TZ_VARIANTS.get(index).map(|tz| tz.name())
    }

    fn len(&self) -> usize {
        TZ_VARIANTS.len()
    }
}
const TIME_ZONE_OPTS: TimeZoneOpts = TimeZoneOpts{};

elytra!( pub MOCK_CONF: MockConf {
    info: {
        WifiStatus: status("Connection Status")
            .with_help("The current progress or result (failure or success)")
            .with_icon("wifi-sync"),

        FlashUUID: bytes("Flash Unique ID", 8)
            .with_help("A unique identifier for the flash chip"),
        FlashJEDEC: bytes("Flash JEDEC ID", 4)
            .with_help("The manufacturer flash chip designation"),
        PicoROM: info("Pico ROM")
            .with_help("The version of the Read Only firmware of the Pico"),
        Time: info("Time")
            .with_help("The current time, as would be displayed on the clock")
    },
    props: {
        WifiNetwork: prop("Network (SSID)")
            .with_help("The name the WiFi network to connect to"),
        WifiPassword: secret("Password")
            .with_help("The password for the WiFi network"),
        BrightOffset: integer("Brightness Offset")
            .with_help("Adjustment of the display brightness auto value")
            .writable()
            .with_range(-1500..1500),
        Serial: integer("Serial number")
            .with_help("The unique series number of your device")
            .writable(),
        TimeZone: prop("Timezone")
            .with_options(&TIME_ZONE_OPTS)
            .with_help("The timezone used for adjusting DST and displayed time offset")
            .with_default_text("Europe/Stockholm"),
        NtpServer: prop("NTP Server")
            .with_help("The Network Time Protocol server to query for the current time")
            .with_default_text("ntp.se")
    },
    sections: {
        Wifi: section("WiFi")
            .with_help("Connection details to be used to sync the time over the internet")
            .with_icon("wifi"),

        Display: section("Display")
            .with_icon("contrast"),

        Clock: section("Clock")
            .with_icon("clock"),

        Hardware: section("Hardware Info")
            .with_icon("cog")
    },
    actions: {
        Reset: action("Reset")
            .with_icon("power"),

        DFU: action("DFU")
            .with_icon("hard-drive-download")
    },
    layout: {
        Section::Wifi: [
            Field::Info(InfoField::WifiStatus),
            Field::Prop(PropField::WifiNetwork),
            Field::Prop(PropField::WifiPassword)
        ],
        Section::Display: [
            Field::Prop(PropField::BrightOffset)
        ],
        Section::Clock: [
            Field::Info(InfoField::Time),
            Field::Prop(PropField::NtpServer),
            Field::Prop(PropField::TimeZone)
        ],
        Section::Hardware: [
            Field::Prop(PropField::Serial),
            Field::Info(InfoField::FlashUUID),
            Field::Info(InfoField::FlashJEDEC),
            Field::Info(InfoField::PicoROM)
        ]
    }
}
);

#[cfg(target_arch = "wasm32")]
elytra_wasm::elytra_wasm! { ELYTRA_MOCK, &mut crate::handler::MockHandler }