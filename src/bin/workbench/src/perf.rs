use std::time::{
    Duration,
    Instant,
};

use open_modular_engine::{
    module::{
        ModuleIdentify as _,
        ModuleSource,
    },
    processor::Processor,
};
use open_modular_module_gen::Sine;
use open_modular_module_util::Multiple;
use uuid::Uuid;

use crate::module::ModulePerf;

// =================================================================================================
// Performance
// =================================================================================================

#[allow(clippy::cast_precision_loss)]
#[allow(dead_code)]
pub fn run() {
    let mut processor = Processor::<ModulePerf<()>>::default();

    for _ in 0..50 {
        let a_ref = processor.add(Uuid::new_v4(), ModulePerf::get(&Sine::<()>::id(), ()));
        let b_ref = processor.add(Uuid::new_v4(), ModulePerf::get(&Multiple::<()>::id(), ()));

        unsafe {
            processor.connect(&a_ref.output_ref(0), &b_ref.input_ref(0));
        }
    }

    let iterations = 1_000_000;
    let start = Instant::now();

    for i in 0..iterations {
        processor.process(i);
    }

    let stop = Instant::now();
    let elapsed = stop - start;

    println!("total: {elapsed:#?}");

    let iteration = elapsed / u32::try_from(iterations).expect("iterations to be within u32 range");

    println!("iteration: {iteration:#?}");

    let iterations_per_s = Duration::from_secs(1).as_nanos() / iteration.as_nanos();

    println!("iteration/s: {iterations_per_s}");

    let iterations_required = 44_100 / 64;
    let iterations_usage = (f64::from(iterations_required)) / (iterations_per_s as f64) * 100.;

    println!("load: {iterations_usage:.5}%");
}
