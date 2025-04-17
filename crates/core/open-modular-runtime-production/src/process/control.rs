use std::{
    fmt::Debug,
    marker::PhantomData,
    str::FromStr as _,
    thread,
    time::Duration,
};

use crossbeam::channel::Sender;
use fancy_constructor::new;
use open_modular_engine::processor::{
    ProcessorProtocol,
    ProcessorProtocolAdd,
    ProcessorProtocolConnect,
};
use open_modular_synchronization::control::Exit;
use uuid::Uuid;

use crate::runtime::Runtime;

// =================================================================================================
// Control
// =================================================================================================

#[derive(new, Debug)]
#[new(args(runtime: &'rt Runtime), vis())]
pub struct Control<'rt> {
    #[new(val = runtime.exit.clone())]
    exit: Exit,
    processor_sender: Sender<ProcessorProtocol>,
    #[new(default)]
    _rt: PhantomData<&'rt ()>,
}

impl<'rt> Control<'rt> {
    pub fn spawn(runtime: &'rt Runtime, processor_sender: Sender<ProcessorProtocol>) {
        Self::new(runtime, processor_sender).process();
    }
}

impl Control<'_> {
    fn process(&mut self) {
        let sine_instance = Uuid::new_v4();
        let sine_module = Uuid::from_str("f75487a4-7847-43f9-ab47-71bd6acfb78d").unwrap();

        let mult_instance = Uuid::new_v4();
        let mult_module = Uuid::from_str("54d93000-7dd2-45ce-a3f1-ad53b0a04fac").unwrap();

        let out_instance = Uuid::new_v4();
        let out_module = Uuid::from_str("47d0fca2-cb58-4011-8a55-31ecd4b184c1").unwrap();

        // Add

        self.processor_sender
            .send(ProcessorProtocolAdd::new(sine_instance, sine_module).into())
            .unwrap();

        self.processor_sender
            .send(ProcessorProtocolAdd::new(mult_instance, mult_module).into())
            .unwrap();

        self.processor_sender
            .send(ProcessorProtocolAdd::new(out_instance, out_module).into())
            .unwrap();

        // Connect

        self.processor_sender
            .send(ProcessorProtocolConnect::new(mult_instance, 0, sine_instance, 0).into())
            .unwrap();

        self.processor_sender
            .send(ProcessorProtocolConnect::new(out_instance, 0, mult_instance, 0).into())
            .unwrap();

        self.processor_sender
            .send(ProcessorProtocolConnect::new(out_instance, 1, mult_instance, 1).into())
            .unwrap();

        thread::sleep(Duration::from_secs(180));

        self.exit.trigger();
    }
}
