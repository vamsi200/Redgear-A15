use crate::protocol::{Operation, generate_packets};
use anyhow::{Context, Result};
use hidapi::HidApi;
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

const VID: u16 = 0x1bcf;
const PID: u16 = 0x08a0;

pub enum DeviceEvent {
    Started {
        operation: String,
        total: usize,
    },
    Progress {
        sent: usize,
        total: usize,
        tx: String,
        rx: Option<String>,
    },
    Completed,
    Failed(String),
}

pub struct Mouse;
impl Mouse {
    pub fn is_connected() -> bool {
        HidApi::new()
            .map(|api| {
                api.device_list()
                    .any(|d| d.vendor_id() == VID && d.product_id() == PID)
            })
            .unwrap_or(false)
    }

    pub fn apply_async(operation: Operation) -> Receiver<DeviceEvent> {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || match Self::apply(operation, &tx) {
            Ok(()) => {
                let _ = tx.send(DeviceEvent::Completed);
            }
            Err(error) => {
                let _ = tx.send(DeviceEvent::Failed(format!("{error:#}")));
            }
        });
        rx
    }

    fn apply(operation: Operation, tx: &Sender<DeviceEvent>) -> Result<()> {
        // Validate the entire sequence before opening or writing to the device.
        let packets = generate_packets(operation)?;
        let total = packets.len();
        let api = HidApi::new()?;
        let dev = api.open(VID, PID).context("Cannot open Redgear A15")?;
        let _ = tx.send(DeviceEvent::Started {
            operation: operation.name().into(),
            total,
        });
        for (i, packet) in packets.iter().enumerate() {
            dev.send_feature_report(packet)
                .with_context(|| format!("SET_REPORT {}/{} failed", i + 1, total))?;
            thread::sleep(Duration::from_millis(120));
            let mut report = packet.clone();
            let response = dev
                .get_feature_report(&mut report)
                .ok()
                .map(|n| hex::encode_upper(&report[..n.min(report.len())]));
            let _ = tx.send(DeviceEvent::Progress {
                sent: i + 1,
                total,
                tx: hex::encode_upper(packet),
                rx: response,
            });
        }
        Ok(())
    }
}
