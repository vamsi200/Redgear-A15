use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use hex;
use hidapi::{HidApi, HidDevice};
use std::{process::exit, thread::sleep, time::Duration};

const VID: u16 = 0x1bcf;
const PID: u16 = 0x08a0;

#[derive(Parser, Debug)]
#[command(name = "Redgear-A15", version, about = "Control Redgear A-15 mouse")]
pub struct MouseArgs {
    #[command(flatten)]
    pub fire_control: Option<FireControl>,

    #[arg(
        short,
        long,
        value_parser = clap::value_parser!(u8).range(0..=255),
        help = "Mouse movement speed (0–255). Default: 6"
    )]
    pub moving_speed: Option<u8>,

    #[arg(
        short,
        long,
        value_parser = clap::value_parser!(u8).range(0..=255),
        help = "Double-click speed (0–255). Default: 7"
    )]
    pub double_click_speed: Option<u8>,

    #[arg(
        long,
        value_parser = clap::value_parser!(u8).range(0..=255),
        help = "Mouse scroll/rolling speed (0–255). Default: 3"
    )]
    pub rolling_speed: Option<u8>,

    #[command(flatten)]
    pub led_args: Option<LedArgs>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Args, Debug, Clone)]
pub struct FireControl {
    #[arg(
        short,
        long,
        value_parser = clap::value_parser!(u8).range(0..=255),
        help = "Auto-fire repeat count (0–255). Default: 3"
    )]
    pub repeat: Option<u8>,

    #[arg(
        short,
        long,
        value_parser = clap::value_parser!(u8).range(0..=255),
        help = "Delay between shots (0–255). Default: 6"
    )]
    pub firing_interval: Option<u8>,

    #[arg(long, help = "Enable/disable continuous firing. Default: Disable")]
    pub continously: Option<ContinouslyState>,
}

#[derive(Args, Debug, Clone)]
pub struct LedArgs {
    #[arg(long, help = "LED brightness (All/Half). Default: All")]
    pub led_brightness: Option<LedBrightness>,

    #[arg(long, help = "Breathing speed (1–8, higher = faster). Default: BS4")]
    pub breathing_speed: Option<BreathingSpeed>,
}

#[derive(Args, Debug, Clone)]
pub struct GlobalMouseOptions {
    #[command(flatten)]
    pub fire_control: Option<FireControl>,

    #[arg(short, long, value_parser = clap::value_parser!(u8).range(0..=255),
          help = "Mouse movement speed (0–255). Default: 6")]
    pub moving_speed: Option<u8>,

    #[arg(short, long, value_parser = clap::value_parser!(u8).range(0..=255),
          help = "Double-click speed (0–255). Default: 7")]
    pub double_click_speed: Option<u8>,

    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=255),
          help = "Mouse scroll/rolling speed (0–255). Default: 3")]
    pub rolling_speed: Option<u8>,

    #[command(flatten)]
    pub led_args: Option<LedArgs>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Set DPI level
    Dpi {
        #[command(flatten)]
        opts: GlobalMouseOptions,

        #[arg(
    help = concat!(
        "Choose DPI level\n\n",
        "  DPI Values:\n",
        "  +------+--------+\n",
        "  | Name | Value  |\n",
        "  +------+--------+\n",
        "  | dpi1 | 1000   |\n",
        "  | dpi2 | 1600   |\n",
        "  | dpi3 | 2400   |\n",
        "  | dpi4 | 3200   |\n",
        "  | dpi5 | 4800   |\n",
        "  | dpi6 | 6400   |\n",
        "  | dpi7 | 7200   |\n",
        "  | dpi8 | 8000   |\n",
        "  +------+--------+\n",
    )
)]
        dpi_val: DpiVal,
    },

    /// Set LED lighting mode
    Led {
        #[command(flatten)]
        opts: GlobalMouseOptions,

        #[arg(value_enum)]
        mode: LedMode,
    },

    /// Enable or disable LED lights
    LedStatus {
        #[command(flatten)]
        opts: GlobalMouseOptions,

        state: LedStatus,
    },

    #[command(about = "Reset all mouse settings to their default values")]
    Reset,
}
pub enum Reset {
    RepeatVal(u8),
    FiringInterval(u8),
    Continously(ContinouslyState),
    DpiVal(DpiVal),
    LedStatus(LedStatus),
    LedBrightness(LedBrightness),
    LedMode(LedMode),
    BreathingSpeed(BreathingSpeed),
}

