// SPDX-FileCopyrightText: 2024 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::{context::RatmanContext, crypto, procedures::SenderSystem};
use async_eris::BlockSize;
use chrono::Utc;
use libratman::{
    tokio::{
        fs::File,
        io::{copy_buf, BufReader, BufWriter},
        net::TcpStream,
    },
    tokio_util::compat::TokioAsyncReadCompatExt,
    types::{AddrAuth, Address, Ident32, LetterheadV1},
    EncodingError, RatmanError, Result,
};
use std::{env::temp_dir, os::unix::fs::MetadataExt, sync::Arc};

pub async fn exec_send_many_socket(
    ctx: &Arc<RatmanContext>,
    client_id: Ident32,
    stream: TcpStream,
    this_addr: Address,
    auth: AddrAuth,
    letterheads: Vec<LetterheadV1>,
    senders: &Arc<SenderSystem>,
) -> Result<()> {
    let this_key = crypto::get_addr_key(&ctx.meta_db, this_addr, auth)?;
    let buffer_file = temp_dir().join(format!("{this_addr}-{}", Utc::now()));
    let stream_buf = File::create(&buffer_file).await?;
    let mut r = BufReader::new(stream);
    let mut w = BufWriter::new(stream_buf);

    debug!(
        "Accepting incoming {} sender stream...",
        this_addr.pretty_string()
    );

    // First copy the incoming stream to a file, since we need to encode it multiple times
    if let Err(e) = copy_buf(&mut r, &mut w).await {
        error!("failed to read sending client stream!");
        return Err(e.into());
    }

    //// Open the file in read mode now
    drop(w);
    drop(r);

    let buf_f = File::open(&buffer_file).await?;
    let stream_size = buf_f.metadata().await?.size();
    debug!("Start encoding for {stream_size} byte stream");

    let buf_r = BufReader::new(buf_f);
    let mut stream = buf_r.compat();

    for mut lh in letterheads {
        lh.stream_size = stream_size;
        trace!("Generate shared key");
        let shared_key = crypto::diffie_hellman(&this_key, lh.to.inner_address()).ok_or(
            RatmanError::Encoding(EncodingError::Encryption(
                "failed to compute diffie-hellman".into(),
            )),
        )?;

        trace!(
            "Created shared key between {} x {}",
            lh.to.inner_address().pretty_string(),
            lh.from.pretty_string()
        );

        let chosen_block_size = match lh.stream_size {
            m if m < (8 * 1024) => async_eris::BlockSize::_1K,
            _ => async_eris::BlockSize::_32K,
        };
        trace!("{client_id} Start encoding for block size {chosen_block_size}");

        let read_cap = async_eris::encode(
            &mut stream,
            &shared_key.to_bytes(),
            chosen_block_size,
            &ctx.journal.blocks,
        )
        .await?;

        trace!("Block encoding complete");
        trace!("Dispatch block on {chosen_block_size} queue");
        match chosen_block_size {
            BlockSize::_1K => {
                senders.tx_1k.send((read_cap, lh)).await.map_err(|e| {
                    RatmanError::Schedule(libratman::ScheduleError::Contention(e.to_string()))
                })?;
            }
            BlockSize::_32K => {
                senders.tx_32k.send((read_cap, lh)).await.map_err(|e| {
                    RatmanError::Schedule(libratman::ScheduleError::Contention(e.to_string()))
                })?;
            }
        }
    }

    drop(stream);

    Ok(())
}
