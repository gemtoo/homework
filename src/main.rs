#[macro_use]
extern crate log;
pub const CRATE_NAME: &str = module_path!();
mod logger;
mod misc;
mod program;
mod engine;
use program::{UnscheduledProgram, schedule_programs};

const INPUTS: [&str; 8] = [
    "program1,password1,pid123123,06:01,0.3,5,1.5,10,msp1-1,1,0,ac=*-666666",
    "program2,password2,pid123123,07:01,0.3,5,1.5,10,msp1-1,1,0,ac=*-666666",
    "program3,password3,pid123123,07:02,0.3,5,1.5,10,msp1-1,1,0,ac=*-333333",
    "program4,password4,pid123123,08:03,0.3,5,1.5,10,msp1-1,1,0,ac=*-666666",
    "program5,password5,pid123123,16:01,0.3,5,1.5,10,msp1-1,1,0,ac=*-7777",
    "program6,password6,pid123123,16:02,0.3,5,1.5,10,msp1-1,1,0,ac=*-666555",
    "program7,password7,pid123123,17:03,0.3,5,1.5,10,msp1-1,1,0,ac=*-7777",
    "program8,password8,pid123123,17:04,0.3,5,1.5,10,msp1-1,1,0,ac=*-666555",
];

pub const TIME_SPEEDUP_FACTOR: f32 = 100000.0;

fn main() {
    misc::boot();
    let unscheduled_programs: Vec<UnscheduledProgram> = INPUTS
        .into_iter()
        .map(|input| UnscheduledProgram::from(input))
        .collect();

    let queue = schedule_programs(unscheduled_programs);
    engine::run(queue);
}
