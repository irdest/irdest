
/// Indicate which endpoint target an envelope should be sent to
pub enum EpTarget {
    /// Send message to all reachable endpoints
    Flood,
    /// Send to all targets, except one
    FloodExcept(u16),
    /// Exclude this envelope from all targets
    Drop,
    /// Encodes a specific target ID
    Single(u16),
}
