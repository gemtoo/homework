use crate::misc::die;
use chrono::{DateTime, Datelike};
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, Timelike};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub type Tokens<'a> = [&'a str; 12];

#[derive(Debug, Clone)]
pub struct UnscheduledProgram {
    name: String,
    default_start_time: NaiveDateTime,
    start_time_randomization_hours: f64,
    default_execution_hours: f64, // assert that it is not larger than total execution time
    execution_time_randomization_hours: f64,
    program_id: String,
}

impl UnscheduledProgram {
    pub fn from(unparsed_parameters: &str) -> Self {
        debug!("Constructing program object ...");
        let tokenized_parameters = parse_input(unparsed_parameters);
        let name = tokenized_parameters[0].to_string();

        // This is the default value parsed from input, later this value is going to be changed by scheduler in accordance with the requirements.
        let start_time_str = tokenized_parameters[3];
        let start_time_randomization_hours =
            tokenized_parameters[4].parse::<f64>().unwrap_or_else(|e| {
                let reason = format!("Error parsing start time randomization modifier: {}.", e);
                die(reason);
            });
        let default_execution_hours = tokenized_parameters[5].parse::<f64>().unwrap_or_else(|e| {
            let reason = format!("Error parsing execution time: {}.", e);
            die(reason);
        });
        let execution_time_randomization_hours =
            tokenized_parameters[6].parse::<f64>().unwrap_or_else(|e| {
                let reason = format!(
                    "Error parsing execution time randomization modifier: {}.",
                    e
                );
                die(reason);
            });
        let default_start_time = parse_clock_string_to_naive_datetime(start_time_str);

        let program_id = tokenized_parameters[11].to_string();
        return UnscheduledProgram {
            name,
            default_start_time,
            start_time_randomization_hours,
            default_execution_hours,
            execution_time_randomization_hours,
            program_id,
        };
    }
    pub fn schedule_program(self) -> ScheduledProgram {
        trace!("Scheduling {} ...", &self.name);
        let start_delta = generate_randomized_delta_from_hours(self.start_time_randomization_hours);
        let naive_real_start_time =
            apply_delta_to_start_moment(self.default_start_time, start_delta);
        let real_start_time = convert_naive_datetime_to_system_time(naive_real_start_time);
        let execution_delta =
            generate_randomized_delta_from_hours(self.execution_time_randomization_hours);
        let real_execution_duration =
            apply_delta_to_execution_time(self.default_execution_hours, execution_delta);
        return ScheduledProgram {
            name: self.name.clone(),
            real_start_time,
            real_execution_duration,
            program_id: self.program_id.clone(),
            created_from: self,
        };
    }
}

#[derive(Debug, Clone)]
pub struct ScheduledProgram {
    pub name: String,
    pub real_start_time: SystemTime,
    pub real_execution_duration: Duration,
    program_id: String,
    created_from: UnscheduledProgram,
}

impl ScheduledProgram {
    pub fn check_against_the_schedule_and_reschedule_if_necessary(
        mut self,
        schedule: &Vec<ScheduledProgram>,
    ) -> Self {
        trace!("Filtering schedule by program_id ...");
        let schedule_filtered_by_program_id: Vec<&ScheduledProgram> = schedule
            .into_iter()
            .filter(|scheduled_program| scheduled_program.program_id == self.program_id)
            .collect();
        trace!("Checking whether {} is scheduled to run not earlier than 1 hour from the start of the previous one ...", self.name);
        match schedule_filtered_by_program_id.last() {
            Some(last_scheduled_program_with_the_same_id) => {
                // Loop rescheduling until we don't break the 1 hour rule.
                loop {
                    let duration_window_between_runs = self
                        .real_start_time
                        .duration_since(last_scheduled_program_with_the_same_id.real_start_time)
                        // This unwrap is fine as time can't go backwards since we've sorted the array of UnscheduledPrograms beforehand.
                        .unwrap();
                    if duration_window_between_runs <= Duration::from_secs(60 * 60) {
                        warn!(
                            "Duration between starts is less than 60 minutes, rescheduling {} ...",
                            self.name
                        );
                        self = self.created_from.schedule_program();
                        continue;
                    } else {
                        info!(
                            "Scheduled {} to start running for {} seconds at timestamp {}.",
                            self.name,
                            self.real_execution_duration.as_secs(),
                            self.real_start_time
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        );
                        return self;
                    }
                }
            }
            None => {
                debug!("The schedule doesn't yet contain any programs with the same ID.");
                info!(
                    "Scheduled {} to start running for {} seconds at timestamp {}.",
                    self.name,
                    self.real_execution_duration.as_secs(),
                    self.real_start_time
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                );
                return self;
            }
        }
    }
    pub fn run(&self) {
        use crate::misc::sleep;
        trace!("Running {} payload for {} hours ...", self.name, (self.real_execution_duration.as_secs_f32() / 60.0) / 60.0);
        sleep(self.real_execution_duration);
    }
}

fn parse_input(input: &str) -> Tokens {
    trace!("Parsing input: '{}' ...", input);
    let tokenized_parameters: Tokens = input
        .split(",")
        .collect::<Vec<&str>>()
        .try_into()
        .unwrap_or_else(|untokenized_vector: Vec<&str>| {
            let reason = format!(
                "Error tokenizing a vector. Vector length: {}. Needed length: 12.",
                untokenized_vector.len()
            );
            die(reason);
        });
    return tokenized_parameters;
}

