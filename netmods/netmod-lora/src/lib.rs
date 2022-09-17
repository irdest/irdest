// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

#[macro_use]
extern crate tracing;

use netmod::{Endpoint, Result, Frame, Target};

use async_std::{sync::Arc, task, channel, sync::Mutex};
use async_trait::async_trait;

use serde::{Serialize, Deserialize};

use std::time::Duration;
use serialport::TTYPort;
use std::io::prelude::*;

const RADIO_MTU: usize = 255;
const BUFFER_SIZE: usize = 32;
const IRDEST_MAGIC: u8 = 0xCA;

#[repr(u8)]
#[derive(Copy, Clone, Serialize, Deserialize)]
enum CtrlTypeCode {
    Invalid = 0x00,
    Data,
    RtxReq,
}

#[repr(packed)]
#[derive(Copy, Clone, Serialize, Deserialize)]
struct CtrlHeader {
    magic: u8,
    packet_type: CtrlTypeCode,
    length: u8,
}

#[repr(packed)]
#[derive(Serialize, Deserialize)]
struct LoraPacket<'a> {
    header: CtrlHeader,
    payload: &'a[u8],
}

pub struct LoraEndpoint {
    rx: channel::Receiver<Frame>,
    serial: Mutex<TTYPort>,
}

impl LoraEndpoint {
    pub fn spawn() -> Arc<Self> {
        let (tx, rx) = channel::bounded(BUFFER_SIZE);
        let serial = serialport::new("/dev/ttyUSB0", 115_200)
            .timeout(Duration::from_millis(10))
            .open_native().expect("Failed to open port");
        
        let this = Arc::new(Self{
                rx,
                serial: Mutex::new(serial),
            });

        task::spawn(Self::read_serial(this.clone(), tx));
        
        this
    }

    async fn read_serial(self: Arc<Self>, c: channel::Sender<Frame>) {
        let mut buffer: [u8; RADIO_MTU] = [0; RADIO_MTU];
        loop {
            self.serial.lock().await.read_exact(&mut buffer).unwrap();
            todo!("deserialise frame");
            let frame;
            c.send(frame).await;
        }
    }
}

#[async_trait]
impl Endpoint for LoraEndpoint {
    fn size_hint(&self) -> usize {
        RADIO_MTU - std::mem::size_of::<CtrlHeader>()
    }

    async fn send(&self, frame: Frame, target: Target, exclude: Option<u16>) -> Result<()> {
        if exclude.is_some() {
            return Ok(());
        }

        let ctrl = CtrlHeader {
            magic: IRDEST_MAGIC,
            packet_type: CtrlTypeCode::Data,
            length: self.size_hint() as u8,
        };

        todo!("serialise Frame");
        let buffer: [u8; RADIO_MTU] = [0; RADIO_MTU];

        self.serial.lock().await.write(&buffer).unwrap();

        Ok(())
    }

    async fn next(&self) -> Result<(Frame, Target)> {
        let frame = self.rx.recv().await.unwrap();
        Ok((frame, Target::Single(0)))
    }
}

