use crate::types::Address;
use std::ffi::CString;

pub struct ContactAdd {
    /// The address to add as a contact
    pub addr: Address,
    /// A user defined (private!) note
    pub note: Option<CString>,
    /// Key=Value tags, strings are \0 terminated
    pub tags: Vec<(CString, CString)>,
    /// Trust level between 1 and 7
    pub trust: Option<u8>,
}

pub struct ContactDelete {
    /// An address to delete the contact book entry for
    pub addr: Option<Address>,
    /// Key=Value tags, strings are \0 terminated
    pub tags: Vec<(CString, CString)>,
    /// Trust level between 1 and 7
    pub trust: Option<u8>,
}

pub struct ContactModify {
    pub add_tags: Vec<(CString, CString)>,
    pub remove_tags_by_key: Vec<CString>,
    pub addr: Option<Address>,
    pub new_trust: Option<u8>,
}

pub enum ContactCommand {
    Add(ContactAdd),
    Delete(ContactDelete),
    Modify(ContactModify),
}