fn parse_clock_string_to_naive_datetime(clock_time: &str) -> NaiveDateTime {
    trace!("Parsing time '{}' ...", clock_time);
    let naive_time = NaiveTime::parse_from_str(clock_time, "%H:%M").unwrap_or_else(|e| {
        let reason = format!("Error parsing clock time: {}.", e);
        die(reason);
    });
    let today = Local::now().naive_local();
    let datetime = NaiveDate::from_ymd_opt(today.year(), today.month(), today.day())
        .unwrap()
        .and_hms_opt(naive_time.hour(), naive_time.minute(), 0)
        .unwrap();
    return datetime;
}

fn convert_naive_datetime_to_system_time(datetime: NaiveDateTime) -> SystemTime {
    let duration_since_epoch = datetime.and_utc().timestamp() as u64;
    let system_time = UNIX_EPOCH + Duration::from_secs(duration_since_epoch);
    return system_time;
}

use rand::Rng;
fn generate_randomized_delta_from_hours(hours: f64) -> TimeDelta {
    let max_seconds_delta = (hours * 60.0 * 60.0).round() as i64;
    let mut rng = rand::thread_rng();
    let random_delta = rng.gen_range(-max_seconds_delta..=max_seconds_delta);
    trace!("New random delta is {} seconds.", random_delta);
    // This unwrap is fine as we don't expect insane inputs.
    let timedelta = TimeDelta::new(random_delta, 0).unwrap();
    return timedelta;
}

fn apply_delta_to_start_moment(
    unrandomized_start_time: NaiveDateTime,
    delta: TimeDelta,
) -> NaiveDateTime {
    trace!("Applying delta to start moment ...");
    let randomized_start_time = unrandomized_start_time + delta;
    return randomized_start_time;
}

fn apply_delta_to_execution_time(
    unmodified_execution_time_hours: f64,
    delta: TimeDelta,
) -> Duration {
    trace!("Applying delta to execution time ...");
    let unmodified_execution_time_seconds =
        convert_hours_to_seconds(unmodified_execution_time_hours);
    let unmodified_execution_duration = Duration::from_secs(unmodified_execution_time_seconds);
    let unmodified_execution_timedelta =
        TimeDelta::from_std(unmodified_execution_duration).unwrap();
    // make sure delta is not exceeding 1st arg, then this unwrap is fine
    let modified_duration = (unmodified_execution_timedelta + delta)
        .to_std()
        .unwrap_or_else(|e| {
            let reason = format!("Error. Duration + Delta is negative: {}.", e);
            die(reason);
        });
    return modified_duration;
}

fn convert_hours_to_seconds(hours: f64) -> u64 {
    return (hours * 60.0 * 60.0).round() as u64;
}

// We can determine the start time only when we know about the neighbors, this function schedules the programs
pub fn schedule_programs(unscheduled_programs: Vec<UnscheduledProgram>) -> Vec<ScheduledProgram> {
    info!("Scheduling programs ...");
    // Value is moved into the function then made mutable.
    let mut internal_unscheduled_programs = unscheduled_programs;
    
    // Sorting is done just in case we need to work with custom input.
    trace!("Sorting unscheduled programs by start time ...");
    internal_unscheduled_programs.sort_by(|a, b| a.default_start_time.cmp(&b.default_start_time));
    let mut schedule: Vec<ScheduledProgram> = vec![];
    for (index, internal_unscheduled_program) in internal_unscheduled_programs.iter().enumerate() {
        let scheduled_program = internal_unscheduled_program
            .clone()
            .schedule_program()
            .check_against_the_schedule_and_reschedule_if_necessary(&schedule);
        // Check if we're past the moment, if yes, then shift the schedule for tomorrow.
        if index == 0 {
            debug!("Checking whether a schedule needs to be shifted for tomorrow ...");
            let now = SystemTime::now();
            match scheduled_program.real_start_time.duration_since(now) {
                Ok(_) => {
                    info!("The real start time of 0th ScheduledProgram is in the future. Continuing ...");
                }
                Err(_) => {
                    warn!("The real start time of 0th ScheduledProgram is in the past. Adding +24h to default start time ...");
                    let rescheduled_internal_unscheduled_programs: Vec<UnscheduledProgram> =
                        internal_unscheduled_programs
                            .clone()
                            .into_iter()
                            .map(|mut internal_unscheduled_program| {
                                internal_unscheduled_program.default_start_time +=
                                    Duration::from_secs(60 * 60 * 24);
                                return internal_unscheduled_program;
                            })
                            .collect();
                    // Function runs itself with adjusted default schedule.
                    let programs_for_tomorrow =
                        schedule_programs(rescheduled_internal_unscheduled_programs);
                    return programs_for_tomorrow;
                }
            }
        }
        schedule.push(scheduled_program);
    }
    // Reschedule everything in case some start timings intersect.
    let start_timings: Vec<String> = schedule
        .iter()
        .map(|entry| convert_datetime_into_string_with_precision_to_minutes(entry.real_start_time))
        .collect();
    if has_duplicates(start_timings) {
        return schedule_programs(internal_unscheduled_programs);
    } else {
        return schedule;
    }
}

pub fn convert_datetime_into_string_with_precision_to_minutes(datetime: SystemTime) -> String {
    let timestamp = datetime.duration_since(UNIX_EPOCH).unwrap();
    let datetime = DateTime::from_timestamp(timestamp.as_secs() as i64, 0).unwrap();
    return datetime.format("%Y-%m-%d %H:%M").to_string();
}

fn has_duplicates<T: std::hash::Hash + Eq>(vec: Vec<T>) -> bool {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    for item in vec {
        if !seen.insert(item) {
            warn!("Found intersecting timings ...");
            return true;
        }
    }
    debug!("Intersecting timings were not found.");
    false
}
