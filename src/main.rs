use bsec::bme::test_support::{FakeBmeSensor, UnitError};
use bsec::clock::TimePassed;
use bsec::{
    clock::{test_support::FakeClock, Clock},
    Bsec, Input, InputKind, OutputKind, SampleRate, SubscriptionRequest,
};
use nb::{block, Result};
use std::thread::sleep;
use std::time::Duration;

fn sleep_for(dur: Duration) {
    println!("sleeping for {}us", dur.as_micros());
    sleep(dur);
}

fn main() {
    let clock = TimePassed::new();
    // let clock = FakeClock::new();
    let fake_measurements = vec![Input {
        signal: 69.0,
        sensor: InputKind::Pressure,
    }];

    let sensor = FakeBmeSensor::new(Ok(fake_measurements));

    // Acquire handle to the BSEC library.
    // Only one such handle can be acquired at any time.
    let mut bsec: Bsec<_, TimePassed, _> = Bsec::init(sensor, &clock).unwrap();

    // Configure the outputs you want to subscribe to.
    bsec.update_subscription(&[SubscriptionRequest {
        sample_rate: SampleRate::Continuous,
        sensor: OutputKind::RawPressure,
    }])
    .unwrap();

    // We need to feed BSEC regularly with new measurements.
    loop {
        // Wait for when the next measurement is due.
        let next_measurement = bsec.next_measurement();
        let clock_ts = clock.timestamp_ns();
        println!(
            "bsec-ts={}, clock-ts={}, bsec-clock={}",
            next_measurement,
            clock_ts,
            next_measurement - clock_ts
        );

        if next_measurement > clock_ts {
            sleep_for(Duration::from_nanos(
                (next_measurement - clock_ts) as u64,
            ));
        }

        // Start the measurement.
        let wait_duration = block!(bsec.start_next_measurement()).unwrap();
        sleep_for(wait_duration);

        // Process the measurement when ready and print the BSEC outputs.
        let outputs = block!(bsec.process_last_measurement()).unwrap();
        for output in &outputs {
            println!("{:?}: {}", output.sensor, output.signal);
        }
    }
}
