use anyhow::Result;
use clap::{Args, Parser, ValueEnum};
use core::str;
use hex;
use hidapi::{HidApi, HidDevice};
use std::{process::exit, thread::sleep, time::Duration};

const VID: u16 = 0x1bcf;
const PID: u16 = 0x08a0;

#[derive(Parser, Debug)]
#[command(name = "Redgear-A15", version, about = "Control Redgear A-15 mouse")]
pub struct MouseArgs {
    #[arg(long = "no-confirm", help = "Apply changes without confirmation")]
    pub no_confirm: bool,

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

#[derive(Debug, Clone, Parser)]
pub struct LedArgs {
    #[arg(long, help = "LED brightness (All/Half). Default: All")]
    pub led_brightness: Option<LedBrightness>,

    #[arg(long, help = "Breathing speed (1–8, higher = faster). Default = 4")]
    pub breathing_speed: Option<BreathingSpeed>,
}

#[derive(Debug, Clone, Parser)]
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

#[derive(Debug, Clone, Parser)]
pub enum Commands {
    /// Set DPI level
    Dpi {
        #[command(flatten)]
        opts: GlobalMouseOptions,

        #[arg(help = r#"
        Choose DPI level

        DPI Values:
        ┌───────┬────────┐
        │ Name  │ Value  │
        ├───────┼────────┤
        │ 1     │ 1000   │
        │ 2     │ 1600   │
        │ 3     │ 2400   │
        │ 4     │ 3200   │
        │ 5     │ 4800   │
        │ 6     │ 6400   │
        │ 7     │ 7200   │
        │ 8     │ 8000   │
        └───────┴────────┘
        "#)]
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
#[derive(Clone, Debug)]
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

impl std::str::FromStr for LedBrightness {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "All" => Ok(LedBrightness::All),
            "Half" => Ok(LedBrightness::Half),
            _ => Err("Error: See --help for possible values"),
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

#[derive(Clone, Debug)]
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

impl str::FromStr for DpiVal {
    type Err = &'static str;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "1" => Ok(DpiVal::DPI1),
            "2" => Ok(DpiVal::DPI2),
            "3" => Ok(DpiVal::DPI3),
            "4" => Ok(DpiVal::DPI4),
            "5" => Ok(DpiVal::DPI5),
            "6" => Ok(DpiVal::DPI6),
            "7" => Ok(DpiVal::DPI7),
            "8" => Ok(DpiVal::DPI8),
            _ => Err("Error: See --help for possible values"),
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

// Yes.. these macros could just be functions.
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
            eprintln!("FATAL: Failed to send report: {e}");
            break;
        }

        sleep(Duration::from_millis(300));

        let mut report_id = pkts.clone();
        if let Err(e) = dev.get_feature_report(&mut report_id) {
            eprintln!("WARN: Failed to read report: {e}");
        } else {
            println!("< GET_REPORT {}", bytes_to_hex(&report_id));
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

#[derive(Clone, Debug)]
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
impl std::str::FromStr for BreathingSpeed {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "1" => Ok(Self::BS1),
            "2" => Ok(Self::BS2),
            "3" => Ok(Self::BS3),
            "4" => Ok(Self::BS4),
            "5" => Ok(Self::BS5),
            "6" => Ok(Self::BS6),
            "7" => Ok(Self::BS7),
            "8" => Ok(Self::BS8),
            _ => Err("Invalid breathing speed"),
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

const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";

fn main() -> Result<()> {
    use std::io::{self, Write};
    let args = MouseArgs::parse();
    let default_val = MouseConfig::default();
    let mut repeat = default_val.repeat;
    let mut firing_interval = default_val.firing_interval;
    let led_args = default_val.led_args;
    let led_brightness = led_args.led_brightness.unwrap();
    let breathing_speed = led_args.breathing_speed.unwrap();
    let mut changes: Vec<(String, String)> = Vec::new();
    let mut continously = default_val.continously;

    if let Some(fc) = args.fire_control.as_ref() {
        if let Some(rep) = fc.repeat {
            changes.push(("Repeat".into(), rep.to_string()));
            repeat = rep;
        }
        if let Some(intv) = fc.firing_interval {
            changes.push(("Firing Interval".into(), format!("{}", intv)));
            firing_interval = intv;
        }
        if let Some(cont) = &fc.continously {
            changes.push(("Continously".into(), format!("{:?}", cont)));
            continously = cont.to_owned();
        }
    }

    if let Some(led) = args.led_args.as_ref() {
        if let Some(br) = &led.led_brightness {
            changes.push(("LED Brightness".into(), format!("{:?}", br)));
        }
        if let Some(bs) = &led.breathing_speed {
            changes.push(("LED Breathing Speed".into(), format!("{:?}", bs)));
        }
    }

    if let Some(cmd) = args.command.as_ref() {
        match cmd {
            Commands::Dpi { dpi_val, .. } => {
                changes.push(("DPI".into(), format!("{:?}", dpi_val)));
            }
            Commands::Led { mode, .. } => {
                changes.push(("LED Mode".into(), format!("{:?}", mode)));
            }
            Commands::LedStatus { state, .. } => {
                changes.push(("LED Status".into(), format!("{:?}", state)));
            }
            Commands::Reset => {
                changes.push(("Reset".into(), "Factory Defaults".into()));
            }
        }
    }

    if args.moving_speed.is_some() {
        eprintln!(
            "{RED}{BOLD}Error:{RESET} Changing 'moving_speed' is not implemented. See notes on GitHub - https://github.com/vamsi200/Redgear-A15/tree/main#some-notes."
        );
        std::process::exit(1);
    }

    if args.double_click_speed.is_some() {
        eprintln!(
            "{RED}{BOLD}Error:{RESET} Changing 'double_click_speed' is not implemented. See notes on GitHub - https://github.com/vamsi200/Redgear-A15/tree/main#some-notes"
        );
        std::process::exit(1);
    }

    if args.rolling_speed.is_some() {
        eprintln!(
            "{RED}{BOLD}Error:{RESET} Changing 'rolling_speed' is not implemented. See notes on GitHub - https://github.com/vamsi200/Redgear-A15/tree/main#some-notes"
        );
        std::process::exit(1);
    }

    if !args.no_confirm {
        if changes.is_empty() {
            eprintln!("{RED}{BOLD}Error:{RESET} No changes detected. Nothing to apply.");
            std::process::exit(1);
        }

        if let Some(_) = changes.iter().find(|(x, _)| x == "Continously") {
            println!("{YELLOW}[INFO]{RESET} Enabling Continously makes repeat disabled!");
        }

        println!("\n{BOLD}{CYAN}Changes{RESET}");
        println!("{DIM}──────────────────────────────────────────{RESET}");

        for (field, value) in &changes {
            println!("{GREEN}+ {RESET}{BOLD}{}:{RESET} {}", field, value);
        }

        println!("{DIM}──────────────────────────────────────────{RESET}");

        print!("{BOLD}> Apply these changes?{RESET} {YELLOW}[y/N]{RESET}: ");
        io::stdout().flush().unwrap();

        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        if !matches!(buf.trim(), "y" | "Y") {
            println!("{RED}Aborted.{RESET}");
            return Ok(());
        }
    }

    let repeat_hex = generate_hex_val_for_repeat!(repeat, COMMON_HEX);

    let firing_interval_hex = generate_hex_for_interval!(firing_interval, repeat_hex.clone());

    let led_brght_hex = if let Some(LedArgs {
        led_brightness: Some(led_brght),
        breathing_speed: None,
    }) = args.led_args.clone()
    {
        gen_hex_for_led_brgt!(led_brght, firing_interval_hex.clone())
    } else {
        gen_hex_for_led_brgt!(led_brightness, firing_interval_hex.clone())
    };

    let continously_hex = if let Some(FireControl {
        continously: Some(cont),
        ..
    }) = args.fire_control.clone()
    {
        gen_hex_for_continously!(cont, led_brght_hex)
    } else {
        gen_hex_for_continously!(continously, led_brght_hex)
    };

    let breathing_speed_hex = if let Some(LedArgs {
        led_brightness: None,
        breathing_speed: Some(brgt_speed),
    }) = args.led_args.clone()
    {
        gen_hex_for_breathing_speed!(brgt_speed, continously_hex.clone())
    } else {
        gen_hex_for_breathing_speed!(breathing_speed, continously_hex.clone())
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
                generate_hex_val_for_repeat!(repeat, COMMON_HEX)
            }

            FireControl {
                repeat: None,
                firing_interval: Some(interval),
                ..
            } => {
                generate_hex_for_interval!(interval, COMMON_HEX)
            }

            FireControl {
                repeat: Some(repeat),
                firing_interval: Some(interval),
                ..
            } => {
                let repeat_hex = generate_hex_val_for_repeat!(repeat, COMMON_HEX);
                generate_hex_for_interval!(interval, repeat_hex)
            }

            FireControl {
                continously: Some(continously),
                ..
            } => {
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
                gen_hex_for_breathing_speed!(breathing_speed, COMMON_HEX)
            }
            _ => Vec::new(),
        }
    } else if let Some(..) = args.moving_speed {
        todo!()
    } else if let Some(..) = args.double_click_speed {
        todo!()
    } else if let Some(..) = args.rolling_speed {
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

    println!();
    if let Ok(_) = send_report_to_mouse(packets, dev) {
        println!("> All reports processed.");
    }

    Ok(())
}
