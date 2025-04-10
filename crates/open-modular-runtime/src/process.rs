use open_modular_synchronization::barrier::Barriers;

// =================================================================================================
// Process
// =================================================================================================

pub trait Process: AsMut<Barriers> {
    fn configure(&mut self) -> ProcessControl {
        ProcessControl::Continue
    }

    fn compute(&mut self) {}

    fn io(&mut self) {}

    fn process(&mut self) {
        loop {
            if self.configure() == ProcessControl::Exit {
                break;
            }

            self.as_mut().configuration.wait();
            self.compute();
            self.as_mut().compute.wait();
            self.io();
            self.as_mut().io.wait();
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Control

#[derive(Debug, Eq, PartialEq)]
pub enum ProcessControl {
    Continue,
    Exit,
}