pub fn reset_val() -> Vec<Reset> {
    vec![
        Reset::RepeatVal(3),
        Reset::FiringInterval(6),
        Reset::Continously(ContinouslyState::Disable),
        Reset::DpiVal(DpiVal::DPI6),
        Reset::LedStatus(LedStatus::Enable),
        Reset::LedBrightness(LedBrightness::All),
        Reset::LedMode(LedMode::Multi),
        Reset::BreathingSpeed(BreathingSpeed::BS6),
    ]
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ContinouslyState {
    Enable,
    Disable,
}

impl ContinouslyState {
    pub fn hex(&self) -> &'static str {
        match self {
            ContinouslyState::Enable => CONTINOUUSLY_ENABLED,
            ContinouslyState::Disable => CONTINOUUSLY_DISABLED,
        }
    }
}
#[derive(ValueEnum, Clone, Debug)]
pub enum LedBrightness {
    All,
    Half,
}

impl LedBrightness {
    pub fn hex(&self) -> (&'static str, &'static str) {
        match self {
            LedBrightness::All => LED_BRGT_FULL,
            LedBrightness::Half => LED_BRGT_HALF,
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum LedStatus {
    Enable,
    Disable,
}

impl LedStatus {
    pub fn hex(&self) -> &'static str {
        match self {
            LedStatus::Enable => LED_ENABLE,
            LedStatus::Disable => LED_DISABLE,
        }
    }
}

#[derive(Debug)]
pub struct MouseConfig {
    pub repeat: u8,
    pub firing_interval: u8,
    pub continously: ContinouslyState,
    pub moving_speed: u8,
    pub double_click_speed: u8,
    pub rolling_speed: u8,
    pub led_args: LedArgs,
    pub led_status: LedStatus,
    pub dpi: DpiVal,
    pub led_mode: LedMode,
    pub reset: bool,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            dpi: DpiVal::DPI2,
            led_mode: LedMode::Dpi,
            repeat: 3,
            firing_interval: 6,
            continously: ContinouslyState::Disable,
            moving_speed: 6,
            double_click_speed: 7,
            rolling_speed: 3,
            led_status: LedStatus::Enable,
            led_args: LedArgs {
                led_brightness: Some(LedBrightness::All),
                breathing_speed: Some(BreathingSpeed::BS4),
            },
            reset: false,
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum DpiVal {
    DPI1,
    DPI2,
    DPI3,
    DPI4,
    DPI5,
    DPI6,
    DPI7,
    DPI8,
}

impl DpiVal {
    pub fn hex(&self) -> &'static str {
        match self {
            DpiVal::DPI1 => DPI1,
            DpiVal::DPI2 => DPI2,
            DpiVal::DPI3 => DPI3,
            DpiVal::DPI4 => DPI4,
            DpiVal::DPI5 => DPI5,
            DpiVal::DPI6 => DPI6,
            DpiVal::DPI7 => DPI7,
            DpiVal::DPI8 => DPI8,
        }
    }
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

const DPI1: &str = "040700ff817e807f"; // 1000
const DPI2: &str = "040701fe817e807f"; // 1600
const DPI3: &str = "040702fd817e807f"; // 2400
const DPI4: &str = "040703fd817e807f"; // 3200
const DPI5: &str = "040704fd817e807f"; // 4800
const DPI6: &str = "040705fd817e807f"; // 6400
const DPI7: &str = "040706fd817e807f"; // 7200
const DPI8: &str = "040707fd817e807f"; // 8000

const CONTINOUUSLY_DISABLED: &str = "0407fdfffffc1bff";
const CONTINOUUSLY_ENABLED: &str = "0407fdfffffc64ff"; // Repeat shall be disabled - 04070afdffa1fe03
const LED_DISABLE: &str = "040701fe8976807f";
const LED_ENABLE: &str = "040701fe817e807f";
const LED_MODE_MULTI: &str = "040701fe827d807f";
const LED_MODE_RAINBOW: &str = "040701fe837c807f";
const LED_MODE_FLOE_LIGHT: &str = "040701fe847b807f";
const LED_MODE_WALTZ: &str = "040701fe857a807f";
const LED_MODE_FOUR_SEASONS: &str = "040701fe8679807f";
const LED_MODE_DPI: &str = "040701fe817e807f";
const LED_MODE_OFF: &str = "040701fe8778807f";
const LED_BRGT_FULL: (&str, &str) = ("040745f80638ff00", "0407ff00ffffff71");
const LED_BRGT_HALF: (&str, &str) = ("040745f80630ff00", "0407ff00ffffff79");

macro_rules! generate_hex_val_for_repeat {
    (
        $REPEAT_REQ: expr,
        $FULL_HEX: expr
    ) => {{
        let hex_val = "04070afd03a1fe03";
        let repeat_req_in_hex = hex::encode([$REPEAT_REQ]);
        let final_val = hex_val.replace("fd03", format!("fd{}", repeat_req_in_hex).as_str());
        let final_hex: Vec<String> = $FULL_HEX
            .iter()
            .map(|&x| x.replace(hex_val, final_val.as_str()))
            .collect();
        final_hex
    }};
}

macro_rules! generate_hex_for_interval {
    (
        $REPEAT_REQ: expr,
        $FULL_HEX: expr
    ) => {{
        let hex_val = "040721fe08fc94ff";
        let firing_interval_req_in_hex = hex::encode([$REPEAT_REQ]);
        let final_val =
            hex_val.replace("fe08", format!("fe{}", firing_interval_req_in_hex).as_str());
        let final_hex: Vec<String> = $FULL_HEX
            .iter()
            .map(|x| x.replace(hex_val, final_val.as_str()))
            .collect();
        final_hex
    }};
}

macro_rules! gen_hex_for_led {
    ($MODE:expr, $FULL_HEX:expr) => {{
        let mode_hex = $MODE.hex();
        let output: Vec<String> = $FULL_HEX
            .iter()
            .map(|x| x.replace("040701fe817e807f", mode_hex))
            .collect();
        output
    }};
}

macro_rules! gen_hex_for_dpi {
    (
        $MODE: expr,
        $FULL_HEX: expr
    ) => {{
        let mod_hex = $MODE.hex();
        let output: Vec<String> = $FULL_HEX
            .iter()
            .map(|x| x.replace("040701fe817e807f", mod_hex))
            .collect();
        output
    }};
}

macro_rules! gen_hex_for_led_brgt {
    (
        $MODE: expr,
        $FULL_HEX: expr
    ) => {{
        let (first_hex, second_hex) = $MODE.hex();
        let output: Vec<String> = $FULL_HEX
            .iter()
            .map(|x| x.replace("040745f80638ff00", first_hex))
            .map(|y| y.replace("0407ff00ffffff71", second_hex))
            .collect();
        output
    }};
}

macro_rules! gen_hex_for_breathing_speed {
    (
        $MODE: expr,
        $FULL_HEX: expr
    ) => {{
        let breathing_hex = $MODE.hex();
        let output: Vec<String> = $FULL_HEX
            .iter()
            .map(|x| x.replace("040701fe817e807f", breathing_hex))
            .collect();
        output
    }};
}

macro_rules! gen_hex_for_continously {
    (
        $MODE: expr,
        $FULL_HEX: expr
    ) => {{
        let continously_hex = $MODE.hex();
        if continously_hex == "0407fdfffffc64ff" {
            let output: Vec<String> = $FULL_HEX
                .iter()
                .map(|x| x.replace("0407fdfffffc94ff", continously_hex))
                .map(|y| y.replace("04070afd03a1fe03", "04070afdffa1fe03"))
                .collect();
            output
        } else {
            let output: Vec<String> = $FULL_HEX
                .iter()
                .map(|x| x.replace("0407fdfffffc94ff", continously_hex))
                .collect();
            output
        }
    }};
}

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

const BREATHING_SPEED_HEX: [&str; 8] = [
    "040701fee11e807f",
    "040701fec13e807f",
    "040701fea15e807f",
    "040701fe817e807f",
    "1040701fe619e807f",
    "040701fe41be807f",
    "040701fe21de807f",
    "040701fe01fe807f",
];

#[derive(Clone, Debug, ValueEnum)]
pub enum BreathingSpeed {
    BS1,
    BS2,
    BS3,
    BS4,
    BS5,
    BS6,
    BS7,
    BS8,
}
impl BreathingSpeed {
    pub fn hex(&self) -> &'static str {
        match self {
            BreathingSpeed::BS1 => BREATHING_SPEED_HEX[0],
            BreathingSpeed::BS2 => BREATHING_SPEED_HEX[1],
            BreathingSpeed::BS3 => BREATHING_SPEED_HEX[2],
            BreathingSpeed::BS4 => BREATHING_SPEED_HEX[3],
            BreathingSpeed::BS5 => BREATHING_SPEED_HEX[4],
            BreathingSpeed::BS6 => BREATHING_SPEED_HEX[5],
            BreathingSpeed::BS7 => BREATHING_SPEED_HEX[6],
            BreathingSpeed::BS8 => BREATHING_SPEED_HEX[7],
        }
    }
}

const COMMON_HEX: [&str; 48] = [
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
    "04070afd03a1fe03",
    "040721fe08fc94ff",
    "0407fdfffffc94ff",
    "0408000000000000",
    "0402000000000000",
];

fn main() -> Result<()> {
    let args = MouseArgs::parse();
    let default_val = MouseConfig::default();
    let mut repeat = default_val.repeat;
    let mut firing_interval = default_val.firing_interval;
    let led_args = default_val.led_args;
    let led_brightness = led_args.led_brightness.unwrap();
    let breathing_speed = led_args.breathing_speed.unwrap();

    if let Some(fire_control) = args.fire_control.clone() {
        repeat = fire_control.repeat.unwrap_or_default()
    };

    let repeat_hex = generate_hex_val_for_repeat!(repeat, COMMON_HEX);

    if let Some(firing_control) = args.fire_control.clone() {
        firing_interval = firing_control.firing_interval.unwrap_or_default()
    };

    let firing_interval_hex = generate_hex_for_interval!(firing_interval, repeat_hex.clone());

    let led_brght_hex = if let Some(LedArgs {
        led_brightness: Some(led_brightness),
        breathing_speed: None,
    }) = args.led_args.clone()
    {
        gen_hex_for_led_brgt!(led_brightness, firing_interval_hex.clone())
    } else {
        gen_hex_for_led_brgt!(led_brightness, firing_interval_hex.clone())
    };

    let breathing_speed_hex = if let Some(LedArgs {
        led_brightness: None,
        breathing_speed: Some(breathing_speed),
    }) = args.led_args.clone()
    {
        gen_hex_for_breathing_speed!(breathing_speed, led_brght_hex.clone())
    } else {
        gen_hex_for_breathing_speed!(breathing_speed, led_brght_hex.clone())
    };

    let final_hex = if let Some(commands) = args.command.clone() {
        match commands {
            Commands::Dpi { dpi_val, .. } => {
                gen_hex_for_dpi!(dpi_val, breathing_speed_hex)
            }
            Commands::Led { .. } => {
                let led_mode = if let Some(Commands::Led { mode, .. }) = args.command {
                    mode
                } else {
                    default_val.led_mode
                };
                gen_hex_for_led!(led_mode, breathing_speed_hex.clone())
            }
            Commands::LedStatus { state, .. } => {
                gen_hex_for_led!(state, breathing_speed_hex.clone())
            }
            Commands::Reset => {
                let mut reset_hex = Vec::new();
                for val in reset_val() {
                    match val {
                        Reset::RepeatVal(repeat) => {
                            reset_hex = generate_hex_val_for_repeat!(repeat, COMMON_HEX)
                        }
                        Reset::FiringInterval(firing_interval) => {
                            reset_hex = generate_hex_for_interval!(firing_interval, reset_hex)
                        }
                        Reset::Continously(cstate) => {
                            reset_hex = gen_hex_for_continously!(cstate, reset_hex)
                        }
                        Reset::DpiVal(dpival) => reset_hex = gen_hex_for_dpi!(dpival, reset_hex),
                        Reset::LedStatus(lstatus) => {
                            reset_hex = gen_hex_for_led!(lstatus, reset_hex)
                        }
                        Reset::LedBrightness(led_brightness) => {
                            reset_hex = gen_hex_for_led_brgt!(led_brightness, reset_hex)
                        }
                        Reset::LedMode(led_mode) => {
                            reset_hex = gen_hex_for_led!(led_mode, reset_hex)
                        }
                        Reset::BreathingSpeed(breathing_speed) => {
                            reset_hex = gen_hex_for_breathing_speed!(breathing_speed, reset_hex)
                        }
                    }
                }
                reset_hex
            }
        }
    } else if let Some(fire_control_commands) = args.fire_control {
        match fire_control_commands {
            FireControl {
                repeat: Some(repeat),
                firing_interval: None,
                ..
            } => {
                println!("Called Repeat!!");
                generate_hex_val_for_repeat!(repeat, COMMON_HEX)
            }

            FireControl {
                repeat: None,
                firing_interval: Some(interval),
                ..
            } => {
                println!("Called firing_interval!!");

                generate_hex_for_interval!(interval, COMMON_HEX)
            }

            FireControl {
                repeat: Some(repeat),
                firing_interval: Some(interval),
                ..
            } => {
                println!("Lil bro called both!");
                let repeat_hex = generate_hex_val_for_repeat!(repeat, COMMON_HEX);
                generate_hex_for_interval!(interval, repeat_hex)
            }

            FireControl {
                continously: Some(continously),
                ..
            } => {
                match continously {
                    ContinouslyState::Enable => {
                        println!("Enabling Continously makes repeat disabled!");
                    }
                    ContinouslyState::Disable => {
                        println!("Called continously!!");
                    }
                }
                gen_hex_for_continously!(continously, repeat_hex)
            }

            _ => Vec::new(),
        }
    } else if let Some(led_args) = args.led_args {
        match led_args {
            LedArgs {
                breathing_speed: Some(breathing_speed),
                ..
            } => {
                println!("Called Breathing Speed");
                gen_hex_for_breathing_speed!(breathing_speed, COMMON_HEX)
            }
            _ => Vec::new(),
        }
    } else if let Some(..) = args.moving_speed {
        // the instructions seems to be the same for every operation..
        // meaning that it is not `changing` anything when increasing or decreasing.
        todo!()
    } else if let Some(..) = args.double_click_speed {
        // Same as moving_speed
        todo!()
    } else if let Some(..) = args.rolling_speed {
        // Same as moving_speed
        todo!()
    } else {
        eprintln!("Error: No Args Provided, use --help");
        exit(1);
    };

    let packets: Vec<Vec<u8>> = final_hex
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
