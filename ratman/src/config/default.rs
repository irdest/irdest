use kdl::KdlDocument;

const DEFAULT_CONFIG: &'static str = include_str!("./ratmand-0.5.kdl");

/// Create a new default configuration from scratch
pub fn create_new_default() -> KdlDocument {
    DEFAULT_CONFIG.parse().expect("error in built-in configuration (if you are not a developer of Irdest, please report this problem!): ")
}

/// Take an existing KDL configuration and update it to the latest format
pub fn update_existing(kdl_doc: KdlDocument) -> KdlDocument {
    kdl_doc
}
