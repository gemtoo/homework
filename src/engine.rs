use crate::program::ScheduledProgram;
use std::sync::Arc;
use std::thread;
use std_semaphore::Semaphore;
use std::time::SystemTime;
use crate::misc::sleep;
use crate::program::convert_datetime_into_string_with_precision_to_minutes;

pub fn run(queue: Vec<ScheduledProgram>) {
    info!("Preparing to run the queue of tasks ...");
    // Semaphore with 4 permits
    debug!("Creating a semaphore with 4 permits.");
    let semaphore = Arc::new(Semaphore::new(4));
    let mut handles = vec![];
    for program in queue.clone() {
        let semaphore = Arc::clone(&semaphore);
        debug!("Spawning thread for {}.", &program.name);
        let handle = thread::spawn(move || {
            let now = SystemTime::now();
            let time_to_sleep_before_start = program.real_start_time.duration_since(now).unwrap();
            let starting_point = convert_datetime_into_string_with_precision_to_minutes(program.real_start_time);
            debug!("Task for {} will start at {}, after {} seconds.", program.name, starting_point, time_to_sleep_before_start.as_secs());
            trace!("Thread for {} is sleeping for {} seconds.", program.name, time_to_sleep_before_start.as_secs());
            sleep(time_to_sleep_before_start);
            trace!("Acquiring semaphore permit for {} ...", program.name);
            semaphore.acquire();
            program.run();
            semaphore.release();
            info!("Done running {}.", &program.name);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    let start = queue[0].real_start_time;
    let end = queue.last().unwrap().real_start_time + queue.last().unwrap().real_execution_duration;
    let time_elapsed = end.duration_since(start).unwrap().as_secs_f32();
    info!("Total time spent running tasks: {} hours.", ((time_elapsed / 60.0) / 60.0));
}
