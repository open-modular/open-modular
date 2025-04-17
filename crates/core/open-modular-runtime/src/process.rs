use open_modular_synchronization::barrier::Barriers;

// =================================================================================================
// Process
// =================================================================================================

pub trait Process: AsMut<Barriers> {
    fn phase_0(&mut self) -> ProcessControl {
        ProcessControl::Continue
    }

    fn phase_1(&mut self) {}

    fn phase_2(&mut self) {}

    fn process(&mut self) {
        loop {
            if self.phase_0() == ProcessControl::Exit {
                break;
            }

            self.as_mut().phase_0.wait();
            self.phase_1();
            self.as_mut().phase_1.wait();
            self.phase_2();
            self.as_mut().phase_2.wait();
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
