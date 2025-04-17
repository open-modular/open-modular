use std::{
    fmt::Debug,
    thread::{
        self,
        ScopedJoinHandle,
    },
};

use crossbeam::channel::{
    self,
    Receiver,
};
use fancy_constructor::new;
use open_modular_engine::module::{
    Module,
    ModuleSource,
};
#[cfg(feature = "perf")]
use open_modular_performance::timing::TimingAggregator;
use open_modular_runtime::runtime;
use open_modular_synchronization::{
    barrier::BarrierGroups,
    control::Exit,
};
use open_modular_utilities::thread::BuilderExt as _;
use thread_priority::{
    RealtimeThreadSchedulePolicy,
    ThreadBuilder,
    ThreadPriority,
    ThreadSchedulePolicy,
};

#[cfg(feature = "perf")]
use crate::process::statistics::Statistics;
use crate::{
    context::Context,
    process::{
        compute::Compute,
        control::Control,
        io::{
            Io,
            audio::{
                AudioContext,
                AudioProtocol,
            },
        },
    },
};

// =================================================================================================
// Runtime
// =================================================================================================

// Module

#[rustfmt::skip]
pub trait RuntimeModule =
      Debug 
    + Module
    + ModuleSource<Context = <Runtime as runtime::Runtime>::Context>;

// -------------------------------------------------------------------------------------------------

// Runtime

#[derive(new, Debug)]
#[new(vis())]
pub struct Runtime {
    pub(crate) audio_receiver: Receiver<AudioProtocol>,
    #[new(default)]
    pub(crate) barrier_groups: BarrierGroups,
    pub(crate) context: <Self as runtime::Runtime>::Context,
    #[new(default)]
    pub(crate) exit: Exit,

    #[cfg(feature = "perf")]
    #[new(default)]
    pub timing_aggregator: TimingAggregator,
}

impl Runtime {
    fn complete(handle: ScopedJoinHandle<'_, ()>, _name: &str) {
        match handle.join() {
            Ok(()) => {}
            Err(_err) => {}
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        let audio_channels = channel::unbounded();
        let audio_context = AudioContext::new(audio_channels.0);

        let context = Context::new(audio_context);

        Self::new(audio_channels.1, context)
    }
}

impl runtime::Runtime for Runtime {
    type Context = Context;

    fn run<M>(&self)
    where
        M: RuntimeModule,
    {
        thread::scope(|scope| {
            // NOTE: Creating the barriers before spawning the threads prevents a race
            // condition where a thread has already passed a barrier wait before
            // other threads have increased the active count, thus leading to an
            // unsynchronized set of barriers (and a resultant deadlock). Any
            // additional new barriers should always be created before the first
            // barrier is reached, in the logical configuration phase.

            let compute = self.barrier_groups.barriers();
            let io = self.barrier_groups.barriers();

            let compute = {
                let policy = RealtimeThreadSchedulePolicy::RoundRobin;
                let policy = ThreadSchedulePolicy::Realtime(policy);

                let priority = ThreadPriority::max_value_for_policy(policy).expect("priority");
                let priority = u8::try_from(priority).expect("priority value within range");
                let priority = priority.try_into().expect("priority value");
                let priority = ThreadPriority::Crossplatform(priority);

                ThreadBuilder::default()
                    .named("compute")
                    .policy(policy)
                    .priority(priority)
                    .spawn_scoped(scope, |priority| match priority {
                        Ok(()) => Compute::<M>::spawn(self, compute),
                        Err(_err) => {}
                    })
                    .expect("compute thread to spawn without error")
            };

            let control = ThreadBuilder::default()
                .named("control")
                .spawn_scoped(scope, |_| Control::spawn(self))
                .expect("control thread to spawn without error");

            let io = ThreadBuilder::default()
                .named("io")
                .spawn_scoped(scope, |_| Io::spawn(self, io))
                .expect("io thread to spawn without error");

            #[cfg(feature = "perf")]
            let statistics = ThreadBuilder::default()
                .named("statistics")
                .spawn_scoped(scope, |_| Statistics::spawn(self))
                .expect("statistics thread to spawn without error");

            Self::complete(compute, stringify!(compute));
            Self::complete(control, stringify!(control));
            Self::complete(io, stringify!(io));

            #[cfg(feature = "perf")]
            Self::complete(statistics, stringify!(statistics));
        });
    }
}
