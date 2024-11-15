use std::io::{stdout, Write};

pub fn update() {
    stdout().flush().unwrap();
}

pub fn boot()
{
    use crate::logger;
    logger::init();
    let _ = ctrlc::set_handler(move || {
        info!("Stopping gracefully ...");
        update();
        std::process::exit(0);
    });
}

pub fn die(reason: impl Into<String>) -> ! {
    error!("{}", reason.into());
    std::process::exit(1);
}

use std::time::Duration;
use std::thread;

use crate::TIME_SPEEDUP_FACTOR;
pub fn sleep(duration: Duration) {
    let scaled_duration = Duration::from_secs_f32(duration.as_secs_f32() / TIME_SPEEDUP_FACTOR);
    thread::sleep(scaled_duration);
}