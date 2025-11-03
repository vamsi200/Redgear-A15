#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_macros)]

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use hex;
use hidapi::{HidApi, HidDevice};
use std::{thread::sleep, time::Duration};

const VID: u16 = 0x1bcf;
const PID: u16 = 0x08a0;

#[derive(Parser, Debug)]
#[command(
    name = "Redgear-A15",
    version,
    about = "Control Redgear A-15 mouse firmware from Linux"
)]

pub struct Args {
    #[arg(short, long, global = true, value_parser = clap::value_parser!(u8).range(0..=255))]
    pub repeat: Option<u8>,

    #[arg(short, long, global = true, value_parser = clap::value_parser!(u8).range(0..=255))]
    pub firing_interval: Option<u8>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Dpi {
        #[arg(value_parser = clap::value_parser!(u16).range(100..=8000))]
        value: u16,
    },

    Led {
        #[arg(value_enum)]
        mode: LedMode,
    },

    Power {
        #[arg(value_enum)]
        state: LedPowerState,
    },

    List,

    Reset,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum LedMode {
    Dpi,
    Multi,
    Rainbow,
    FloeLight,
    Waltz,
    FourSeasons,
    Off,
}

impl LedMode {
    pub fn hex(&self) -> &'static str {
        match self {
            LedMode::Multi => LED_MODE_MULTI,
            LedMode::Rainbow => LED_MODE_RAINBOW,
            LedMode::FloeLight => LED_MODE_FLOE_LIGHT,
            LedMode::Waltz => LED_MODE_WALTZ,
            LedMode::FourSeasons => LED_MODE_FOUR_SEASONS,
            LedMode::Dpi => LED_MODE_DPI,
            LedMode::Off => LED_MODE_OFF,
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum LedPowerState {
    Enable,
    Disable,
}

const DPI1: &str = "040700ff817e807f"; // 1000
const DPI2: &str = "040701fe817e807f"; // 1600
const DPI3: &str = "040702fd817e807f"; // 2400
const DPI4: &str = "040703fd817e807f"; // 3200
const DPI5: &str = "040704fd817e807f"; // 4800
const DPI6: &str = "040705fd817e807f"; // 6400
const DPI7: &str = "040706fd817e807f"; // 7200
const DPI8: &str = "040707fd817e807f"; // 8000

const CONTINOUUSLY_DISABLED: &str = "0407fdfffffc1bff";
const CONTINOUUSLY_LED_HEX: &str = "0407fdfffffc64ff"; // Repeat shall be disabled - 04070afdffa1fe03
const LED_DISABLE: &str = "040701fe8976807f";
const LED_ENABLE: &str = "040701fe817e807f";
const LED_MODE_MULTI: &str = "040701fe827d807f";
const LED_MODE_RAINBOW: &str = "040701fe837c807f";
const LED_MODE_FLOE_LIGHT: &str = "040701fe847b807f";
const LED_MODE_WALTZ: &str = "040701fe857a807f";
const LED_MODE_FOUR_SEASONS: &str = "040701fe8679807f";
const LED_MODE_DPI: &str = "040701fe817e807f";
const LED_MODE_OFF: &str = "040701fe8778807f";

macro_rules! generate_hex_val_for_repeat {
    (
        $HEX_VAL:expr,
        $REPEAT_REQ: expr
    ) => {
        let hex_val = $HEX_VAL.clone();
        let repeat_req_in_hex = hex::encode([$REPEAT_REQ]);
        let final_val = $HEX_VAL.replace("fd03", format!("fd{}", repeat_req_in_hex).as_str());
        println!("{}", final_val);
    };
}

macro_rules! generate_hex_for_interval {
    (
        $HEX_VAL:expr,
        $REPEAT_REQ: expr
    ) => {
        let hex_val = $HEX_VAL.clone();
        let repeat_req_in_hex = hex::encode([$REPEAT_REQ]);
        let final_val = $HEX_VAL.replace("fe08", format!("fe{}", repeat_req_in_hex).as_str());
        println!("{}", final_val);
    };
}

macro_rules! gen_hex_for_led {
    ($MODE:expr, $FULL_HEX:expr) => {{
        let mode_hex = $MODE.hex();
        let output: Vec<String> = $FULL_HEX
            .iter()
            .map(|&x| x.replace("040701fe817e807f", mode_hex))
            .collect();
        output
    }};
}

macro_rules! gen_hex_for_dpi {
    (
        $MODE: expr,
        $FULL_HEX: expr
    ) => {{
        let output: Vec<String> = $FULL_HEX
            .iter()
            .map(|&x| x.replace("040700ff817e807f", $MODE))
            .collect();
        output
    }};
}

fn set_repeat(val: u8) {
    let hex_val = "04070afd03a1fe03";
    if val < 2 {
        eprintln!("Repeat can only be between 2-255!");
    } else {
        generate_hex_val_for_repeat!(hex_val, val);
    }
}

fn set_firing_interval(val: u8) {
    let hex_val = "040721fe08fc94ff";
    if val < 2 {
        eprintln!("Repeat can only be between 2-255!");
    } else {
        generate_hex_for_interval!(hex_val, val);
    }
}

const LED_HEX: [&str; 48] = [
    "0401000000000000",
    "0403000000000000",
    "04060000ff000000",
    "040745f80638ff00",
    "040702040607090a",
    "0407070104030002",
    "04070506ff007fff",
    "0407ffff00ff00ff",
    "040700ff0000ffff",
    "0407000000ffffff",
    "0407ff00ffffff71",
    "040701fe817e807f",
    "0407ffffffffffff",
    "0407feffffff0101",
    "0407000104000102",
    "0407000108000110",
    "0407000500000700",
    "0407000800000600",
    "0407f00101000104",
    "0407000102000108",
    "0407000110000500",
    "0407000700000800",
    "0407000600f006ff",
    "0407feffffffffff",
    "0407fe990e05010e",
    "040705190e05310e",
    "040705490e05610e",
    "040705790e05910e",
    "040705a90e05c10e",
    "040705d9ffffffff",
    "0407ffffffffffff",
    "0407feffffffffff",
    "0407fdff00ff00ff",
    "040700ff00ff00ff",
    "040700ff00ff00ff",
    "040700ff00ffffff",
    "0407feffffffffff",
    "0407fdffffffff00",
    "04070000ff000000",
    "0407ffffff00ff00",
    "0407ff00ffffff80",
    "040700ff008000ff",
    "040780ffffffffff",
    "04070afd02a1fe03",
    "040721fe7bfc1bff",
    "0407fdfffffc1bff",
    "0408000000000000",
    "0402000000000000",
];

const DPI_HEX: [&str; 48] = [
    "0401000000000000",
    "0403000000000000",
    "04060000ff000000",
    "040745f80638ff00",
    "040702040607090a",
    "0407070104030002",
    "04070506ff007fff",
    "0407ffff00ff00ff",
    "040700ff0000ffff",
    "0407000000ffffff",
    "0407ff00ffffff71",
    "040700ff817e807f",
    "0407ffffffffffff",
    "0407feffffff0101",
    "0407000104000102",
    "0407000108000110",
    "0407000500000700",
    "0407000800000600",
    "0407f00101000104",
    "0407000102000108",
    "0407000110000500",
    "0407000700000800",
    "0407000600f006ff",
    "0407feffffffffff",
    "0407fe990e05010e",
    "040705190e05310e",
    "040705490e05610e",
    "040705790e05910e",
    "040705a90e05c10e",
    "040705d9ffffffff",
    "0407ffffffffffff",
    "0407feffffffffff",
    "0407fdff00ff00ff",
    "040700ff00ff00ff",
    "040700ff00ff00ff",
    "040700ff00ffffff",
    "0407feffffffffff",
    "0407fdffffffff00",
    "04070000ff000000",
    "0407ffffff00ff00",
    "0407ff00ffffff80",
    "040700ff008000ff",
    "040780ffffffffff",
    "04070afd02a1fe03",
    "040721fe06fc96ff",
    "0407fdfffffc96ff",
    "0408000000000000",
    "0402000000000000",
];

fn convert_str_hex(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|parts| {
            let hex_str = std::str::from_utf8(parts).unwrap();
            u8::from_str_radix(hex_str, 16).unwrap()
        })
        .collect()
}

fn bytes_to_hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02X}", b)).collect()
}

fn send_report_to_mouse(packets: Vec<Vec<u8>>, dev: HidDevice) -> Result<()> {
    println!("> Sending feature reports...");
    for pkts in &packets {
        println!("> SET_REPORT {}", bytes_to_hex(pkts));
        if let Err(e) = dev.send_feature_report(pkts) {
            eprintln!("! Failed to send: {e}");
            continue;
        }

        sleep(Duration::from_millis(300));

        let mut report_id = pkts.clone();
        println!("< GET_REPORT {}", bytes_to_hex(&report_id));
        if let Err(e) = dev.get_feature_report(&mut report_id) {
            eprintln!("! Failed to read: {e}");
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut led_mode = LedMode::Dpi;
    match args.command {
        Commands::Led { mode } => led_mode = mode,
        _ => {}
    }

    let hex_val = gen_hex_for_led!(led_mode, LED_HEX);
    let packets: Vec<Vec<u8>> = hex_val
        .iter()
        .map(|val| convert_str_hex(val.as_str()))
        .collect();
    let api = HidApi::new()?;
    let dev = api.open(VID, PID)?;

    if let Ok(_) = send_report_to_mouse(packets, dev) {
        println!("> All reports processed.");
    }

    Ok(())
}
