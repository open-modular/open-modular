use fancy_constructor::new;
use open_modular_utils::sync::{
    Barrier,
    BarrierGroup,
};

// =================================================================================================
// Barrier
// =================================================================================================

#[derive(Debug, Default)]
pub struct BarrierGroups {
    compute: BarrierGroup,
    configuration: BarrierGroup,
    io: BarrierGroup,
}

impl BarrierGroups {
    #[must_use]
    pub fn barriers(&self) -> Barriers {
        Barriers::new(
            self.compute.barrier(),
            self.configuration.barrier(),
            self.io.barrier(),
        )
    }
}

#[derive(new, Debug)]
pub struct Barriers {
    pub compute: Barrier,
    pub configuration: Barrier,
    pub io: Barrier,
}
