use fancy_constructor::new;
use open_modular_utilities::sync::{
    Barrier,
    BarrierGroup,
};

// =================================================================================================
// Barrier
// =================================================================================================

#[allow(clippy::struct_field_names)]
#[derive(Debug, Default)]
pub struct BarrierGroups {
    phase_0: BarrierGroup,
    phase_1: BarrierGroup,
    phase_2: BarrierGroup,
}

impl BarrierGroups {
    #[must_use]
    pub fn barriers(&self) -> Barriers {
        Barriers::new(
            self.phase_0.barrier(),
            self.phase_1.barrier(),
            self.phase_2.barrier(),
        )
    }
}

#[derive(new, Debug)]
pub struct Barriers {
    pub phase_0: Barrier,
    pub phase_1: Barrier,
    pub phase_2: Barrier,
}
