//! KDL Configuration handling module
//!
//! This module handles the Ratman (and ecosystem) configuration
//! system.  KDL itself is a very nice language for expressing complex
//! nested structures, without having the complexity get in the way of
//! editing larger configurations.  The KDL crate also supports
//! in-line edits, preserving comments, and more.  Unfortunately this
//! comes at a cost of API usability.  Some of the ways the KDL
//! structures are exposed are very not-obvious, and so this module
//! aims to build around the KDL crate to create an easy to use, *easy
//! to log* (!) API surface for interacting with the various bits of
//! Ratman configuration.
//!
//! Here are some common errors that must be expressed to the user:
//!
//! - Grabbing a KDL sub-configuration (for example `settings
//! ratmand`), which doesn't contain a child block.
//! -

use kdl::{KdlDocument, KdlNode, KdlValue};
use libratman::{
    tokio::{
        fs::{File, OpenOptions},
        io::{AsyncReadExt, AsyncWriteExt},
    },
    Result,
};
use std::path::PathBuf;

mod default;
pub mod helpers;
pub mod netmods;
pub mod peers;

/// Represent the well-known `ratmand` configuration tree
pub(crate) const CFG_RATMAND: &'static str = "ratmand";

//// Represent the well-known `irdest.intrinsics` configuration tree
//pub(crate) const CFG_INTRINSICS: &'static str = "irdest.intrinsics";

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

impl ConfigTree {
    /// Create a new default configuration
    ///
    /// In ratmand, this function should only be called once, when
    /// running for the first time.  During tests, it can be useful to
    /// start with the default configuration, and then use `patch` to
    /// augment it in various ways.
    ///
    /// Alternatively this function may run when the user invokes
    /// `ratmand config generate`, initially populating a
    /// configuration that they can then customise.
    pub fn default_in_memory() -> Self {
        Self {
            inner: default::create_new_default(),
        }
    }

    /// Quickly override any parts of the default config for tests
    ///
    /// A key path is segments of the configuration, split by `/`, so
    /// for example `ratmand/verbosity`, or `inet/enable`.
    pub fn patch(mut self, key_path: &str, value: impl Into<KdlValue>) -> Self {
        let (tree, setting) = key_path.split_once('/').expect("invalid key_path syntax");

        let subtree = helpers::select_mut_settings_tree(&mut self.inner, tree)
            .expect(&format!("invalid subtree {}", tree));

        let node = subtree
            .children_mut()
            .as_mut()
            .unwrap()
            .get_mut(setting)
            .expect(&format!("setting {} doesn't exist", setting));
        node.clear_entries();
        node.push(value);

        self
    }

    /// Use this function to patch a list block (for example `ratmand/peers`)
    pub fn patch_list(mut self, key_path: &str, value: impl Into<KdlValue>) -> Self {
        let (tree, setting) = key_path.split_once('/').expect("invalid key_path syntax");
        let subtree = helpers::select_mut_settings_tree(&mut self.inner, tree)
            .expect(&format!("invalid subtree {}", tree));

        // println!("First tree: {}", subtree.entries().first().unwrap());
        helpers::append_to_list_block(
            subtree.children_mut().as_mut().expect("invalid subtree"),
            setting,
            value,
        );

        self
    }

    pub async fn load_path(path: impl Into<PathBuf>) -> Result<Self> {
        let mut f = File::open(path.into()).await?;
        let mut buf = String::new();
        f.read_to_string(&mut buf).await?;

        Ok(Self {
            inner: buf.parse().unwrap(),
        })
    }

    /// Take the current in-memory configuration and write it to disk
    ///
    /// This will mostly occur when running ratmand for the first
    /// time, but may also be done after applying changes to the
    /// router configuration from the web management dashboard.
    pub async fn write_changes(&self, path: impl Into<PathBuf>) -> Result<()> {
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .open(path.into())
            .await?;
        f.write_all(self.inner.to_string().as_bytes()).await?;
        Ok(())
    }

