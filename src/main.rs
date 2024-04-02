#[allow(dead_code)]
mod sequence;
use sequence::*;

use std::{
    cell::RefCell,
    hint::black_box,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use itertools::Itertools;

thread_local! {
    pub static SEQUENCER_64_8: RefCell<Sequencer> = RefCell::new(Sequencer::new(64, 64 * 8));
    pub static SEQUENCER_64_16: RefCell<Sequencer> = RefCell::new(Sequencer::new(64, 64 * 16));
    pub static SEQUENCER_64_32: RefCell<Sequencer> = RefCell::new(Sequencer::new(64, 64 * 32));
    pub static SEQUENCER_128_8: RefCell<Sequencer> = RefCell::new(Sequencer::new(128, 128 * 8));
    pub static SEQUENCER_128_16: RefCell<Sequencer> = RefCell::new(Sequencer::new(128, 128 * 16));
    pub static SEQUENCER_128_32: RefCell<Sequencer> = RefCell::new(Sequencer::new(128, 128 * 32));
}

fn coarse(loops: usize) -> Duration {
    let now = Instant::now();
    for _ in 0..loops {
        let _ = coarsetime::Instant::now();
    }
    now.elapsed()
}

fn primitive(loops: usize) -> Duration {
    let mut cnt = 0usize;
    let now = Instant::now();
    for _ in 0..loops {
        cnt += 1;
        let _ = cnt;
    }
    now.elapsed()
}

fn atomic(loops: usize, atomic: Arc<AtomicUsize>) -> Duration {
    let now = Instant::now();
    for _ in 0..loops {
        let _ = atomic.fetch_add(1, Ordering::Relaxed);
    }
    now.elapsed()
}

fn atomic_skip(loops: usize, atomic: Arc<AtomicUsize>, skip: usize) -> Duration {
    let mut cnt = 0usize;
    let now = Instant::now();
    for _ in 0..loops {
        cnt += 1;
        let _ = cnt;
        if cnt % skip == 0 {
            let _ = atomic.fetch_add(skip, Ordering::Relaxed);
        } else {
            let _ = atomic.load(Ordering::Relaxed);
        }
    }
    now.elapsed()
}

fn sequencer(loops: usize, step: Sequence, lag_amp: Sequence) -> Duration {
    let sequencer = match (step, lag_amp) {
        (64, 8) => &SEQUENCER_64_8,
        (64, 16) => &SEQUENCER_64_16,
        (64, 32) => &SEQUENCER_64_32,
        (128, 8) => &SEQUENCER_128_8,
        (128, 16) => &SEQUENCER_128_16,
        (128, 32) => &SEQUENCER_128_32,
        _ => unimplemented!(),
    };
    let now = Instant::now();
    for _ in 0..loops {
        let _ = sequencer.with(|s| s.borrow_mut().inc());
    }
    now.elapsed()
}

fn benchmark<F>(name: &str, threads: usize, loops: usize, f: F)
where
    F: Fn() -> Duration + Clone + Send + 'static,
{
    let handles = (0..threads)
        .map(|_| std::thread::spawn(black_box(f.clone())))
        .collect_vec();
    let mut dur = Duration::from_nanos(0);
    for handle in handles {
        dur += handle.join().unwrap();
    }
    println!(
        "{:20} {} threads {} loops: {:?} per iter",
        name,
        threads,
        loops,
        Duration::from_nanos((dur.as_nanos() / threads as u128 / loops as u128) as u64)
    );
}

fn main() {
    for (threads, loops) in [
        (1, 10_000_000),
        (4, 10_000_000),
        (8, 10_000_000),
        (16, 10_000_000),
        (32, 10_000_000),
    ] {
        println!();

        benchmark("primitive", threads, loops, move || primitive(loops));

        let a = Arc::new(AtomicUsize::new(0));
        benchmark("atomic", threads, loops, move || atomic(loops, a.clone()));

        let a = Arc::new(AtomicUsize::new(0));
        benchmark("atomic skip 8", threads, loops, move || {
            atomic_skip(loops, a.clone(), 8)
        });

        let a = Arc::new(AtomicUsize::new(0));
        benchmark("atomic skip 16", threads, loops, move || {
            atomic_skip(loops, a.clone(), 16)
        });

        let a = Arc::new(AtomicUsize::new(0));
        benchmark("atomic skip 32", threads, loops, move || {
            atomic_skip(loops, a.clone(), 32)
        });

        let a = Arc::new(AtomicUsize::new(0));
        benchmark("atomic skip 64", threads, loops, move || {
            atomic_skip(loops, a.clone(), 64)
        });

        benchmark("sequencer(64,8)", threads, loops, move || {
            sequencer(loops, 64, 8)
        });
        benchmark("sequencer(64,16)", threads, loops, move || {
            sequencer(loops, 64, 16)
        });
        benchmark("sequencer(64,32)", threads, loops, move || {
            sequencer(loops, 64, 32)
        });
        benchmark("sequencer(128,8)", threads, loops, move || {
            sequencer(loops, 128, 8)
        });
        benchmark("sequencer(128,16)", threads, loops, move || {
            sequencer(loops, 128, 16)
        });
        benchmark("sequencer(128,32)", threads, loops, move || {
            sequencer(loops, 128, 32)
        });

        benchmark("coarse", threads, loops, move || coarse(loops));
    }
}

/*

Results:

primitive            1 threads 10000000 loops: 0ns per iter
atomic               1 threads 10000000 loops: 1ns per iter
atomic skip 8        1 threads 10000000 loops: 0ns per iter
atomic skip 16       1 threads 10000000 loops: 0ns per iter
atomic skip 32       1 threads 10000000 loops: 0ns per iter
atomic skip 64       1 threads 10000000 loops: 0ns per iter
sequencer(64,8)      1 threads 10000000 loops: 1ns per iter
sequencer(64,16)     1 threads 10000000 loops: 1ns per iter
sequencer(64,32)     1 threads 10000000 loops: 1ns per iter
sequencer(128,8)     1 threads 10000000 loops: 1ns per iter
sequencer(128,16)    1 threads 10000000 loops: 1ns per iter
sequencer(128,32)    1 threads 10000000 loops: 1ns per iter
coarse               1 threads 10000000 loops: 4ns per iter

primitive            4 threads 10000000 loops: 0ns per iter
atomic               4 threads 10000000 loops: 19ns per iter
atomic skip 8        4 threads 10000000 loops: 3ns per iter
atomic skip 16       4 threads 10000000 loops: 4ns per iter
atomic skip 32       4 threads 10000000 loops: 1ns per iter
atomic skip 64       4 threads 10000000 loops: 2ns per iter
sequencer(64,8)      4 threads 10000000 loops: 4ns per iter
sequencer(64,16)     4 threads 10000000 loops: 4ns per iter
sequencer(64,32)     4 threads 10000000 loops: 3ns per iter
sequencer(128,8)     4 threads 10000000 loops: 3ns per iter
sequencer(128,16)    4 threads 10000000 loops: 3ns per iter
sequencer(128,32)    4 threads 10000000 loops: 2ns per iter
coarse               4 threads 10000000 loops: 14ns per iter

primitive            8 threads 10000000 loops: 0ns per iter
atomic               8 threads 10000000 loops: 60ns per iter
atomic skip 8        8 threads 10000000 loops: 10ns per iter
atomic skip 16       8 threads 10000000 loops: 8ns per iter
atomic skip 32       8 threads 10000000 loops: 6ns per iter
atomic skip 64       8 threads 10000000 loops: 4ns per iter
sequencer(64,8)      8 threads 10000000 loops: 8ns per iter
sequencer(64,16)     8 threads 10000000 loops: 7ns per iter
sequencer(64,32)     8 threads 10000000 loops: 6ns per iter
sequencer(128,8)     8 threads 10000000 loops: 6ns per iter
sequencer(128,16)    8 threads 10000000 loops: 6ns per iter
sequencer(128,32)    8 threads 10000000 loops: 5ns per iter
coarse               8 threads 10000000 loops: 23ns per iter

primitive            16 threads 10000000 loops: 0ns per iter
atomic               16 threads 10000000 loops: 126ns per iter
atomic skip 8        16 threads 10000000 loops: 29ns per iter
atomic skip 16       16 threads 10000000 loops: 16ns per iter
atomic skip 32       16 threads 10000000 loops: 11ns per iter
atomic skip 64       16 threads 10000000 loops: 8ns per iter
sequencer(64,8)      16 threads 10000000 loops: 23ns per iter
sequencer(64,16)     16 threads 10000000 loops: 16ns per iter
sequencer(64,32)     16 threads 10000000 loops: 15ns per iter
sequencer(128,8)     16 threads 10000000 loops: 21ns per iter
sequencer(128,16)    16 threads 10000000 loops: 10ns per iter
sequencer(128,32)    16 threads 10000000 loops: 10ns per iter
coarse               16 threads 10000000 loops: 57ns per iter

primitive            32 threads 10000000 loops: 0ns per iter
atomic               32 threads 10000000 loops: 408ns per iter
atomic skip 8        32 threads 10000000 loops: 72ns per iter
atomic skip 16       32 threads 10000000 loops: 41ns per iter
atomic skip 32       32 threads 10000000 loops: 31ns per iter
atomic skip 64       32 threads 10000000 loops: 20ns per iter
sequencer(64,8)      32 threads 10000000 loops: 145ns per iter
sequencer(64,16)     32 threads 10000000 loops: 66ns per iter
sequencer(64,32)     32 threads 10000000 loops: 27ns per iter
sequencer(128,8)     32 threads 10000000 loops: 144ns per iter
sequencer(128,16)    32 threads 10000000 loops: 65ns per iter
sequencer(128,32)    32 threads 10000000 loops: 15ns per iter
coarse               32 threads 10000000 loops: 253ns per iter

*/
