use anyhow::{Context, Result};

pub const DPI_VALUES: [u16; 8] = [1000, 1600, 2400, 3200, 4800, 6400, 7200, 8000];
pub const LED_MODES: [&str; 7] = [
    "DPI",
    "Multi",
    "Rainbow",
    "Floe Light",
    "Waltz",
    "Four Seasons",
    "Off",
];
pub const PACKET_COUNT: usize = 48;

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

const BREATHING_SPEED_HEX: [&str; 8] = [
    "040701fee11e807f",
    "040701fec13e807f",
    "040701fea15e807f",
    "040701fe817e807f",
    "040701fe619e807f",
    "040701fe41be807f",
    "040701fe21de807f",
    "040701fe01fe807f",
];

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

const DPI_HEX: [&str; 8] = [DPI1, DPI2, DPI3, DPI4, DPI5, DPI6, DPI7, DPI8];
const LED_MODE_HEX: [&str; 7] = [
    LED_MODE_DPI,
    LED_MODE_MULTI,
    LED_MODE_RAINBOW,
    LED_MODE_FLOE_LIGHT,
    LED_MODE_WALTZ,
    LED_MODE_FOUR_SEASONS,
    LED_MODE_OFF,
];

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Dpi(usize),
    LedMode(usize),
    LedStatus(bool),
    Brightness(bool),
    BreathingSpeed(usize),
    Repeat(u8),
    FiringInterval(u8),
    Continuous(bool),
    Reset,
}

impl Operation {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Dpi(_) => "DPI",
            Self::LedMode(_) => "LED MODE",
            Self::LedStatus(_) => "LED STATUS",
            Self::Brightness(_) => "LED BRIGHTNESS",
            Self::BreathingSpeed(_) => "BREATHING SPEED",
            Self::Repeat(_) => "REPEAT",
            Self::FiringInterval(_) => "FIRING INTERVAL",
            Self::Continuous(_) => "CONTINUOUS FIRING",
            Self::Reset => "FACTORY RESET (LEGACY)",
        }
    }
}

fn replace(packets: &mut [String], old: &str, new: &str) {
    for packet in packets {
        *packet = packet.replace(old, new);
    }
}

fn repeat(packets: &mut [String], value: u8) {
    replace(
        packets,
        "04070afd03a1fe03",
        &format!("04070afd{value:02x}a1fe03"),
    );
}

fn interval(packets: &mut [String], value: u8) {
    replace(
        packets,
        "040721fe08fc94ff",
        &format!("040721fe{value:02x}fc94ff"),
    );
}

fn continuous(packets: &mut [String], enabled: bool) {
    replace(
        packets,
        "0407fdfffffc94ff",
        if enabled {
            CONTINOUUSLY_ENABLED
        } else {
            CONTINOUUSLY_DISABLED
        },
    );
    if enabled {
        replace(packets, "04070afd03a1fe03", "04070afdffa1fe03");
    }
}

fn brightness(packets: &mut [String], full: bool) {
    let (first, second) = if full { LED_BRGT_FULL } else { LED_BRGT_HALF };
    replace(packets, LED_BRGT_FULL.0, first);
    replace(packets, LED_BRGT_FULL.1, second);
}

fn control(packets: &mut [String], value: &str) {
    replace(packets, LED_MODE_DPI, value);
}

fn command_defaults(packets: &mut [String]) {
    repeat(packets, 3);
    interval(packets, 6);
    brightness(packets, true);
    continuous(packets, false);
    control(packets, BREATHING_SPEED_HEX[3]);
}

pub fn generate_hex(operation: Operation) -> Result<Vec<String>> {
    let mut packets: Vec<String> = COMMON_HEX.iter().map(|s| (*s).to_owned()).collect();
    match operation {
        Operation::Dpi(index) => {
            command_defaults(&mut packets);
            control(
                &mut packets,
                DPI_HEX.get(index).context("Invalid DPI index")?,
            );
        }
        Operation::LedMode(index) => {
            command_defaults(&mut packets);
            control(
                &mut packets,
                LED_MODE_HEX.get(index).context("Invalid LED mode index")?,
            );
        }
        Operation::LedStatus(enabled) => {
            command_defaults(&mut packets);
            control(&mut packets, if enabled { LED_ENABLE } else { LED_DISABLE });
        }
        Operation::Brightness(full) => brightness(&mut packets, full),
        Operation::BreathingSpeed(index) => control(
            &mut packets,
            BREATHING_SPEED_HEX
                .get(index)
                .context("Invalid breathing speed index")?,
        ),
        Operation::Repeat(value) => repeat(&mut packets, value),
        Operation::FiringInterval(value) => interval(&mut packets, value),
        Operation::Continuous(enabled) => continuous(&mut packets, enabled),
        Operation::Reset => {
            repeat(&mut packets, 3);
            interval(&mut packets, 6);
            continuous(&mut packets, false);
            control(&mut packets, DPI6);
            control(&mut packets, LED_ENABLE);
            brightness(&mut packets, true);
            control(&mut packets, LED_MODE_MULTI);
            control(&mut packets, BREATHING_SPEED_HEX[5]);
        }
    }
    Ok(packets)
}

pub fn generate_packets(operation: Operation) -> Result<Vec<Vec<u8>>> {
    generate_hex(operation)?
        .iter()
        .enumerate()
        .map(|(index, value)| {
            hex::decode(value).with_context(|| format!("Invalid report {}: {}", index + 1, value))
        })
        .collect()
}
