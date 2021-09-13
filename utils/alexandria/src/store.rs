//! The internal data store

use crate::{
    delta::{DeltaBuilder, DeltaType},
    error::{Error, Result},
    notify::Notify,
    record::Record,
    utils::{Diff, Id, Path, TagSet},
    Session,
};
use async_std::sync::Arc;
use std::collections::BTreeMap;
use tracing::trace;

// /// Main data store (mirrored to /records)
// #[derive(Default)]
// pub(crate) struct Store {
//     /// The shared datastore
//     shared: BTreeMap<Path, Notify<Encrypted<Arc<Record>, SharedKey>>>,
//     /// The per-user datastore
//     usrd: BTreeMap<Id, Notify<BTreeMap<Path, Notify<Encrypted<Arc<Record>, KeyPair>>>>>,
//     /// Per-user GC locks
//     gc_usr: BTreeMap<Id, BTreeMap<Path, GcGuard>>,
//     /// Shared-scope GC lock
//     gc_shared: BTreeMap<Path, GcGuard>,
// }

/// A guard marker for garbage collection
///
/// Specifies if an item should be held from GC, how many holders
/// there are and if the item should be deleted when the hold expires.
#[derive(Default)]
struct GcGuard {
    /// Number of GC holders
    ctr: usize,
    /// Determine if the item should be deleted
    del: bool,
}

// impl Store {
//     pub(crate) fn new() -> Self {
//         Self::default()
//     }

//     /// Get a single record from the store via the path
//     ///
//     /// If providing a user ID, check the user store first, before
//     /// checking the shared store.
//     pub(crate) fn get_path(&self, id: Session, path: &Path) -> Result<Arc<Record>> {
//         trace!("Getting path `{}`", path);
//         id.id()
//             .and_then(|ref id| self.usrd.get(id))
//             .and_then(|tree| {
//                 tree.get(path)
//                     .and_then(|e| e.deref().map(|ref rec| Arc::clone(&rec)).ok())
//             })
//             .or(self
//                 .shared
//                 .get(path)
//                 .and_then(|e| e.deref().map(|ref rec| Arc::clone(&rec)).ok()))
//             .map_or(Err(Error::NoSuchPath { path: path.into() }), |rec| Ok(rec))
//     }

//     /// Get a single encrypted record to store
//     ///
//     /// The selected entry _won't_ be closed, but encrypted via a
//     /// side-channel.  No decrypted records can ever be written to
//     /// disk.
//     // TODO: this function is awful and should not exist!  FIXME please.
//     //
//     // Why?  I'll tell you whyyy!  Okay, a "Record" in the database
//     // contains a body, and a secret header.  Both the body and secret
//     // header should be encrypted, leaving the rest of the header
//     // untouched.  This means that we can do a quick index of the
//     // database with tags and IDs, without exposing paths, users, etc.
//     //
//     // But, this doesn't actually work currently.  We need to have a
//     // way to get a record into an encryptable state without blocking
//     // other access to the database while this operation is happening.
//     // Furthermore, do we automatically re-open a record or should
//     // future code-paths dynamically unlock the entries?  This
//     // mechanism isn't currently implemented.
//     //
//     // Fundamentally we can't change a Record because it is wrappen in
//     // Arc<Record>.  So do we lock a record in place by swapping its
//     // location in memory with an encrypted variant?
//     pub(crate) fn get_encrypted(
//         &self,
//         key: Arc<KeyPair>,
//         id: Session,
//         path: &Path,
//     ) -> Result<CipherText> {
//         trace!("Synchronising path: {:?}/{}", id, path);
//         let rec: Record = self
//             .tree(id)
//             .get(path)
//             .ok_or(Error::NoSuchPath { path: path.into() })
//             .and_then(|not| not.deref().unwrap().prepare(Arc::clone(&key)))?;

//         key.seal(&rec)
//     }

//     /// Similar to `insert`, but useful to seed an entire record from
//     /// individual diffs at the same time
//     #[tracing::instrument(skip(self, db, diffs, path, tags), level = "trace")]
//     pub(crate) fn batch(
//         &mut self,
//         db: &mut DeltaBuilder,
//         id: Session,
//         path: &Path,
//         tags: TagSet,
//         mut diffs: Vec<Diff>,
//     ) -> Result<Id> {
//         // Check if the path exists already
//         if self.tree_mut(id).contains_key(path) {
//             return Err(Error::PathExists { path: path.into() });
//         }

//         db.tags(&tags);
//         db.path(&path);

//         // Create a record
//         let ulterior = diffs.split_off(1);
//         let initial = diffs.remove(0);

//         let mut rec = Record::create(tags, initial)?;
//         let rec_id = rec.header.id;
//         trace!("Created skeleton record `{}`", rec_id.to_string());

//         for d in ulterior {
//             rec.apply(d)?;
//         }
//         trace!("Applied diffs to skeleton record");

//         let record = Notify::new(Encrypted::new(Arc::new(rec)));
//         db.rec_id(rec_id);

//         self.tree_mut(id).insert(path.clone(), record);
//         self.wake_tree(id, path);

//         Ok(rec_id)
//     }

//     /// Insert a record into the store
//     ///
//     /// This operation will fail if the path already exists
//     #[tracing::instrument(skip(self, db, diff, path, tags), level = "trace")]
//     pub(crate) fn insert(
//         &mut self,
//         db: &mut DeltaBuilder,
//         id: Session,
//         path: &Path,
//         tags: TagSet,
//         diff: Diff,
//     ) -> Result<Id> {
//         // Check if the path exists already
//         if self.tree_mut(id).contains_key(path) {
//             return Err(Error::PathExists { path: path.into() });
//         }

//         db.tags(&tags);
//         db.path(&path);

