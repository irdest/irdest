// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use netmod::{Endpoint, Frame, Target};
use netmod::Result as NMResult;
use irdest_firmware_util::{decode_frame, encode_frame};

use async_std::{channel, sync::Arc, sync::Mutex, task};
use async_trait::async_trait;

use serialport::TTYPort;
use std::io::prelude::*;
use std::time::Duration;

const RADIO_MTU: usize = 255;
const BUFFER_SIZE: usize = 32;
const IRDEST_MAGIC: u8 = 0xCA;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum CtrlTypeCode {
    _Invalid = 0x00,
    Data,
    RtxReq,
}

#[repr(packed)]
#[derive(Debug)]
struct CtrlHeader {
    magic: u8,
    packet_type: CtrlTypeCode,
    length: u8,
}

const PAYLOAD_SIZE: usize = RADIO_MTU - std::mem::size_of::<CtrlHeader>();

#[derive(Debug)]
enum LoraPacketError {
    ControlCodeError(u8),
}

#[repr(packed)]
#[derive(Debug)]
struct LoraPacket {
    header: CtrlHeader,
    payload: [u8; PAYLOAD_SIZE],
}

impl LoraPacket {
    
    // HERE BE DRAGONS: ENDIANESS NOT HANDLED!
    fn encode(&self) -> [u8; RADIO_MTU] {
        let mut out = [0; RADIO_MTU];
        out[0] = self.header.magic;
        out[1] = self.header.packet_type as u8;
        out[2] = self.header.length;
        // todo mut slice writeall
        for i in 0 .. PAYLOAD_SIZE {
            out[i+3] = self.payload[i];
        }
        out
    }

    fn decode(data: [u8; RADIO_MTU]) -> Result<Self, LoraPacketError> {
        let magic = data[0];
        let length = data[2];
        let packet_type = match data[1] {
            1 => Ok(CtrlTypeCode::Data),
            2 => Ok(CtrlTypeCode::RtxReq),
            _ => Err(LoraPacketError::ControlCodeError(data[1])),
        }?;

        let header = CtrlHeader { magic, packet_type, length };
        
        Ok(Self{header, payload: data[3..].try_into().unwrap()})
    }
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
            .open_native()
            .expect("Failed to open port");

        let this = Arc::new(Self {
            rx,
            serial: Mutex::new(serial),
        });

        task::spawn(Self::read_serial(this.clone(), tx));

        this
    }

    async fn read_serial(self: Arc<Self>, c: channel::Sender<Frame>) {
        let buffer: [u8; RADIO_MTU] = [0; RADIO_MTU];
        loop {
            let mut rx_packet = LoraPacket::decode(buffer).unwrap();

            // check header format is correct.
            if rx_packet.header.magic != IRDEST_MAGIC {
                continue;
            }
            

            self.serial.lock().await.read_exact(&mut rx_packet.payload).unwrap();
            let frame = decode_frame(&mut buffer.as_slice()).unwrap();
            c.send(frame).await.unwrap();
        }
    }
}

#[async_trait]
impl Endpoint for LoraEndpoint {
    fn size_hint(&self) -> usize {
        PAYLOAD_SIZE
    }

    async fn send(&self, frame: Frame, _target: Target, exclude: Option<u16>) -> NMResult<()> {
        if exclude.is_some() {
            return Ok(());
        }

        let header = CtrlHeader {
            magic: IRDEST_MAGIC,
            packet_type: CtrlTypeCode::Data,
            length: self.size_hint() as u8,
        };

        let mut payload = vec![];
        encode_frame(&mut payload, &frame).unwrap();

        assert_eq!(payload.len(), PAYLOAD_SIZE);

        let payload = payload.as_slice().try_into().unwrap();

        let tx_packet = LoraPacket { header, payload };

        let buffer = tx_packet.encode();

        self.serial.lock().await.write(&buffer).unwrap();

        Ok(())
    }

    async fn next(&self) -> NMResult<(Frame, Target)> {
        let frame = self.rx.recv().await.unwrap();
        Ok((frame, Target::Single(0)))
    }
}
