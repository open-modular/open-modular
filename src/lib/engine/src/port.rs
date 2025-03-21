use std::{
    cell::SyncUnsafeCell,
    sync::Arc,
};

use bon::Builder;
use fancy_constructor::new;
use open_modular_core::Vector;
use uuid::Uuid;

use crate::{
    module::{
        ModuleDefinition,
        ProcessToken,
    },
    processor::InstanceRef,
};

// =================================================================================================
// Port
// =================================================================================================

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

#[derive(Debug)]
pub struct Ports {
    input: Arc<Vec<Arc<SyncUnsafeCell<PortInput>>>>,
    output: Arc<Vec<Arc<SyncUnsafeCell<PortOutput>>>>,
}

impl Ports {
    #[must_use]
    pub fn from_definition(definition: &ModuleDefinition) -> Self {
        let input = Arc::new(definition.inputs.iter().map(|_| Arc::default()).collect());
        let output = Arc::new(definition.outputs.iter().map(|_| Arc::default()).collect());

        Self { input, output }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PortReference {
    Input(PortInputReference),
    Output(PortOutputReference),
}

impl From<PortInputReference> for PortReference {
    fn from(input_ref: PortInputReference) -> Self {
        Self::Input(input_ref)
    }
}

impl From<PortOutputReference> for PortReference {
    fn from(output_ref: PortOutputReference) -> Self {
        Self::Output(output_ref)
    }
}

// -------------------------------------------------------------------------------------------------

// Input

#[derive(Debug)]
pub enum PortInput {
    Connected(Arc<SyncUnsafeCell<PortOutput>>),
    Disconnected,
}

impl Default for PortInput {
    fn default() -> Self {
        Self::Disconnected
    }
}

pub trait GetPortInput {
    fn port(&self, input: usize, token: &ProcessToken) -> Option<Port<&Vector>>;
}

#[derive(new, Debug)]
pub struct PortInputs {
    pub(crate) inputs: Arc<Vec<Arc<SyncUnsafeCell<PortInput>>>>,
}

impl GetPortInput for PortInputs {
    fn port(&self, input: usize, token: &ProcessToken) -> Option<Port<&Vector>> {
        self.inputs
            .get(input)
            .map(|input| match unsafe { &(*input.get()) } {
                PortInput::Connected(output) => match unsafe { &(*output.get()) } {
                    PortOutput::Connected(_, vectors) => {
                        Port::Connected(unsafe { vectors.get_unchecked(token.0) })
                    }
                    PortOutput::Disconnected => Port::Disconnected,
                },
                PortInput::Disconnected => Port::Disconnected,
            })
    }
}

pub trait GetPortInputs {
    fn inputs(&self) -> PortInputs;
}

impl<T> GetPortInputs for T
where
    T: AsRef<Ports>,
{
    fn inputs(&self) -> PortInputs {
        PortInputs::new(self.as_ref().input.clone())
    }
}

#[derive(Builder, Debug)]
#[builder(derive(Debug), on(String, into))]
pub struct PortInputDefinition {
    pub name: Option<String>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortInputReference(pub (Uuid, usize));

impl PortInputReference {
    #[must_use]
    pub fn instance_ref(&self) -> InstanceRef {
        InstanceRef(self.0.0)
    }
}

// -------------------------------------------------------------------------------------------------

// Output

#[derive(Debug)]
pub enum PortOutput {
    Connected(Arc<SyncUnsafeCell<PortInput>>, Box<[Vector; 2]>),
    Disconnected,
}

impl Default for PortOutput {
    fn default() -> Self {
        Self::Disconnected
    }
}

pub trait GetPortOutput {
    fn port(&mut self, output: usize, token: &ProcessToken)
    -> Option<Port<(&mut Vector, &Vector)>>;
}

#[derive(new, Debug)]
pub struct PortOutputs {
    pub(crate) outputs: Arc<Vec<Arc<SyncUnsafeCell<PortOutput>>>>,
}

impl GetPortOutput for PortOutputs {
    fn port(
        &mut self,
        output: usize,
        token: &ProcessToken,
    ) -> Option<Port<(&mut Vector, &Vector)>> {
        self.outputs
            .get(output)
            .map(|output| match unsafe { &mut (*output.get()) } {
                PortOutput::Connected(_, vectors) => {
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

pub trait GetPortOutputs {
    fn outputs(&self) -> PortOutputs;
}

impl<T> GetPortOutputs for T
where
    T: AsRef<Ports>,
{
    fn outputs(&self) -> PortOutputs {
        PortOutputs::new(self.as_ref().output.clone())
    }
}

#[derive(Builder, Debug)]
#[builder(derive(Debug), on(String, into))]
pub struct PortOutputDefinition {
    pub name: Option<String>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortOutputReference(pub (Uuid, usize));

impl PortOutputReference {
    #[must_use]
    pub fn instance_ref(&self) -> InstanceRef {
        InstanceRef(self.0.0)
    }
}

// -------------------------------------------------------------------------------------------------

// Connection

pub(crate) trait Connect<P> {
    fn connect(&self, other: &P);
}

impl Connect<Arc<SyncUnsafeCell<PortInput>>> for Arc<SyncUnsafeCell<PortOutput>> {
    fn connect(&self, input: &Arc<SyncUnsafeCell<PortInput>>) {
        unsafe {
            (*self.get()) = PortOutput::Connected(input.clone(), Box::new([Vector::default(); 2]));
        }

        unsafe {
            (*input.get()) = PortInput::Connected(self.clone());
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Disconnection

#[allow(dead_code)]
pub(crate) trait Disconnect {
    fn disconnect(&mut self);
}

// impl<P> Disconnect for Port<P>
// where
//     P: Disconnect,
// {
//     fn disconnect(&mut self) {
//         unsafe {
//             (*self.0.get()).disconnect();
//         }
//     }
// }

// impl Disconnect for Input {
//     fn disconnect(&mut self) {
//         if let Self::Connected(output) = self {
//             unsafe {
//                 (*output.get()) = Output::Disconnected;
//             }
//         }

//         *self = Self::Disconnected;
//     }
// }

// impl Disconnect for Output {
//     fn disconnect(&mut self) {
//         if let Self::Connected(input, _) = self {
//             unsafe {
//                 (*input.get()) = Input::Disconnected;
//             }
//         }

//         *self = Self::Disconnected;
//     }
// }
