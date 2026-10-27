use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tikzforge_core::compile::{run_with_timeout, RunOutcome};

fn spawn(args: &[&str]) -> std::process::Child {
    Command::new(args[0])
        .args(&args[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("test helper process failed to spawn")
}

fn generation(value: u64) -> AtomicU64 {
    AtomicU64::new(value)
}

#[test]
fn returns_finished_before_timeout() {
    let gen_atomic = generation(1);
    let mut child = spawn(&["sh", "-c", "echo hello"]);
    match run_with_timeout(&mut child, Duration::from_secs(10), &gen_atomic, 1) {
        Ok(RunOutcome::Finished(status)) => assert!(status.success()),
        other => panic!("expected finished outcome, got {other:?}"),
    }
}

#[test]
fn kills_process_on_timeout() {
    let gen_atomic = generation(1);
    let start = Instant::now();
    let mut child = spawn(&["sleep", "60"]);
    match run_with_timeout(&mut child, Duration::from_secs(1), &gen_atomic, 1) {
        Ok(RunOutcome::TimedOut) => {
            assert!(
                start.elapsed() < Duration::from_secs(30),
                "timeout did not fire promptly"
            );
        }
        other => panic!("expected timeout, got {other:?}"),
    }
}

#[test]
fn kills_process_superseded_by_a_newer_generation() {
    let gen_atomic = generation(2);
    let mut child = spawn(&["sleep", "60"]);
    match run_with_timeout(&mut child, Duration::from_secs(60), &gen_atomic, 1) {
        Ok(RunOutcome::Cancelled) => {}
        other => panic!("expected cancelled outcome, got {other:?}"),
    }
}

#[test]
fn newer_generation_mid_flight_stops_a_running_process() {
    let gen_atomic = generation(1);
    let mut child = spawn(&["sleep", "60"]);
    let start = Instant::now();
    std::thread::scope(|s| {
        s.spawn(|| {
            std::thread::sleep(Duration::from_millis(200));
            gen_atomic.store(2, Ordering::SeqCst);
        });
        match run_with_timeout(&mut child, Duration::from_secs(60), &gen_atomic, 1) {
            Ok(RunOutcome::Cancelled) => {
                assert!(
                    start.elapsed() < Duration::from_secs(30),
                    "supersede did not stop the process promptly"
                );
            }
            other => panic!("expected cancelled outcome, got {other:?}"),
        }
    });
}