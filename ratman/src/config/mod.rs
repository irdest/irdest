use kdl::KdlNode;
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
    inner: BTreeMap<String, SubConfig>,
}

impl ConfigTree {
    pub fn ratmand_config(&self) -> &SubConfig {
        self.inner
            .get(RATMAND_CFG)
            .expect(&format!("Configuration key `{}` missing", RATMAND_CFG))
    }

    pub fn irdest_intrinsics_config(&self) -> &SubConfig {
        self.inner
            .get(INTRINSICS_CFG)
            .expect(&format!("Configuration key `{}` missing", INTRINSICS_CFG))
    }
}

pub struct SubConfig {
    inner: KdlNode,
}