//         // Create a record
//         let rec = Record::create(tags, diff)?;
//         let rec_id = rec.header.id;
//         trace!("Seeded record `{}` from diff", rec_id);
//         let record = Notify::new(Encrypted::new(Arc::new(rec)));
//         db.rec_id(rec_id);

//         self.tree_mut(id).insert(path.clone(), record);
//         self.wake_tree(id, path);

//         Ok(rec_id)
//     }

//     #[tracing::instrument(skip(self, db, path), level = "trace")]
//     pub(crate) fn destroy(
//         &mut self,
//         db: &mut DeltaBuilder,
//         id: Session,
//         path: &Path,
//     ) -> Result<()> {
//         // Check if the path exists
//         if !self.tree_mut(id).contains_key(path) {
//             return Err(Error::NoSuchPath { path: path.into() });
//         }

//         db.path(&path);

//         // Check if the path GC is locked and mark to delete
//         if let Some(GcGuard { ref mut del, .. }) = self.gc_set_mut(id).get_mut(path) {
//             trace!("Marking path `{}` for future deletion", path);
//             *del = true;
//             return Ok(());
//         }

//         self.wake_tree(id, path);
//         if let Ok(rec) = self.tree_mut(id).remove(path).unwrap().deref() {
//             db.rec_id(rec.header.id);
//             trace!("Deleting record `{}` from store", rec.header.id);
//         }

//         Ok(())
//     }

//     #[tracing::instrument(skip(self, db, path, diff), level = "trace")]
//     pub(crate) fn update(
//         &mut self,
//         db: &mut DeltaBuilder,
//         id: Session,
//         path: &Path,
//         diff: Diff,
//     ) -> Result<()> {
//         // Check that the path actually exists
//         if !self.tree_mut(id).contains_key(path) {
//             return Err(Error::NoSuchPath { path: path.into() });
//         }

//         db.path(&path);

//         // Make a copy of the underlying record
//         let mut not: Notify<_> = self.tree_mut(id).remove(path).unwrap();
//         let arc: &Arc<_> = not.deref()?;
//         let mut rec: Record = (**arc).clone();

//         db.rec_id(rec.header.id);

//         // Apply changes
//         rec.apply(diff)?;

//         // Swap old and new records
//         let mut arc = Arc::new(rec);
//         not.swap(&mut arc);

//         // Re-insert into the tree and wake pollers
//         self.tree_mut(id).insert(path.clone(), not);
//         self.wake_tree(id, path);
//         Ok(())
//     }

//     /// Lock the GC for a set of paths
//     #[tracing::instrument(skip(self), level = "trace")]
//     pub(crate) fn gc_lock(&mut self, paths: &Vec<(Path, Session)>) {
//         paths.iter().for_each(|(path, id)| {
//             self.gc_set_mut(*id).entry(path.clone()).or_default().ctr += 1;
//         });
//     }

//     /// Release the GC guard for a set of paths and delete them
//     #[tracing::instrument(skip(self), level = "trace")]
//     pub(crate) fn gc_release(&mut self, paths: &Vec<(Path, Session)>) -> Result<()> {
//         paths.iter().fold(Ok(()), |res, (path, id)| {
//             if let Some(GcGuard {
//                 ref mut ctr,
//                 ref del,
//             }) = self.gc_set_mut(*id).get_mut(&path)
//             {
//                 // Decrement ctr
//                 *ctr -= 1;

//                 // If we were last, delete
//                 if *ctr == 0 && *del {
//                     let mut db = DeltaBuilder::new(*id, DeltaType::Delete);
//                     res.and_then(|_| self.destroy(&mut db, *id, path))
//                 } else {
//                     res
//                 }
//             } else {
//                 res
//             }
//         })
//     }

//     /// A helper to wake a tree, depending on Id
//     fn wake_tree(&mut self, id: Session, path: &Path) {
//         match id.id() {
//             Some(ref id) => {
//                 let tree = self
//                     .usrd
//                     .get_mut(id)
//                     .expect("Don't try to wake something that doen't exist!");
//                 Notify::notify(tree);

//                 let rec = tree
//                     .get_mut(path)
//                     .expect("Don't try to wake something that doen't exist!");
//                 Notify::notify(rec);
//             }
//             None => {
//                 let tree = self
//                     .shared
//                     .get_mut(path)
//                     .expect("Don't try to wake something that doen't exist!");
//                 Notify::notify(tree);
//             }
//         }
//     }

//     /// A utility function to get the mutable tree, depending on id
//     fn tree_mut(
//         &mut self,
//         id: Session,
//     ) -> &mut BTreeMap<Path, Notify<Encrypted<Arc<Record>, KeyPair>>> {
//         match id.id() {
//             Some(id) => self.usrd.entry(id).or_insert(Notify::new(BTreeMap::new())),
//             None => &mut self.shared,
//         }
//     }

//     /// A utility function to get the mutable tree, depending on id
//     fn tree(&self, id: Session) -> &BTreeMap<Path, Notify<Encrypted<Arc<Record>, KeyPair>>> {
//         match id.id() {
//             Some(ref id) => self
//                 .usrd
//                 .get(id)
//                 .expect(&format!("Failed to get tree for session {}", id)),
//             None => &self.shared,
//         }
//     }

//     /// A utility function to get the mutable gc lock, depending on id
//     fn gc_set_mut(&mut self, id: Session) -> &mut BTreeMap<Path, GcGuard> {
//         match id.id() {
//             Some(id) => self.gc_usr.entry(id).or_default(),
//             None => &mut self.gc_shared,
//         }
//     }

//     #[cfg(test)]
//     #[allow(unused)]
//     fn length(&mut self, id: Session) -> usize {
//         self.tree_mut(id).len()
//     }
// }
