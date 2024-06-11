use crate::types::Ident32;

/// Represent an Irdest subnet
///
/// A subnet is any group of devices that identifies as being in a
/// given subnet.  Currently this only represents type level
/// information -- no protocols use this feature yet.
pub struct Subnet {
    pub subnet_id: Ident32,
}
