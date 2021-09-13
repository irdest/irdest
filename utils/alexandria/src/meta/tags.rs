use crate::utils::{Path, Tag, TagSet};

use std::collections::{BTreeMap, BTreeSet};

/// Per-user encrypted tag storage
#[derive(Debug, Clone, Default)]
pub(crate) struct UserTags {
    /// Mapping tags to paths
    t2p: BTreeMap<Tag, BTreeSet<Path>>,
    /// Mapping paths to their set of tags
    p2t: BTreeMap<Path, TagSet>,
}

impl UserTags {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    /// Insert a new tag-path relationship into the store
    ///
    /// If the tag already exists, the path will be appended.  If it
    /// doesn't a new dataset will be created.
    pub(crate) fn insert(&mut self, tag: Tag, path: &Path) {
        self.t2p
            .entry(tag.clone())
            .or_default()
            .insert(path.clone());
        self.p2t.entry(path.clone()).or_default().insert(tag);
    }

    /// Remove a path from all tag models
    pub(crate) fn clear(&mut self, path: &Path) {
        self.p2t.remove(path);
        self.t2p.iter_mut().for_each(|(_, set)| {
            set.remove(path);
        });
    }

    /// Paths with an associated tagset that passed a check
    pub(crate) fn paths<F>(&self, cond: F) -> Vec<Path>
    where
        F: Fn(&TagSet) -> bool,
    {
        self.p2t
            .iter()
            .fold(BTreeSet::new(), |mut set, (path, tagset)| {
                if cond(tagset) {
                    set.insert(path);
                }

                set
            })
            .into_iter()
            .cloned()
            .collect()
    }
}
