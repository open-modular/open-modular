use std::{
    error::Error,
    sync::{
        Arc,
        atomic::{
            AtomicU64,
            Ordering,
        },
    },
    thread,
    time::{
        Duration,
        Instant,
    },
};

use open_modular_io_audio::{
    Api,
    Audio,
    Device,
    Stream,
    StreamParameters,
    StreamStatus,
};
use rtrb::RingBuffer;
use thread_priority::{
    RealtimeThreadSchedulePolicy,
    ThreadBuilder,
    ThreadPriority,
    ThreadSchedulePolicy,
};

const AMPLITUDE: f32 = 0.5;
const FREQ_HZ: f32 = 440.0;

const NANOS_PER_SEC: u64 = 1_000_000_000;
const BUFFERS_PER_SEC: u64 = 48_000 / 64;
const BUFFER_DURATION: Duration = Duration::from_nanos(NANOS_PER_SEC / BUFFERS_PER_SEC);

pub fn main() -> Result<(), Box<dyn Error>> {
    let api = Api::available()?.next().ok_or("no api found")??;
    let audio = Audio::new(api)?;
    let device = Device::from_raw(&audio, 131)?;

    let parameters = StreamParameters::for_device(device.id).channels(1).build();
    let stream = Stream::output(audio, parameters)?;

    let (mut request_tx, mut request_rx) = RingBuffer::<Instant>::new(1);
    let (mut response_tx, mut response_rx) = RingBuffer::<[f32; 64]>::new(1);

    let producer_loops = Arc::new(AtomicU64::new(0));

    let policy = RealtimeThreadSchedulePolicy::RoundRobin;
    let policy = ThreadSchedulePolicy::Realtime(policy);

    let priority = ThreadPriority::max_value_for_policy(policy).expect("priority");
    let priority = u8::try_from(priority).expect("priority value within range");
    let priority = priority.try_into().expect("priority value");
    let priority = ThreadPriority::Crossplatform(priority);

    let _prod = ThreadBuilder::default()
        .policy(policy)
        .priority(priority)
        .spawn({
            {
                let producer_loops = producer_loops.clone();

                let mut phasor = 0.0;
                let phasor_inc = FREQ_HZ / 48_000f32;

                move |priority| {
                    println!("priority: {priority:?}");

                    loop {
                        let finish = loop {
                            producer_loops.fetch_add(1, Ordering::Relaxed);

                            if request_rx.peek().is_ok() {
                                if let Ok(finish) = request_rx.pop() {
                                    break finish;
                                }
                            }

                            thread::sleep(Duration::from_nanos(10));
                        };

                        let mut data = [0f32; 64];

                        for frame in &mut data {
                            let val = (phasor * std::f32::consts::TAU).sin() * AMPLITUDE;
                            phasor = (phasor + phasor_inc).fract();

                            *frame = val;
                        }

                        if let Err(err) = response_tx.push(data) {
                            println!("error sending response: {err:#?}");
                        }

                        thread::sleep(finish.duration_since(Instant::now()));
                    }
                }
            }
        });

    let response_loops = Arc::new(AtomicU64::new(0));

    let _stream = stream.activate({
        let response_loops = response_loops.clone();

        move |data, info| {
            let finish = Instant::now() + BUFFER_DURATION.mul_f32(0.9);

            // finish.

            match info.status {
                StreamStatus::Overflow => println!("overflow"),
                StreamStatus::Underflow => println!("underflow"),
                _ => {}
            }

            if let Err(err) = request_tx.push(finish) {
                println!("error sending request: {err:#?}");
            }

            thread::sleep(finish.duration_since(Instant::now()));

            let response = loop {
                response_loops.fetch_add(1, Ordering::Relaxed);

                if response_rx.peek().is_ok() {
                    if let Ok(response) = response_rx.pop() {
                        break response;
                    }
                }
            };

            data.copy_from_slice(&response[..]);
        }
    })?;

    thread::sleep(Duration::from_secs(5));

    println!("producer_loops: {producer_loops:?}");
    println!("response_loops: {response_loops:?}");

    println!("stopping");

    Ok(())
}
