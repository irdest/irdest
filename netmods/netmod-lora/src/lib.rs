// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

#[macro_use]
extern crate tracing;

use irdest_firmware_util::{decode_frame, encode_frame};
use libratman::{
    netmod::{Endpoint, InMemoryEnvelope, Target},
    Result as RatmanResult,
};

use async_std::{channel, sync::Arc, sync::Mutex, task};
use async_trait::async_trait;

use serialport::TTYPort;
use std::io::prelude::*;
use std::time::Duration;

const BUFFER_SIZE: usize = 32; // sets the depth of the netmod's recieve buffer.

const IRDEST_MAGIC: u8 = 0xCA; // sets the unique protocol identifier for irdest traffic, changing will split the network.
const RADIO_MTU: usize = 255; // sets the size of data block expected by the modem. This is correct for sx127x based modems.

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum CtrlTypeCode {
    _Invalid = 0x00,
    Data,
    RtxReq,
}

#[repr(packed)]
#[derive(Copy, Clone, Debug)]
struct CtrlHeader {
    magic: u8,
    packet_type: CtrlTypeCode,
    length: u8,
}

const PAYLOAD_SIZE: usize = RADIO_MTU - std::mem::size_of::<CtrlHeader>();

#[derive(Debug)]
enum LoraPacketError {
    InvalidMagicNumber(u8),
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
        for i in 0..PAYLOAD_SIZE {
            out[i + 3] = self.payload[i];
        }
        out
    }

    fn decode(data: [u8; RADIO_MTU]) -> Result<Self, LoraPacketError> {
        if data[0] != IRDEST_MAGIC {
            return Err(LoraPacketError::InvalidMagicNumber(data[0]));
        }
        let magic = data[0];
        let length = data[2];
        let packet_type = match data[1] {
            1 => Ok(CtrlTypeCode::Data),
            2 => Ok(CtrlTypeCode::RtxReq),
            _ => Err(LoraPacketError::ControlCodeError(data[1])),
        }?;

        let header = CtrlHeader {
            magic,
            packet_type,
            length,
        };

        Ok(Self {
            header,
            payload: data[3..].try_into().unwrap(),
        })
    }
}

pub struct LoraEndpoint {
    rx: channel::Receiver<InMemoryEnvelope>,
    serial: Mutex<TTYPort>,
}

impl LoraEndpoint {
    pub fn spawn(port: &str, baud: u32) -> Arc<Self> {
        let (tx, rx) = channel::bounded(BUFFER_SIZE);
        let serial = serialport::new(port, baud)
            .timeout(Duration::from_millis(10))
            .open_native()
            .expect("Failed to open port");

        let this = Arc::new(Self {
            rx,
            serial: Mutex::new(serial),
        });

        task::spawn(Self::read_serial(this.clone(), tx));

        info!("Created Successfully!");
        this
    }

    async fn read_serial(self: Arc<Self>, c: channel::Sender<InMemoryEnvelope>) {
        debug!("Starting serial Read loop");
        let mut buffer: [u8; RADIO_MTU] = [0; RADIO_MTU];
        loop {
            match self.serial.lock().await.read_exact(&mut buffer) {
                Ok(()) => (),
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
                Err(e) => panic!("{:?}", e),
            }

            // trace!("rx <= {:?}", buffer);

            let rx_packet = match LoraPacket::decode(buffer) {
                Ok(p) => p,
                Err(e) => {
                    info!("rx bad packet {:?}", e);
                    continue;
                }
            };

            trace!("recieved packet");

            let frame = match decode_frame(&mut rx_packet.payload.as_slice()) {
                Ok(f) => f,
                Err(e) => {
                    error!("failed to decode recieved packet: {}", e);
                    continue;
                }
            };

            c.send(frame).await.unwrap();
        }
    }
}

#[async_trait]
impl Endpoint for LoraEndpoint {
    fn size_hint(&self) -> usize {
        PAYLOAD_SIZE
    }

    async fn send(
        &self,
        frame: InMemoryEnvelope,
        _target: Target,
        exclude: Option<u16>,
    ) -> RatmanResult<()> {
        if exclude.is_some() {
            warn!("Cannot send messages containing exlude fields");
            return Ok(());
        }

        let header = CtrlHeader {
            magic: IRDEST_MAGIC,
            packet_type: CtrlTypeCode::Data,
            length: self.size_hint() as u8,
        };

        let mut payload = vec![];
        encode_frame(&mut payload, &frame).unwrap();

        loop {
            if payload.len() == PAYLOAD_SIZE {
                break;
            }
            payload.push(0);
        }

        assert_eq!(payload.len(), PAYLOAD_SIZE);

        let payload = payload.as_slice().try_into().unwrap();

        let tx_packet = LoraPacket { header, payload };

        let buffer = tx_packet.encode();

        // trace!("tx => {:?}", buffer);

        match self.serial.lock().await.write_all(&buffer) {
            Ok(()) => trace!("Sent Packet"),
            Err(e) => error!("Serial Write error: {}", e),
        }

        Ok(())
    }

    async fn next(&self) -> RatmanResult<(InMemoryEnvelope, Target)> {
        let frame = self.rx.recv().await.unwrap();
        trace!("delivering frame to deamon");
        Ok((frame, Target::Single(0)))
    }
}
