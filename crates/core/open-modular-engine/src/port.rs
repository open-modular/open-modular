//! # Port
//!
//! The `port` module defines types, traits and some implementations for the
//! input and output port types. The port types are provided to modules based on
//! the module definition supplied as part of the module, and a set of inputs
//! and outputs is provided as part of the arguments when instantiating a
//! module.
//!
//! The port types are lock-free, making use of unsafe cells. They are intended
//! to be used in very specific ways and with very specific patterns of thread
//! usage. Only the output port holds data - the input port holds a pointer to
//! the relevant output port, and reads the data from the output port when input
//! is required.
//!
//! The output port actually holds two vectors, and alternates them - on each
//! logical iteration, a module will write to one vector, while the input reads
//! from the other, making read/write logic effectively striped. This should
//! eliminate the need for locking, provided that this access pattern is
//! maintained.

use std::{
    cell::SyncUnsafeCell,
    sync::Arc,
};

use bon::Builder;
use fancy_constructor::new;
use open_modular_core::Vector;

use crate::{
    module::ModuleDefinition,
    processor::ProcessToken,
};

// =================================================================================================
// Port
// =================================================================================================

/// `Port<T>` is a generic return type, used to distinguish cases where
/// something which may be connected or disconnected holds different data
/// depending on state (in this case, no data is held when disconnected).
///
/// This type may be aliased as an internal value type or used as a direct
/// return type. The default state of a port is disconnected.
#[derive(Debug)]
pub enum Port<T> {
    Connected(T),
    Disconnected,
}

impl<T> Default for Port<T> {
    fn default() -> Self {
        Self::Disconnected
    }
}

// -------------------------------------------------------------------------------------------------

// Connect

/// Represents logical port connection, where self is expected to be either an
/// input or output port, and other is expected to be the opposing port type.
pub(crate) trait PortConnect<P> {
    /// Connects two logical ports, of types defined by the parameterisation of
    /// the trait. See documentation on concrete implementations for relevant
    /// implemehtation detail or constraints.
    unsafe fn connect(&self, other: &P);
}

