use std::fmt::Debug;

use criterion::{
    Criterion,
    black_box,
    criterion_group,
    criterion_main,
};
use open_modular_engine::{
    module::{
        ModuleIdentify as _,
        ModuleSource as _,
        module_enum,
    },
    processor::Processor,
};
use open_modular_modules_generators::Sine;
use open_modular_modules_utilities::Multiple;
use uuid::Uuid;

// =================================================================================================
// Engine
// =================================================================================================

criterion_main!(engine);

criterion_group!(engine, process_sin_750);

// -------------------------------------------------------------------------------------------------

// Processing

fn process_sin_750(criterion: &mut Criterion) {
    let mut processor = Processor::<Module<()>>::default();

    for _ in 0..50 {
        let a_ref = processor.add(Uuid::new_v4(), Module::get(&Sine::<()>::id(), ()));
        let b_ref = processor.add(Uuid::new_v4(), Module::get(&Multiple::<()>::id(), ()));

        unsafe {
            processor.connect(&a_ref.output_ref(0), &b_ref.input_ref(0));
        }
    }

    criterion.bench_function("process sin 750", |bencher| {
        bencher.iter(|| {
            for i in 0..750 {
                processor.process(black_box(i));
            }
        });
    });
}

// -------------------------------------------------------------------------------------------------

// Module

#[module_enum(id = "68f9841f-983d-4eb0-a99d-444a615436d6")]
#[derive(Debug)]
pub enum Module<R>
where
    R: Debug,
{
    Sine,
    Multiple,
}
