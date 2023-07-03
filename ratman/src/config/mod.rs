use kdl::{KdlDocument, KdlNode, KdlValue};
use std::collections::BTreeMap;

pub mod default;

const RATMAND_CFG: &'static str = "ratmand";
const INTRINSICS_CFG: &'static str = "irdest.intrinsics";

/// Represents the top-level configuration structure
///
/// Certain keys are _guaranteed_ to exist in the inner tree (mainly
/// `ratmand` and `irdest.intrinsics`).  Other keys MUST be handled
/// optionally, depending on the desired runtime configuration of the
/// router instance.
pub struct ConfigTree {
    inner: KdlDocument,
}

impl ConfigTree {
    pub fn ratmand_config(&self) -> &SubConfig {
        todo!()
    }

    /// Get a particular configuration tree
    pub fn get_config<'p>(&'p self, section: &str) -> SubConfig<'p> {
        todo!()
    }
}

pub struct SubConfig<'p> {
    inner: &'p KdlNode,
}

impl<'p> SubConfig<'p> {
    pub fn get_value(&self, key: &str) -> &'p KdlValue {
        todo!()
    }
}
