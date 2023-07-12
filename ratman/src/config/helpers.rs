//! A set of helper functions for dealing with configuration data

use kdl::{KdlDocument, KdlNode};
use libratman::types::Result;
use std::{fs::File, io::Read, path::PathBuf};

/// Take a path (from the configuration) and load a peer file from it
pub fn load_peers_file(path: impl Into<PathBuf>) -> Result<Vec<String>> {
    let mut f = File::open(path.into())?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;

    Ok(buf.lines().into_iter().fold(vec![], |mut vec, line| {
        vec.push(line.to_string());
        // TODO: validate peer entry ?
        vec
    }))
}

/// A utility function to select a Node by its first entry, instead of
/// its name
pub(super) fn select_settings_tree<'d>(doc: &'d KdlDocument, scope: &str) -> Option<&'d KdlNode> {
    doc.nodes().iter().find_map(|node| {
        node.get(0)
            .and_then(|entry| match entry.value_repr().map(|name| name == scope) {
                Some(_) => Some(node),
                _ => None,
            })
    })
}
