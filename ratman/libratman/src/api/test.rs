// /// This test is horrible and a bad idea but whatever
// /// also you need to kill the daemon(kill process) after the test
// #[async_std::test]
// #[ignore]
// async fn send_message() {
//     pub fn setup_logging() {
//         use tracing_subscriber::{filter::LevelFilter, fmt, EnvFilter};
//         let filter = EnvFilter::default()
//             .add_directive(LevelFilter::TRACE.into())
//             .add_directive("async_std=error".parse().unwrap())
//             .add_directive("async_io=error".parse().unwrap())
//             .add_directive("polling=error".parse().unwrap())
//             .add_directive("mio=error".parse().unwrap());

//         // Initialise the logger
//         fmt().with_env_filter(filter).init();
//     }

//     setup_logging();

//     use async_std::task::sleep;
//     use std::{process::Command, time::Duration};

//     let mut daemon = Command::new("cargo")
//         .current_dir("../..")
//         .args(&[
//             "run",
//             "--bin",
//             "ratmand",
//             "--features",
//             "daemon",
//             "--",
//             "--no-inet",
//             "--accept-unknown-peers",
//         ])
//         .spawn()
//         .unwrap();

//     sleep(Duration::from_secs(1)).await;

//     let client = RatmanIpc::default().await.unwrap();
//     let msg = vec![1, 3, 1, 2];
//     info!("Sending message: {:?}", msg);
//     client.send_to(client.address(), msg).await.unwrap();

//     let (_, recv) = client.next().await.unwrap();
//     info!("Receiving message: {:?}", recv);
//     assert_eq!(recv.get_payload(), &[1, 3, 1, 2]);

//     // Exorcise the deamons!
//     daemon.kill().unwrap();
// }

use crate::frame::{
    micro::{client_modes, MicroframeHeader},
    FrameGenerator,
};

use super::{Handshake, RawSocketHandle};
use std::time::Duration;
use tokio::{
    net::{TcpListener, TcpStream},
    spawn, time,
};

#[tokio::test]
async fn send_recv_handshake() {
    spawn(async {
        let l = TcpListener::bind("0.0.0.0:9876").await.unwrap();
        let (stream, _) = l.accept().await.unwrap();
        let mut raw = RawSocketHandle::new(stream);

        let (header, handshake) = raw.read_microframe::<Handshake>().await.unwrap();

        assert_eq!(
            header,
            MicroframeHeader {
                modes: client_modes::make(client_modes::INTRINSIC, client_modes::UP),
                auth: None,
                payload_size: {
                    let mut buf = vec![];
                    Handshake::new().generate(&mut buf).unwrap();
                    buf.len() as u32
                }
            }
        );

        assert_eq!(handshake, Handshake::new());
    });

    spawn(async {
        time::sleep(Duration::from_secs(2)).await;
        let mut raw = RawSocketHandle::new(TcpStream::connect("127.0.0.1:9876").await.unwrap());
        raw.write_microframe(MicroframeHeader::intrinsic_noauth(), Handshake::new())
            .await
            .unwrap();
    });
}