/// Represents logical port connection from output to input.
impl PortConnect<Arc<SyncUnsafeCell<PortInput>>> for Arc<SyncUnsafeCell<PortOutput>> {
    /// Connects a `PortOutput` to a `PortInput` (where both are provided as
    /// `Arc` smart pointers to a `SyncUnsafeCell` containing the port).
    ///
    /// # Panics
    ///
    /// This implementation panics if either or both of the ports are already
    /// connected. Both ports must be in a disconnected state before this
    /// function is called.
    ///
    /// # Safety
    ///
    /// This implementation is not logically thread safe - there is no locking
    /// involved in the implementation, so this should only be called from a
    /// single thread (or where it can be shown that any pairs of output/input
    /// ports are always disjoint - e.g. calling Output A -> Input B and Output
    /// C -> Input D would be safe).
    unsafe fn connect(&self, input: &Arc<SyncUnsafeCell<PortInput>>) {
        let output = unsafe { &mut (*self.get()) };
        let input = unsafe { &mut (*input.get()) };

        match (&output, &input) {
            (Port::Disconnected, Port::Disconnected) => {
                *output = Port::Connected(Box::default());
                *input = Port::Connected(Arc::clone(self));
            }
            (Port::Connected(_), Port::Connected(_)) => panic!("output and input connected"),
            (Port::Connected(_), _) => panic!("output connected"),
            (_, Port::Connected(_)) => panic!("input connected"),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Disconnect

/// Represents logical port disconnection, where self may be either an input or
/// output port. It should disconnect both the port, and the other end of the
/// logical connection.
pub(crate) trait PortDisconnect {
    /// Disconnect, which should result in both self AND the other relevant port
    /// being in the disconnected state after calling.
    unsafe fn disconnect(&self);
}

/// Represents logical disconnection for an input port (note that the current
/// implementation of `PortOutput` does not maintain a reference to any
/// connected `PortInput` and so the dual of this implementation is unlikely to
/// be provided. Any disconnection logic will therefore need to target the input
/// end of a connection).
impl PortDisconnect for Arc<SyncUnsafeCell<PortInput>> {
    /// Disconnects a `PortInput` (provided as an `Arc` smart pointer to a
    /// `SyncUnsafeCell` containing the port).
    ///
    /// # Panics
    ///
    /// This implementation panics if the input or output port is already
    /// disconnected. Both ports must be in a connected state before calling
    /// this function.
    ///
    /// # Safety
    ///
    /// This implementation is not logically thread safe - there is no locking
    /// involved in the implementation, so this should only be called from a
    /// single thread.
    unsafe fn disconnect(&self) {
        let input = unsafe { &mut (*self.get()) };

        match &input {
            Port::Connected(output) => {
                let output = unsafe { &mut (*output.get()) };

                match &output {
                    Port::Connected(_) => {
                        *output = Port::Disconnected;
                        *input = Port::Disconnected;
                    }
                    Port::Disconnected => panic!("output disconnected"),
                }
            }
            Port::Disconnected => panic!("input disconnected"),
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Input

/// `PortInput` is a specialization of the generic Port type, where the data
/// associated with the connected state is a smart pointer to the associated
/// `PortOutput` (which is used for reading data logically available to the
/// port).
pub(crate) type PortInput = Port<Arc<SyncUnsafeCell<PortOutput>>>;

/// Represents the action of obtaining a `PortInput` if one is available within
/// the relevant container.
pub(crate) trait PortInputGet {
    /// Get a `PortInput` at the given index (ports are positional) if the index
    /// is within range.
    fn port(&self, port: usize) -> Option<&Arc<SyncUnsafeCell<PortInput>>>;
}

impl PortInputGet for PortInputs {
    fn port(&self, port: usize) -> Option<&Arc<SyncUnsafeCell<PortInput>>> {
        self.inputs.get(port)
    }
}

// -------------------------------------------------------------------------------------------------

// Input Definition

/// A `PortInputDefinition` defines a single input port which will be present on
/// a module. It carries optional properties, such as name and description. It
/// is not created directly in module definition code, but by using the
/// associated methods on a module definition builder.
#[derive(Builder, Debug)]
#[builder(derive(Debug), on(String, into))]
pub struct PortInputDefinition {
    /// The name of the port
    pub name: Option<String>,
    /// A meaingful description of the port
    pub description: Option<String>,
}

impl<S> From<PortInputDefinitionBuilder<S>> for PortInputDefinition
where
    S: port_input_definition_builder::IsComplete,
{
    fn from(builder: PortInputDefinitionBuilder<S>) -> PortInputDefinition {
        builder.build()
    }
}

// -------------------------------------------------------------------------------------------------

// Input Vector

/// Represents the action of obtaining the vector of an input port if it is
/// available in the relevant container.
pub trait PortInputVectorGet {
    /// Gets the input vector, returning `None` if the port is not available
    /// (generally if the port index is out of range) or `Some` port value which
    /// will contain a reference to the vector if connected.
    fn vector(&self, port: usize, token: &ProcessToken) -> Option<Port<&Vector>>;
}

impl PortInputVectorGet for PortInputs {
    fn vector(&self, port: usize, token: &ProcessToken) -> Option<Port<&Vector>> {
        self.inputs
            .get(port)
            .map(|input| match unsafe { &(*input.get()) } {
                PortInput::Connected(output) => match unsafe { &(*output.get()) } {
                    PortOutput::Connected(vectors) => {
                        Port::Connected(unsafe { vectors.get_unchecked(usize::from(token.0 == 0)) })
                    }
                    PortOutput::Disconnected => Port::Disconnected,
                },
                PortInput::Disconnected => Port::Disconnected,
            })
    }
}

// -------------------------------------------------------------------------------------------------

// Inputs

/// Contains an indexed collection of input ports, generally passed as a
/// constructor argument to a module upon instantiation. Port inputs or their
/// interior contents (such as references to connected vectors) can be obtained
/// using the relevant traits in this module.
#[derive(new, Debug)]
pub struct PortInputs {
    inputs: Vec<Arc<SyncUnsafeCell<PortInput>>>,
}

impl PortInputs {
    #[doc(hidden)]
    #[must_use]
    pub fn from_definition(definition: &ModuleDefinition) -> Self {
        let input = definition.inputs.iter().map(|_| Arc::default()).collect();

        Self::new(input)
    }
}

// -------------------------------------------------------------------------------------------------

// Output

/// `PortOuput` is a specialization of the generic Port type, where the data
/// associated with the connected state is a pair of `Vector`s, used for writing
/// in a striped pattern based on the current iteration.
pub(crate) type PortOutput = Port<Box<[Vector; 2]>>;

/// Represents the action of obtaining a `PortOutput` if one is available within
/// the relevant container.
pub(crate) trait PortOutputGet {
    /// Get a `PortOutput` at the given index (ports are positional) if the
    /// index is within range.
    fn port(&self, port: usize) -> Option<&Arc<SyncUnsafeCell<PortOutput>>>;
}

impl PortOutputGet for PortOutputs {
    fn port(&self, port: usize) -> Option<&Arc<SyncUnsafeCell<PortOutput>>> {
        self.outputs.get(port)
    }
}

// -------------------------------------------------------------------------------------------------

// Output Definition

/// A `PortOutputDefinition` defines a single output port which will be present
/// on a module. It carries optional properties, such as name and description.
/// It is not created directly in module definition code, but by using the
/// associated methods on a module definition builder.
#[derive(Builder, Debug)]
#[builder(derive(Debug), on(String, into))]
pub struct PortOutputDefinition {
    /// The name of the port
    pub name: Option<String>,
    /// A meaingful description of the port
    pub description: Option<String>,
}

impl<S> From<PortOutputDefinitionBuilder<S>> for PortOutputDefinition
where
    S: port_output_definition_builder::IsComplete,
{
    fn from(builder: PortOutputDefinitionBuilder<S>) -> PortOutputDefinition {
        builder.build()
    }
}

// -------------------------------------------------------------------------------------------------

// Output Vector

/// Represents the action of obtaining the vector (or vectors) of an output port
/// if it is available in the relevant container.
pub trait PortOutputVectorGet {
    /// Gets the output vector, returning `None` if the port is not available
    /// (generally if the port index is out of range) or `Some` port value which
    /// will contain a reference to the vector if connected. This is a mutable
    /// vector, and is the vehicle for writing data to a port at the end of a
    /// processing iteration.
    fn vector(&mut self, port: usize, token: &ProcessToken) -> Option<Port<&mut Vector>>;

    /// Gets the current output vector, and the previous output vector,
    /// returning `None` if the port is not available (generally if the port
    /// index is out of range) or `Some` port value which will contain a
    /// reference to the vector if connected.
    ///
    /// The current output vector is a mutable vector, and is the vehicle for
    /// writing data to a port at the end of a processing iteration, the
    /// previous output vector is not mutable, but may be useful when
    /// calculating the current output vector in some operations.
    fn vectors(
        &mut self,
        port: usize,
        token: &ProcessToken,
    ) -> Option<Port<(&mut Vector, &Vector)>>;
}

impl PortOutputVectorGet for PortOutputs {
    fn vector(&mut self, port: usize, token: &ProcessToken) -> Option<Port<&mut Vector>> {
        self.outputs
            .get(port)
            .map(|output| match unsafe { &mut (*output.get()) } {
                PortOutput::Connected(vectors) => {
                    let current = unsafe { vectors.get_unchecked_mut(token.0) };

                    Port::Connected(current)
                }
                PortOutput::Disconnected => Port::Disconnected,
            })
    }

    fn vectors(
        &mut self,
        port: usize,
        token: &ProcessToken,
    ) -> Option<Port<(&mut Vector, &Vector)>> {
        self.outputs
            .get(port)
            .map(|output| match unsafe { &mut (*output.get()) } {
                PortOutput::Connected(vectors) => {
                    let [current, previous] = unsafe {
                        vectors.get_disjoint_unchecked_mut([token.0, usize::from(token.0 == 0)])
                    };

                    let current: &mut Vector = current;
                    let previous: &Vector = previous;

                    Port::Connected((current, previous))
                }
                PortOutput::Disconnected => Port::Disconnected,
            })
    }
}

// -------------------------------------------------------------------------------------------------

// Outputs

/// Contains an indexed collection of output ports, generally passed as a
/// constructor argument to a module upon instantiation. Port outputs or their
/// interior contents (such as output vectors) can be obtained using the
/// relevant traits in this module.
#[derive(new, Debug)]
pub struct PortOutputs {
    outputs: Vec<Arc<SyncUnsafeCell<PortOutput>>>,
}

impl PortOutputs {
    #[doc(hidden)]
    #[must_use]
    pub fn from_definition(definition: &ModuleDefinition) -> Self {
        let output = definition.outputs.iter().map(|_| Arc::default()).collect();

        Self::new(output)
    }
}
