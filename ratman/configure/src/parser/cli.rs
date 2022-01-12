use crate::{config::Endpoint, EpBuilder, Id};

/// This function parses the PEER SYNTAX used by ratmand
pub fn parse_peer(id: &mut Id, p: &str) -> Option<(Id, Endpoint)> {
    let split: Vec<_> = p.split('#').collect();

    let nmtt = split.get(0)?;
    match nmtt {
        &"tcp" => {
            let data: Vec<_> = split.get(1)?.split(':').collect();
            let addr = data.get(0)?.to_string();
            let port = data.get(1)?.parse().ok()?;
            let dynamic = data
                .get(2)
                .and_then(|d| if d == &"true" { Some(true) } else { None })
                .unwrap_or(false);
            Some(EpBuilder::tcp(addr, port, dynamic).build(id))
        }
        &"udp" => {
            let addr = split.get(1)?.to_string();
            Some(EpBuilder::local_udp(addr).build(id))
        }
        _ => return None,
    }
}