    // TODO: implement configuration migrations

    /// Get the subtree from the Ratman config with a particular name
    pub fn get_subtree(&self, id: &str) -> Option<SubConfig<'_>> {
        helpers::select_settings_tree(&self.inner, id).map(|inner| SubConfig { inner })
    }
}

#[derive(Debug)]
pub struct SubConfig<'p> {
    pub inner: &'p KdlNode,
}

impl<'p> SubConfig<'p> {
    /// A utility function that takes a SubConfig and dereferences the
    /// inner KdlDocument.  This is needed because we might have a
    /// subtree with a missing inner document
    fn deref_inner(&self) -> Option<&'p KdlDocument> {
        self.inner.children()
    }

    pub fn get_value(&self, key: &str) -> Option<&'p KdlValue> {
        self.deref_inner()?
            .get(key)
            .and_then(|node| node.entries().first())
            .map(|entry| entry.value())
    }

    /// Utility for get_value which also handles String encoding
    pub fn get_string_value(&self, key: &str) -> Option<String> {
        self.get_value(key)
            .and_then(|value| value.as_string())
            .map(Into::into)
    }

    /// Utility for get_value which also handles bool conversion
    pub fn get_bool_value(&self, key: &str) -> Option<bool> {
        self.get_value(key).and_then(|value| value.as_bool())
    }

    /// Utility for get_value which also handles number conversion
    pub fn get_number_value(&self, key: &str) -> Option<i64> {
        self.get_value(key).and_then(|value| value.as_i64())
    }

    /// Got a subtree and interpret it as a list-block
    pub fn get_list_block(&self, id: &str) -> Option<Vec<&'p KdlValue>> {
        Some(self.deref_inner()?.get_dash_vals(id))
    }

    /// Utility for reading a list block of strings
    pub fn get_string_list_block(&self, id: &str) -> Option<Vec<String>> {
        self.get_list_block(id)
            .and_then(|vec| {
                vec.into_iter()
                    .map(|value| value.as_string())
                    .collect::<Option<Vec<&str>>>()
            })
            .map(|vec| vec.into_iter().map(Into::into).collect())
    }

    /// Get a subtree and interpret it as a full subtree
    pub fn get_subtree(&self, id: &str) -> Option<SubConfig<'p>> {
        self.deref_inner()?
            .nodes()
            .into_iter()
            // Find a node with the given name
            .find_map(|node| {
                if node.name().repr() == Some(id) {
                    Some(node)
                } else {
                    None
                }
            })
            // Create a new API wrapper type
            .map(|inner| SubConfig { inner })
    }
}

/// Pretty-print this configuration without all of the span crap
#[doc(hidden)]
pub fn pretty_print(doc: &KdlDocument) {
    doc.nodes().iter().for_each(|node| {
        let node_header = format!(
            "{} : {:?}",
            node.name(),
            node.entries()
                .iter()
                .map(|e| e.value().as_string().unwrap_or("<?>"))
                .collect::<Vec<_>>()
        );

        let node_children = node
            .children()
            .iter()
            .map(|child| {
                child
                    .nodes()
                    .iter()
                    .map(|node| {
                        format!(
                            "{} : {:?}",
                            node.name(),
                            node.entries()
                                .iter()
                                .map(|e| format!("{}", e.value()).replace("\"", ""))
                                .collect::<Vec<_>>()
                        )
                        .replace("\"", "")
                    })
                    .collect::<Vec<String>>()
            })
            .collect::<Vec<_>>();

        println!("{}", node_header);
        println!("{:#?}", node_children);
    })
}

#[test]
fn config_patch() {
    let cfg = ConfigTree::default_in_memory().patch_list("ratmand/peers", "inet:localhost:99999");

    assert_eq!(
        cfg.get_subtree("ratmand")
            .unwrap()
            .get_string_list_block("peers")
            .unwrap(),
        vec!["inet:localhost:99999".to_owned()]
    )
}
