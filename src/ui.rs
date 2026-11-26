use raylib::prelude::*;

use std::{
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use crate::{
    mouse::{DeviceEvent, Mouse},
    protocol::{DPI_VALUES, LED_MODES, Operation, PACKET_COUNT},
};

pub const WIDTH: i32 = 1120;
pub const HEIGHT: i32 = FOOTER_Y + 60;

const BG: Color = Color::new(174, 179, 183, 255);
const PANEL: Color = Color::new(194, 198, 201, 255);
const BORDER: Color = Color::new(103, 111, 119, 255);
const ACCENT: Color = Color::new(43, 73, 112, 255);
const TEXT: Color = Color::new(32, 37, 42, 255);
const MUTED: Color = Color::new(76, 83, 90, 255);
const ERROR: Color = Color::new(151, 35, 39, 255);
const LIGHT: Color = Color::new(239, 242, 244, 255);
const WHITE: Color = Color::new(255, 255, 255, 255);

const RECESSED: Color = Color::new(213, 217, 219, 255);
const LED_ON: Color = Color::new(76, 139, 65, 255);
const LED_OFF: Color = Color::new(166, 54, 54, 255);

const TOP_Y: i32 = 168;
const TOP_H: i32 = 254;
const ROW_GAP: i32 = 28;
const MIDDLE_Y: i32 = TOP_Y + TOP_H + ROW_GAP;
const MIDDLE_H: i32 = 184;
const TRANSMISSION_Y: i32 = MIDDLE_Y + MIDDLE_H + ROW_GAP;
const TRANSMISSION_H: i32 = 154;
const FOOTER_Y: i32 = TRANSMISSION_Y + TRANSMISSION_H + 12;
const FONT_SMALL: f32 = 24.0;
const FONT_NORMAL: f32 = 24.0;
const FONT_HEADING: f32 = 24.0;
const FONT_DISPLAY: f32 = 24.0;
const FONT_SPACING: f32 = 1.0;

pub struct Ui {
    mouse_icon: Texture2D,
    font: Font,
    reset_confirm: bool,
    dpi: usize,
    led_mode: usize,
    led_enabled: bool,
    brightness_full: bool,
    breathing_speed: usize,

    repeat: u8,
    interval: u8,
    continuous: bool,

    connected: bool,
    busy: bool,

    sent: usize,
    total: usize,

    status: String,
    last_tx: String,
    last_rx: String,
    failed: bool,

    last_scan: Instant,
    events: Option<Receiver<DeviceEvent>>,
}

impl Ui {
    pub fn new(font: Font, mouse_icon: Texture2D) -> Self {
        Self {
            mouse_icon,
            font,
            reset_confirm: false,
            dpi: 1,
            led_mode: 0,
            led_enabled: true,
            brightness_full: true,
            breathing_speed: 3,

            repeat: 3,
            interval: 6,
            continuous: false,

            connected: Mouse::is_connected(),
            busy: false,

            sent: 0,
            total: PACKET_COUNT,

            status: "Ready".into(),
            last_tx: "-".into(),
            last_rx: "-".into(),
            failed: false,

            last_scan: Instant::now(),
            events: None,
        }
    }

    fn apply(&mut self, operation: Operation) {
        if self.busy || !self.connected {
            return;
        }

        self.reset_confirm = false;
        self.busy = true;
        self.failed = false;
        self.sent = 0;

        self.last_tx.clear();
        self.last_rx.clear();

        self.status = format!("PREPARING: {}", operation.name());

        self.events = Some(Mouse::apply_async(operation));
    }

    pub fn update(&mut self) {
        if !self.busy && self.last_scan.elapsed() >= Duration::from_secs(2) {
            self.connected = Mouse::is_connected();
            self.last_scan = Instant::now();
        }

        let events: Vec<DeviceEvent> = self
            .events
            .as_ref()
            .map(|rx| rx.try_iter().collect())
            .unwrap_or_default();

        for event in events {
            match event {
                DeviceEvent::Started { operation, total } => {
                    self.total = total;
                    self.sent = 0;

                    self.status = format!("WRITING: {operation}");
                }

                DeviceEvent::Progress {
                    sent,
                    total,
                    tx,
                    rx,
                } => {
                    self.sent = sent;
                    self.total = total;
                    self.last_tx = tx;

                    self.last_rx = rx.unwrap_or_else(|| "READ FAILED".into());
                }

                DeviceEvent::Completed => {
                    self.busy = false;
                    self.events = None;

                    self.status = "REPORTS SENT!".into();
                }

                DeviceEvent::Failed(error) => {
                    self.busy = false;
                    self.failed = true;
                    self.events = None;
                    self.status = error;
                }
            }
        }
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle) {
        d.clear_background(BG);

        self.draw_header(d);

        self.draw_sensor(d);
        self.draw_lighting(d);
        self.draw_fire_control(d);
        self.draw_unsupported(d);
        self.draw_transmission(d);
        self.draw_footer(d);
    }

    fn draw_header(&self, d: &mut RaylibDrawHandle) {
        bevel(d, 4, 4, WIDTH - 8, HEIGHT - 8, BG, false);

        d.draw_rectangle(10, 10, WIDTH - 20, 38, ACCENT);

        text(d, &self.font, "Redgear A15", 20, 16, FONT_NORMAL, WHITE);

        let scale = 64.0 / self.mouse_icon.width.max(self.mouse_icon.height) as f32;

        let icon_width = self.mouse_icon.width as f32 * scale;
        let icon_height = self.mouse_icon.height as f32 * scale;

        d.draw_texture_ex(
            &self.mouse_icon,
            Vector2::new(
                (24.0 + (64.0 - icon_width) / 2.0).round(),
                (60.0 + (64.0 - icon_height) / 2.0).round(),
            ),
            0.0,
            scale,
            WHITE,
        );

        d.draw_line(102, 60, 102, 124, BORDER);
        d.draw_line(103, 60, 103, 124, LIGHT);

        text(d, &self.font, "Mouse settings", 118, 66, FONT_HEADING, TEXT);

        text(
            d,
            &self.font,
            "USB HID - 1BCF:08A0",
            118,
            98,
            FONT_SMALL,
            MUTED,
        );

        let (status, color) = if self.connected {
            ("DEVICE ONLINE", LED_ON)
        } else {
            ("DEVICE NOT FOUND", LED_OFF)
        };

        let size = self.font.measure_text(status, FONT_SMALL, FONT_SPACING);

        let status_x = WIDTH - 26 - size.x.ceil() as i32;
        let indicator_x = status_x - 28;

        bevel(d, indicator_x, 84, 16, 16, BORDER, true);

        d.draw_rectangle(indicator_x + 3, 87, 10, 10, color);

        text(d, &self.font, status, status_x, 80, FONT_SMALL, TEXT);
    }

    fn draw_sensor(&mut self, d: &mut RaylibDrawHandle) {
        let y = TOP_Y;

        panel(d, &self.font, 20, y, 438, TOP_H, "01. Optical sensor");

        bevel(d, 34, y + 34, 410, 60, RECESSED, true);

        let dpi_text = format!("{} DPI", DPI_VALUES[self.dpi]);
        let level_text = format!("LEVEL {:02}", self.dpi + 1);

        text(d, &self.font, &dpi_text, 48, y + 51, FONT_DISPLAY, TEXT);

        let level_size = self
            .font
            .measure_text(&level_text, FONT_NORMAL, FONT_SPACING);

        text(
            d,
            &self.font,
            &level_text,
            430 - level_size.x.ceil() as i32,
            y + 51,
            FONT_NORMAL,
            MUTED,
        );

        for (index, dpi) in DPI_VALUES.iter().enumerate() {
            let bx = 34 + (index as i32 % 4) * 104;
            let by = y + 104 + (index as i32 / 4) * 46;

            if button(
                d,
                &self.font,
                bx,
                by,
                98,
                40,
                &dpi.to_string(),
                self.dpi == index,
                !self.busy,
            ) {
                self.dpi = index;
            }
        }

        if button(
            d,
            &self.font,
            34,
            y + 202,
            410,
            40,
            "APPLY",
            false,
            self.can_apply(),
        ) {
            self.apply(Operation::Dpi(self.dpi));
        }
    }

    fn draw_lighting(&mut self, d: &mut RaylibDrawHandle) {
        panel(d, &self.font, 470, TOP_Y, 630, TOP_H, "02. Illumination");

        let enabled = !self.busy;
        let apply = self.can_apply();

        let labels = ["LED power", "Mode", "Brightness", "Breathing"];

        for (i, label) in labels.iter().enumerate() {
            text(
                d,
                &self.font,
                label,
                484,
                TOP_Y + 41 + i as i32 * 51,
                FONT_NORMAL,
                TEXT,
            );
        }

        // LED power
        if button(
            d,
            &self.font,
            626,
            TOP_Y + 34,
            154,
            40,
            "Disabled",
            !self.led_enabled,
            enabled,
        ) {
            self.led_enabled = false;
        }

        if button(
            d,
            &self.font,
            786,
            TOP_Y + 34,
            154,
            40,
            "Enabled",
            self.led_enabled,
            enabled,
        ) {
            self.led_enabled = true;
        }

        if button(d, &self.font, 954, TOP_Y + 34, 132, 40, "SET", false, apply) {
            self.apply(Operation::LedStatus(self.led_enabled));
        }

        // Lighting mode
        if button(d, &self.font, 626, TOP_Y + 85, 38, 40, "<", false, enabled) {
            self.led_mode = (self.led_mode + LED_MODES.len() - 1) % LED_MODES.len();
        }

        field(
            d,
            &self.font,
            668,
            TOP_Y + 85,
            230,
            40,
            LED_MODES[self.led_mode],
            enabled,
        );

        if button(d, &self.font, 902, TOP_Y + 85, 38, 40, ">", false, enabled) {
            self.led_mode = (self.led_mode + 1) % LED_MODES.len();
        }

        if button(d, &self.font, 954, TOP_Y + 85, 132, 40, "SET", false, apply) {
            self.apply(Operation::LedMode(self.led_mode));
        }

        // Brightness
        if button(
            d,
            &self.font,
            626,
            TOP_Y + 136,
            154,
            40,
            "Half",
            !self.brightness_full,
            enabled,
        ) {
            self.brightness_full = false;
        }

        if button(
            d,
            &self.font,
            786,
            TOP_Y + 136,
            154,
            40,
            "All",
            self.brightness_full,
            enabled,
        ) {
            self.brightness_full = true;
        }

        if button(
            d,
            &self.font,
            954,
            TOP_Y + 136,
            132,
            40,
            "SET",
            false,
            apply,
        ) {
            self.apply(Operation::Brightness(self.brightness_full));
        }

        // Breathing speed
        if button(d, &self.font, 626, TOP_Y + 187, 38, 40, "-", false, enabled) {
            self.breathing_speed = self.breathing_speed.saturating_sub(1);
        }

        let breath = format!("{}/8", self.breathing_speed + 1);

        field(d, &self.font, 668, TOP_Y + 187, 230, 40, &breath, enabled);

        if button(d, &self.font, 902, TOP_Y + 187, 38, 40, "+", false, enabled) {
            self.breathing_speed = (self.breathing_speed + 1).min(7);
        }

        if button(
            d,
            &self.font,
            954,
            TOP_Y + 187,
            132,
            40,
            "SET",
            false,
            apply && self.breathing_speed != 4,
        ) {
            self.apply(Operation::BreathingSpeed(self.breathing_speed));
        }
    }

    fn draw_fire_control(&mut self, d: &mut RaylibDrawHandle) {
        panel(
            d,
            &self.font,
            20,
            MIDDLE_Y,
            550,
            MIDDLE_H,
            "03. Fire control",
        );
        let enabled = !self.busy;
        let apply = self.can_apply();
        text(
            d,
            &self.font,
            "Repeat",
            34,
            MIDDLE_Y + 40,
            FONT_NORMAL,
            TEXT,
        );
        stepper(
            d,
            &self.font,
            188,
            MIDDLE_Y + 34,
            &mut self.repeat,
            enabled && !self.continuous,
        );
        if button(
            d,
            &self.font,
            430,
            MIDDLE_Y + 34,
            126,
            40,
            "SET",
            false,
            apply && !self.continuous,
        ) {
            self.apply(Operation::Repeat(self.repeat));
        }
        text(
            d,
            &self.font,
            "Interval",
            34,
            MIDDLE_Y + 90,
            FONT_NORMAL,
            TEXT,
        );
        stepper(
            d,
            &self.font,
            188,
            MIDDLE_Y + 84,
            &mut self.interval,
            enabled,
        );
        if button(
            d,
            &self.font,
            430,
            MIDDLE_Y + 84,
            126,
            40,
            "SET",
            false,
            apply,
        ) {
            self.apply(Operation::FiringInterval(self.interval));
        }
        text(
            d,
            &self.font,
            "Continuous",
            34,
            MIDDLE_Y + 140,
            FONT_NORMAL,
            TEXT,
        );
        if button(
            d,
            &self.font,
            188,
            MIDDLE_Y + 134,
            106,
            40,
            "OFF",
            !self.continuous,
            enabled,
        ) {
            self.continuous = false;
        }
        if button(
            d,
            &self.font,
            300,
            MIDDLE_Y + 134,
            106,
            40,
            "ON",
            self.continuous,
            enabled,
        ) {
            self.continuous = true;
        }
        if button(
            d,
            &self.font,
            430,
            MIDDLE_Y + 134,
            126,
            40,
            "SET",
            false,
            apply,
        ) {
            self.apply(Operation::Continuous(self.continuous));
        }
    }

    fn draw_unsupported(&mut self, d: &mut RaylibDrawHandle) {
        panel(
            d,
            &self.font,
            582,
            MIDDLE_Y,
            518,
            MIDDLE_H,
            "04. Pointer settings",
        );
        for (i, (label, value)) in [
            ("Movement", "6"),
            ("Double-click", "7"),
            ("Scroll speed", "3"),
        ]
        .iter()
        .enumerate()
        {
            let y = MIDDLE_Y + 34 + i as i32 * 50;
            text(d, &self.font, label, 596, y + 6, FONT_NORMAL, MUTED);
            field(d, &self.font, 820, y, 112, 40, value, false);
            text(d, &self.font, "N/A", 964, y + 6, FONT_NORMAL, MUTED);
        }
    }

    fn draw_transmission(&self, d: &mut RaylibDrawHandle) {
        let y = TRANSMISSION_Y;

        panel(
            d,
            &self.font,
            20,
            y,
            1080,
            TRANSMISSION_H,
            "05. HID transmission",
        );

        text(
            d,
            &self.font,
            "FEATURE REPORTS",
            34,
            y + 32,
            FONT_SMALL,
            TEXT,
        );

        let count = format!("{:02} / {:02}", self.sent, self.total);

        let count_size = self.font.measure_text(&count, FONT_SMALL, FONT_SPACING);

        text(
            d,
            &self.font,
            &count,
            1086 - count_size.x.ceil() as i32,
            y + 32,
            FONT_SMALL,
            TEXT,
        );

        bevel(d, 34, y + 62, 1052, 24, RECESSED, true);

        let inner_width = 1040;
        let gap = 4;
        let usable = inner_width - gap * (PACKET_COUNT as i32 - 1);

        for index in 0..PACKET_COUNT {
            let i = index as i32;
            let start = i * usable / PACKET_COUNT as i32;
            let end = (i + 1) * usable / PACKET_COUNT as i32;
            let x = 40 + start + i * gap;
            let color = if index < self.sent { ACCENT } else { PANEL };
            d.draw_rectangle(x, y + 67, end - start, 14, color);
        }

        // Feature report values.
        bevel(d, 34, y + 92, 1052, 52, RECESSED, true);

        text(
            d,
            &self.font,
            &format!("TX > {}", self.last_tx),
            44,
            y + 95,
            FONT_SMALL,
            TEXT,
        );

        text(
            d,
            &self.font,
            &format!("RX < {}", self.last_rx),
            44,
            y + 119,
            FONT_SMALL,
            TEXT,
        );
    }

    fn draw_footer(&mut self, d: &mut RaylibDrawHandle) {
        let x = 20;
        let y = FOOTER_Y;
        let w = WIDTH - 40;
        let h = 48;

        let right = x + w - 6;
        let confirming = self.reset_confirm;

        let status_end = if confirming {
            if button(
                d,
                &self.font,
                right - 286,
                y + 6,
                150,
                36,
                "CONFIRM",
                false,
                self.can_apply(),
            ) {
                self.apply(Operation::Reset);
            }

            if button(
                d,
                &self.font,
                right - 130,
                y + 6,
                130,
                36,
                "CANCEL",
                false,
                true,
            ) {
                self.reset_confirm = false;
            }

            right - 298
        } else {
            if button(
                d,
                &self.font,
                right - 220,
                y + 6,
                220,
                36,
                "FACTORY RESET",
                false,
                self.can_apply(),
            ) {
                self.reset_confirm = true;
            }

            right - 232
        };

        let color = if self.failed {
            ERROR
        } else if self.busy {
            ACCENT
        } else {
            TEXT
        };

        let status = if self.reset_confirm {
            "Send original reset sequence?"
        } else {
            &self.status
        };

        let size = self.font.measure_text(status, FONT_SMALL, FONT_SPACING);

        let tx = x + 12;
        let ty = y + (h - size.y.ceil() as i32) / 2;

        d.draw_scissor_mode(tx, y + 4, status_end - tx, h - 8, |mut d| {
            d.draw_text_ex(
                &self.font,
                status,
                Vector2::new(tx as f32, ty as f32),
                FONT_SMALL,
                FONT_SPACING,
                color,
            );
        });
    }

    fn can_apply(&self) -> bool {
        self.connected && !self.busy
    }
}

fn text(
    d: &mut RaylibDrawHandle,
    font: &Font,
    label: &str,
    x: i32,
    y: i32,
    size: f32,
    color: Color,
) {
    d.draw_text_ex(
        font,
        label,
        Vector2::new(x as f32, y as f32),
        size,
        FONT_SPACING,
        color,
    );
}

fn bevel(d: &mut RaylibDrawHandle, x: i32, y: i32, w: i32, h: i32, fill: Color, inset: bool) {
    d.draw_rectangle(x, y, w, h, fill);
    let (top, bottom) = if inset {
        (BORDER, LIGHT)
    } else {
        (LIGHT, BORDER)
    };
    for i in 0..2 {
        d.draw_line(x + i, y + i, x + w - 1 - i, y + i, top);
        d.draw_line(x + i, y + i, x + i, y + h - 1 - i, top);
        d.draw_line(x + i, y + h - 1 - i, x + w - 1 - i, y + h - 1 - i, bottom);
        d.draw_line(x + w - 1 - i, y + i, x + w - 1 - i, y + h - 1 - i, bottom);
    }
}

fn panel(d: &mut RaylibDrawHandle, font: &Font, x: i32, y: i32, w: i32, h: i32, title: &str) {
    let size = font.measure_text(title, FONT_NORMAL, FONT_SPACING);

    let tx = x + 18;
    let ty = y - (size.y / 2.0).round() as i32;

    let left = tx - 6;
    let right = tx + size.x.ceil() as i32 + 6;

    d.draw_rectangle(x, y, w, h, BG);

    for i in 0..2 {
        let x1 = x + i;
        let y1 = y + i;
        let x2 = x + w - 1 - i;
        let y2 = y + h - 1 - i;

        d.draw_line(x1, y1, left, y1, BORDER);
        d.draw_line(right, y1, x2, y1, BORDER);

        d.draw_line(x1, y1, x1, y2, BORDER);
        d.draw_line(x1, y2, x2, y2, LIGHT);
        d.draw_line(x2, y1, x2, y2, LIGHT);
    }

    text(d, font, title, tx, ty, FONT_NORMAL, TEXT);
}

fn centered_text(
    d: &mut RaylibDrawHandle,
    font: &Font,
    label: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: Color,
) {
    let size = font.measure_text(label, FONT_NORMAL, FONT_SPACING);
    let pos = Vector2::new(
        (x as f32 + (w as f32 - size.x) / 2.0).round(),
        (y as f32 + (h as f32 - size.y) / 2.0).round(),
    );
    d.draw_text_ex(font, label, pos, FONT_NORMAL, FONT_SPACING, color);
}

fn field(
    d: &mut RaylibDrawHandle,
    font: &Font,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    label: &str,
    enabled: bool,
) {
    let fill = if enabled { RECESSED } else { PANEL };
    bevel(d, x, y, w, h, fill, true);
    centered_text(
        d,
        font,
        label,
        x,
        y,
        w,
        h,
        if enabled { TEXT } else { MUTED },
    );
}

fn button(
    d: &mut RaylibDrawHandle,
    font: &Font,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    label: &str,
    selected: bool,
    enabled: bool,
) -> bool {
    let bounds = Rectangle::new(x as f32, y as f32, w as f32, h as f32);
    let hovered = enabled && bounds.check_collision_point_rec(d.get_mouse_position());
    let pressed = hovered && d.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);
    let fill = if !enabled {
        PANEL
    } else if selected {
        ACCENT
    } else if hovered {
        LIGHT
    } else {
        PANEL
    };
    bevel(d, x, y, w, h, fill, selected || pressed);
    let color = if !enabled {
        MUTED
    } else if selected {
        WHITE
    } else {
        TEXT
    };
    let shift = if pressed { 1 } else { 0 };
    centered_text(d, font, label, x + shift, y + shift, w, h, color);
    hovered && d.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT)
}

fn stepper(d: &mut RaylibDrawHandle, font: &Font, x: i32, y: i32, value: &mut u8, enabled: bool) {
    if button(d, font, x, y, 42, 40, "-", false, enabled) {
        *value = value.saturating_sub(1);
    }
    if enabled
        && Rectangle::new((x + 46) as f32, y as f32, 126.0, 40.0)
            .check_collision_point_rec(d.get_mouse_position())
    {
        let wheel = d.get_mouse_wheel_move();
        if wheel > 0.0 {
            *value = value.saturating_add(wheel.ceil() as u8);
        }
        if wheel < 0.0 {
            *value = value.saturating_sub((-wheel).ceil() as u8);
        }
    }
    field(
        d,
        font,
        x + 46,
        y,
        126,
        40,
        &format!("{:03}", value),
        enabled,
    );
    if button(d, font, x + 176, y, 42, 40, "+", false, enabled) {
        *value = value.saturating_add(1);
    }
}
