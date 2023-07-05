// SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

mod client;
mod connection;
mod io;
// mod parse;
// mod state;

pub(crate) use connection::ConnectionManager;

// /// Client API manager
// pub struct ClientApiManager {
//     connections: ConnectionManager,
// }

// impl ClientApiManager {
//     ///
//     pub async fn start(bind: SocketAddr) -> Result<Self> {
//         info!("Listening for API connections on socket {:?}", bind);
//         let listener = TcpListener::bind(bind).await?;

//         todo!()
//     }
// }
