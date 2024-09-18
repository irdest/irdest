use libratman::{
    endpoint::NeighbourMetrics,
    tokio::{sync::RwLock, task::spawn, time::sleep},
    NonfatalError, RatmanError, Result,
};
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};

/// The metrics table keeps track of connection metrics for a given
pub struct MetricsTable<A>
where
    A: PartialOrd + Ord + Copy + Send + Sync + 'static,
{
    /// (Last time numbers were consolidated, Last period, Current accumulator)
    pub inner: RwLock<BTreeMap<A, (Instant, NeighbourMetrics, NeighbourMetrics)>>,
}

impl<A> Default for MetricsTable<A>
where
    A: PartialOrd + Ord + Copy + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<A> MetricsTable<A>
where
    A: PartialOrd + Ord + Copy + Send + Sync + 'static,
{
    fn new() -> Self {
        Self {
            inner: RwLock::new(BTreeMap::new()),
        }
    }

    pub async fn get_last_period(self: &Arc<Self>, peer: A) -> Result<NeighbourMetrics> {
        self.inner
            .read()
            .await
            .get(&peer)
            .map(|(_, last_period, _)| *last_period)
            .ok_or(RatmanError::Nonfatal(NonfatalError::NoMetrics))
    }

    pub async fn append_write(self: &Arc<Self>, peer: A, bytes: usize) {
        let this = Arc::clone(&self);
        let mut map = self.inner.write().await;

        map.entry(peer)
            .or_insert_with(move || {
                spawn(async move {
                    sleep(Duration::from_secs(10)).await;
                    let mut map = this.inner.write().await;
                    let (ref mut last_time, mut last_period, mut curr_acc) =
                        map.get_mut(&peer).unwrap();
                    *last_time = Instant::now();
                    last_period.write_bandwidth = curr_acc.write_bandwidth;
                    curr_acc.write_bandwidth = 0;
                });
                (Instant::now(), Default::default(), Default::default())
            })
            .2
            .write_bandwidth += bytes as u64;
    }

    pub async fn append_read(self: &Arc<Self>, peer: A, bytes: usize) {
        let this = Arc::clone(&self);
        let mut map = self.inner.write().await;

        map.entry(peer)
            .or_insert_with(move || {
                spawn(async move {
                    sleep(Duration::from_secs(10)).await;
                    let mut map = this.inner.write().await;
                    let (ref mut last_time, mut last_period, mut curr_acc) =
                        map.get_mut(&peer).unwrap();
                    *last_time = Instant::now();
                    last_period.read_bandwidth = curr_acc.read_bandwidth;
                    curr_acc.read_bandwidth = 0;
                });
                (Instant::now(), Default::default(), Default::default())
            })
            .2
            .read_bandwidth += bytes as u64;
    }
}
