use std::collections::BTreeMap;

use crate::types::Id;

/// A simple authentication object
pub struct ClientAuth {
    pub client_id: Id,
    pub token: Id,
}

/// Apply a tri-state modification to an existing Option<T>
pub enum Modify<T> {
    Keep,
    Change(T),
    DeleteOne(T),
    DeleteAll,
}

/// Apply a Modify object to an Option
pub fn apply_simple_modify<T>(base: &mut Option<T>, mobj: Modify<T>) {
    match mobj {
        // Don't apply a change
        Modify::Keep => {}
        // We would love to do something
        // here but we don't know how to since T isn't a Map
        Modify::DeleteOne(_) => {
            warn!(
                "Function `apply_simple_modify` called with an invalid operand: \
                   Modify::DeleteOne(_) is not implemented by this function."
            );
        }
        // Delete value
        Modify::DeleteAll => *base = None,
        // Add value
        Modify::Change(new) => *base = Some(new),
    }
}

pub fn apply_map_modify<T: Ord>(base: &mut BTreeMap<T, T>, mobj: Modify<T>) {
    match mobj {
        Modify::DeleteOne(ref key) => {
            base.remove(key);
        }
        _ => {
            warn!("Function `apply_map_modify` called with a non-Map base operand");
        }
    };
}
