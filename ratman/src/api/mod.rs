// SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

mod client;
mod connection;
mod io;
// mod parse;
// mod state;

use crate::context::RatmanContext;
pub(crate) use connection::ConnectionManager;
use libratman::types::{api, Message, Recipient};

// async fn run_relay(context: RatmanContext) {
//     loop {
//         let Message {
//             id,
//             sender,
//             recipient,
//             payload,
//             time,
//             signature,
//         } = context.core.next().await;
//         debug!("Receiving message for {:?}", recipient);
//         let recv = api::receive_default(Message::received(
//             id,
//             sender,
//             recipient.clone(),
//             payload,
//             format!("{:?}", time),
//             signature,
//         ));

//         match recipient {
//             ref recp @ Recipient::Standard(_) => {
//                 if let Some(Some(ref mut io)) = todo!()
//                     .lock()
//                     .await
//                     .get(&recp.scope().expect("empty recipient"))
//                     .map(Clone::clone)
//                 {
//                     info!("Forwarding message to online client!");
//                     if let Err(e) = parse::forward_recv(io.as_io(), recv).await {
//                         error!("Failed to forward received message: {}", e);
//                     }
//                 }
//             }
//             Recipient::Flood(_) => {
//                 for (_, ref mut io) in online.lock().await.iter_mut() {
//                     if io.is_none() && continue {}
//                     if let Err(e) =
//                         parse::forward_recv(io.as_mut().unwrap().as_io(), recv.clone()).await
//                     {
//                         error!("Failed to forward received message: {}", e);
//                     }
//                 }
//             }
//         }
//     }
// }

// /// Run the daemon!
// pub async fn run(r: Router, addr: SocketAddr) -> Result<()> {
//     info!("Listening for API connections on socket {:?}", addr);
//     let listener = TcpListener::bind(addr).await?;
//     let mut state = DaemonState::new(&listener, r.clone());
//     let online = state.get_online().await;

//     let relay = spawn(run_relay(r.clone(), online.clone()));

//     while let Ok(io) = state.listen_for_connections().await {
//         let (_self, io) = match io {
//             Some(io) => io,
//             None => continue,
//         };

//         info!("Established new client connection");
//         spawn(parse::parse_stream(
//             r.clone(),
//             online.clone(),
//             _self,
//             io.clone(),
//         ));
//     }

//     relay.cancel().await;
//     Ok(())
// }
