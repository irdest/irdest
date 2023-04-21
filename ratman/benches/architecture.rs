//! Architecture benchmark tests to determine the Ratman 0.10 re-design
//!
//! An architectural benchmark must fulfill a set of requirements.
//! "VARIABLE" refers to a parameter, which should be easily tweakable
//! (i.e., controlled via some kind of optimisation mechanism).  As a
//! foundation to this optimiser, the core requirements are:
//!
//! 1) Accept a connection via a socket (VARIABLE), and read a very
//! simple header type.
//!
//! 2) Read the message length one buffer size (VARIABLE) at a time
//!
//! 3) for each chunk of message, execute a static test function
//! (which inverts the contents)
//!
//! 4) Pass the message chunk to a data consumer, which writes it to
//! `/dev/null`
//!
//! All message fragments must arrive in order, meaning that while
//! each buffer is inverted, no order MUST be changed.  Otherwise no
//! linearity to the router is implied, meaning that two connections
//! don't have to execute in any particular order.
//!
//! A bench manager must follow these stepes:
//!
//! 1) Create a consumer thread, which can be passed into the benchmark
//! target to consume messages for it in the futures
//!
//! 2) Start the benchmark target (with a consumer thread) and give it
//! a second to start up.
//!
//! 3) Create a producer thread, which creates a new socket connection
//! to the benchmark target.
//!
//! 4) The producer thread follows an optimiser strategy
//!
//! 5) The benchmark runs according to the benchmark runner
//! (criterion)

pub const TCP_SOCKET: &'static str = "127.0.0.1:7171";

pub mod x01_PureThreads {
    //! A very naive approach of spawning a new thread for every
    //! incoming TCP connection.  This design approach implies that
    //! all state MUST be shared across all threads.
    //!
    //! This benchmark

    use crate::{BenchmarkTarget, StreamConsumer, StreamProducer};
    use std::sync::{Arc, Mutex};

    pub struct Target<P: StreamProducer> {
        state: Arc<Mutex<usize>>,
        producer: Option<P>,
        consumer: Option<StreamConsumer>,
    }

    impl<P: StreamProducer> BenchmarkTarget<P> for Target<P> {
        fn init(&mut self, prod: P, cons: StreamConsumer) {
            self.producer = Some(prod);
            self.consumer = Some(cons);
        }

        fn wait_for_start(&mut self) {
            let mut r = match self
                .producer
                .expect("you must call init(...) first!")
                .produce()
            {
                Some(x) => x,
                None => return,
            };

            std::thread::spawn(|| {});

            todo!()
        }
    }
}

fn main() {}

///////////////////////////////
////   T h e    G U T S   ////
/////////////////////////////

use std::io::Read;

pub trait StreamProducer {
    fn produce<R: Read>(&mut self) -> Option<R>;
}

pub struct StreamConsumer {}

impl StreamConsumer {
    pub fn consume<R: Read>(&self, chunk: &R) {}
}

pub trait BenchmarkTarget<P>
where
    P: StreamProducer,
{
    /// Called once to pass a stream producer and stream consumer
    fn init(&mut self, prod: P, cons: StreamConsumer);

    /// Start the main loop machinery, which will spawn new work units
    /// as needed by its policy.
    fn wait_for_start(&mut self);
}

// use criterion::{BenchmarkId, Criterion, Throughput};
// use std::iter;

// fn from_elem(c: &mut Criterion) {
//     static KB: usize = 1024;

//     let mut group = c.benchmark_group("from_elem");
//     for size in [KB, 2 * KB, 4 * KB, 8 * KB, 16 * KB].iter() {
//         group.throughput(Throughput::Bytes(*size as u64));
//         group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
//             b.iter(|| {
//                 iter::repeat(0u8)
//                     .map(|t| t % 111)
//                     .take(size)
//                     .collect::<Vec<_>>()
//             });
//         });
//     }
//     group.finish();
// }

// criterion::criterion_group!(benches, from_elem);
// criterion::criterion_main!(benches);
