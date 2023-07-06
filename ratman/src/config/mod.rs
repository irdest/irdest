use async_std::{fs::OpenOptions, io::WriteExt};
use kdl::{KdlDocument, KdlNode, KdlValue};
use libratman::types::Result;
use std::path::PathBuf;

mod default;

/// Represent the well-known `ratmand` configuration tree
pub(crate) const CFG_RATMAND: &'static str = "ratmand";

/// Represent the well-known `irdest.intrinsics` configuration tree
pub(crate) const CFG_INTRINSICS: &'static str = "irdest.intrinsics";

/// Represents the top-level configuration structure
///
/// Certain keys are _guaranteed_ to exist in the inner tree (mainly
/// `ratmand` and `irdest.intrinsics`).  Other keys MUST be handled
/// optionally, depending on the desired runtime configuration of the
/// router instance.
pub struct ConfigTree {
    #[doc(hidden)]
    pub inner: KdlDocument,
}

/// A utility function to select a Node by its first entry, instead of
/// its name
fn select_settings_tree<'d>(doc: &'d KdlDocument, scope: &str) -> Option<&'d KdlNode> {
    doc.nodes().iter().find_map(|node| {
        node.get(0)
            .and_then(|entry| match entry.value_repr().map(|name| name == scope) {
                Some(_) => Some(node),
                _ => None,
            })
    })
}

impl ConfigTree {
    /// Create a new default configuration
    ///
    /// This function should only be called when running for the
    /// "first time" (aka when no existing configuration could be
    /// detected).
    ///
    /// Alternatively this function may run when the user invokes
    /// `ratmand config generate`, initially populating a
    /// configuration that they can then customise.
    pub fn create_new_default() -> Self {
        Self {
            inner: default::create_new_default(),
        }
    }

    /// Take the current in-memory configuration and write it to disk
    ///
    /// This will mostly occur when running ratmand for the first
    /// time, but may also be done after applying changes to the
    /// router configuration from the web management dashboard.
    pub async fn write_changes(&self, path: PathBuf) -> Result<()> {
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)
            .await?;
        f.write_all(self.inner.to_string().as_bytes()).await?;
        Ok(())
    }

    // TODO: implement configuration migrations

    /// Get the subtree from the Ratman config with a particular name
    pub fn get_subtree(&self, id: &str) -> Option<SubConfig<'_>> {
        select_settings_tree(&self.inner, id).map(|inner| SubConfig { inner })
    }
}

pub struct SubConfig<'p> {
    inner: &'p KdlNode,
}

impl<'p> SubConfig<'p> {
    pub fn get_value(&self, key: &str) -> Option<&'p KdlValue> {
        self.inner.get(key).map(|entry| entry.value())
    }

    pub fn get_subtree(&self, id: &str) -> Option<SubConfig<'p>> {
        // self.inner.children().iter().find_map(|node| {
        //     node.get(0)
        //         .and_then(|entry| match entry.value_repr().map(|name| name == scope) {
        //             Some(_) => Some(node),
        //             _ => None,
        //         })
        // })
        println!("{:#?}", self.inner.children());
        
        todo!()
    }
}
