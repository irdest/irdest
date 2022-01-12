use netmod_tcp::{Endpoint as TcpEp, Result as TcpResult};

/// Parse a peer and introduce it to the appropriate netmod metadata
pub async fn attach_peers(ep: &TcpEp, p: Vec<&str>) -> TcpResult<()> {
    let mut tcp = vec![];
    for peer in p {
        let split: Vec<_> = peer.split('#').collect();
        let nmtt = match split.get(0) {
            Some(tt) => tt,
            None => continue,
        };

        match nmtt {
            &"tcp" => tcp.push(match split.get(1).map(Clone::clone) {
                Some(tt) => tt.to_string(),
                None => continue,
            }),
            tt => {
                warn!("Unknown peer type: {}", tt);
                continue;
            }
        }
    }

    ep.add_peers(tcp).await
}
