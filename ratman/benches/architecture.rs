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

// TODO: make this configurable
pub const BUF_SIZE: usize = 256;
pub const TCP_SOCKET: &'static str = "127.0.0.1:7171";

pub mod x01_PureThreads {
    //! A very naive approach of spawning a new thread for every
    //! incoming TCP connection.  This design approach implies that
    //! all state MUST be shared across all threads.
    //!
    //! This benchmark

    use crate::{BenchmarkTarget, StreamConsumer, StreamProducer, BUF_SIZE};
    use std::{
        io::Read,
        sync::{Arc, Mutex},
    };

    pub struct Target<P, R>
    where
        P: StreamProducer<R>,
        R: Read,
    {
        state: Arc<Mutex<usize>>,
        producer: Option<P>,
        consumer: Option<StreamConsumer>,
        _reader: std::marker::PhantomData<R>,
    }

    impl<P, R> BenchmarkTarget<P, R> for Target<P, R>
    where
        P: StreamProducer<R> + Send,
        R: Read + Send + 'static,
    {
        // Store the given producer and consumer handles
        fn init(&mut self, prod: P, cons: StreamConsumer) {
            self.producer = Some(prod);
            self.consumer = Some(cons);
        }

        fn wait_for_start(&mut self) -> Option<()> {
            // Produce a single chunk event
            let temp_r = match self
                .producer
                .as_mut()
                .expect("you must call init(...) first!")
                .produce()
            {
                // Either handle it
                Some(x) => x,
                // Or signal to stop running -- producer is gone
                None => return None,
            };

            // Prepare state to move to new thread
            let state = self.state.clone();
            let _cons = self
                .consumer
                .as_ref()
                .expect("you must call init(...) first!")
                .clone();

            // Start a connection worker
            std::thread::spawn(move || {
                // Read from reader
                let r = temp_r;
                let mut reader = r.lock().unwrap();
                let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
                reader.read_exact(&mut buf).expect("failed read");

                // Transform data
                buf.reverse();

                // Track some internal state
                *state.lock().unwrap() += BUF_SIZE;

                // Push transformed data to consumer
                let _vec = buf.to_vec();
                //let buf_read = BufReader::new(vec.as_slice());
                // cons.consume(buf_read);
            });

            // Signal Ok
            Some(())
        }
    }
}

///////////////////////////////
////   T h e    G U T S   ////
/////////////////////////////

