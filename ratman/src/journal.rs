use async_eris::{Block, BlockReference};
use pagecache::{Materializer, PageCache};
use serde::{Deserialize, Serialize};

pub struct Journal {}

#[derive(Clone, Debug)]
pub enum BlockEvent<const BS: usize> {
    Add(BlockReference, Block<BS>),
    Remove(BlockReference),
    Invalidate(BlockReference),
    Link(LinkMeta),
}

#[derive(Clone, Debug)]
pub enum LinkMeta {
    Add(BlockReference, BlockReference, Option<String>),
    Destroy(BlockReference, BlockReference),
}

impl<const BS: usize> Materializer for BlockEvent<BS> {
    fn merge(&mut self, other: &Self) {}
}

pub struct Blockjournal<const BS: usize> {
    // Store a map of block id -> block content for easy access
    // Store a set of block events that get compressed when read
    // Provide a search for block completeness of a reference manifest
    /// Keep track of block events
    ev_cache: PageCache<BlockEvent<BS>>,
}
