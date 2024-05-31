use std::{
    io::{Read, Write},
    net::{Shutdown, TcpStream},
    sync::{Arc, Barrier},
    thread,
    time::Instant,
};

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

#[path = "../src/bin/tcp_echo_server.rs"]
mod tcp_echo_server;

fn bench_throughput(c: &mut Criterion) {
    thread::spawn(|| {
        tcp_echo_server::main().unwrap();
    });

    let num_clients = 10;
    let num_requests = 1000;
    let barrier = Arc::new(Barrier::new(num_clients + 1));
    let msg = b"Hello, world!";
    let msg_size = msg.len();
    let total_bytes = num_clients * num_requests * msg_size;

    let mut group = c.benchmark_group("throughput");
    group.throughput(Throughput::Bytes(total_bytes as u64));

    group.bench_function("throughput", |b| {
        b.iter_custom(|_| {
            let barrier = Arc::clone(&barrier);
            let handles: Vec<_> = (0..num_clients)
                .map(|i| {
                    let barrier = Arc::clone(&barrier);
                    thread::spawn(move || {
                        let mut stream = TcpStream::connect("[::1]:8080").unwrap();
                        let msg = b"Hello, world!";
                        barrier.wait();

                        for j in 0..num_requests {
                            if let Err(e) = stream.write_all(msg) {
                                eprintln!("Client {}: Write error on request {}: {:?}", i, j, e);
                                break;
                            }
                            let mut buf = [0; 1024];
                            if let Err(e) = stream.read(&mut buf) {
                                eprintln!("Client {}: Read error on request {}: {:?}", i, j, e);
                                break;
                            }
                        }

                        if let Err(e) = stream.shutdown(Shutdown::Both) {
                            eprintln!("Client {}: Error shutting down connection: {:?}", i, e);
                        }
                    })
                })
                .collect();

            barrier.wait(); // Wait for all clients to connect

            let start = Instant::now();

            for handle in handles {
                handle.join().unwrap();
            }

            start.elapsed()
        })
    });

    group.finish();
}

criterion_group!(benches, bench_throughput);
criterion_main!(benches);