use std::{
    fs::OpenOptions,
    io::{BufReader, Cursor, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
};

use criterion::{BenchmarkId, Criterion, Throughput};
use rand::RngCore;
use ratman::scaffold::DataChunk;

pub trait StreamProducer<R: Read> {
    fn produce(&mut self) -> Option<Mutex<Box<dyn Read + Send + 'static>>>;
}

pub trait BenchmarkTarget<P, R>
where
    P: StreamProducer<R>,
    R: Read,
{
    /// Called once to pass a stream producer and stream consumer
    fn init(&mut self, prod: P, cons: StreamConsumer);

    /// Waits for exactly one producer event and executes it
    ///
    /// Returns None if the producer has closed
    fn wait_for_start(&mut self) -> Option<()>;
}

pub struct BenchRunner;

impl BenchRunner {
    // Initialise the pipeline in reverse
    pub fn new(c: &mut Criterion) {
        println!("Initialising BenchRunner");

        let chunk_size = 1024; // * 32;

        // Start a consumer to /dev/null
        let consumer = StreamConsumer::spawn();
        dbg!();

        // Create the middle-data processor and connect it to the consumer
        let (to_proc, from_edge) = channel::<DataChunk>();
        DataProc::spawn(from_edge, consumer.clone());
        dbg!();

        // Start a Tcp listener and give it a way to send data to the processor
        TcpEdgeConnector::spawn("127.0.0.1:7172", to_proc, chunk_size);
        dbg!();

        /////// THE SYSTEM IS READY TO BENCHMARK ///////

        // Get a random number source
        let mut rng = rand::thread_rng();

        let mut group = c.benchmark_group("meh.tcp");
        group.throughput(Throughput::Bytes(chunk_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(chunk_size),
            &chunk_size,
            |b, &cs| {
                // Generate a 1K message
                let mut buf = vec![0 as u8; cs];
                rng.fill_bytes(&mut buf);
                let mut chunk = DataChunk::with_size(cs, buf);
                let mut tcp =
                    TcpStream::connect("127.0.0.1:7172").expect("failed to connect to TCP socket");

                // Repeatedly write into the socket, then immediately check how much has already made it to the other end
                b.iter(|| {
                    tcp.write_all(chunk.as_mut())
                        .expect("failed to write to socket");
                    let tp_now = consumer.throughput.load(Ordering::Acquire);
                    tp_now
                });
            },
        );
    }
}

criterion::criterion_group!(benches, BenchRunner::new);
criterion::criterion_main!(benches);

/// A reader which owns its own stream data
pub struct OwnedReader<T> {
    inner: Vec<T>,
}

impl<T> OwnedReader<T> {
    pub fn as_buf_reader(&mut self) -> BufReader<&Vec<T>> {
        let _c = Cursor::new(&mut self.inner);
        todo!()
    }
}

//////////////// SIMPLE TCP STREAM ARCHITECTURE

pub struct TcpEdgeConnector;

impl TcpEdgeConnector {
    pub fn spawn(addr: &str, to_proc: Sender<DataChunk>, chunk_size: usize) {
        let socket = TcpListener::bind(addr).expect("failed to bind TcpSocket");
        std::thread::spawn(move || {
            let mut inc = socket.incoming();

            while let Some(Ok(stream)) = inc.next() {
                TcpEdgeReader::new(stream, chunk_size).spawn(to_proc.clone())
            }
        });
    }
}

pub struct TcpEdgeReader {
    stream: TcpStream,
    chunk_size: usize,
}

impl TcpEdgeReader {
    pub fn new(stream: TcpStream, chunk_size: usize) -> Self {
        Self { stream, chunk_size }
    }

    pub fn spawn(self, next: Sender<DataChunk>) {
        let Self {
            mut stream,
            chunk_size,
        } = self;

        std::thread::spawn(move || loop {
            // Create a chunk of exactly the right size
            let mut chunk = DataChunk::with_size(chunk_size, vec![0; chunk_size]);

            let mut ctr = 0;
            while ctr < chunk_size {
                let num_bytes = stream.read(chunk.as_mut())
                    .expect("failed to read chunk from TcpStream");
                ctr += num_bytes;
            }

            next.send(chunk).expect("failed to send chunk to <next>");
        });
    }
}

pub struct DataProc;

impl DataProc {
    pub fn spawn(reader: Receiver<DataChunk>, consumer: StreamConsumer) {
        std::thread::spawn(move || {
            while let Ok(mut chunk) = reader.recv() {
                chunk.as_mut().reverse();
                consumer.consume(chunk);
            }
        });
    }
}

#[derive(Clone)]
pub struct StreamConsumer {
    tx: Sender<DataChunk>,
    throughput: Arc<AtomicUsize>,
}

impl StreamConsumer {
    pub fn spawn() -> Self {
        let mut oo = OpenOptions::new()
            .write(true)
            .create(false)
            .open("/dev/null")
            .expect("failed to open: /dev/null");

        let throughput = Arc::new(0.into());
        let (tx, rx) = channel::<DataChunk>();

        let tp: Arc<AtomicUsize> = Arc::clone(&throughput);
        std::thread::spawn(move || {
            let tp = tp;
            while let Ok(mut chunk) = rx.recv() {
                tp.fetch_add(chunk.as_mut().len(), Ordering::Release);
                oo.write_all(chunk.as_mut())
                    .expect("failed to write to /dev/null");
            }
        });

        Self { tx, throughput }
    }

    pub fn consume(&self, chunk: DataChunk) {
        self.tx
            .send(chunk)
            .expect("failed to send DataChunk to StreamConsumer");
    }
}
