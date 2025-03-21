use std::{
    cell::SyncUnsafeCell,
    sync::Arc,
};

use bon::Builder;
use fancy_constructor::new;
use open_modular_core::Vector;
use uuid::Uuid;

use crate::module::{
    ModuleDefinition,
    ModuleInstanceReference,
    ProcessToken,
};

// =================================================================================================
// Port
// =================================================================================================

#[derive(new, Debug)]
pub enum Port<T> {
    #[new]
    Connected(T),
    Disconnected,
}

impl<T> Default for Port<T> {
    fn default() -> Self {
        Self::Disconnected
    }
}

#[derive(new, Debug)]
#[new(vis())]
pub struct Ports {
    input: Arc<Vec<Arc<SyncUnsafeCell<PortInput>>>>,
    output: Arc<Vec<Arc<SyncUnsafeCell<PortOutput>>>>,
}

impl Ports {
    #[must_use]
    pub fn from_definition(definition: &ModuleDefinition) -> Self {
        let input = Arc::new(definition.inputs.iter().map(|_| Arc::default()).collect());
        let output = Arc::new(definition.outputs.iter().map(|_| Arc::default()).collect());

        Self::new(input, output)
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

#[derive(new, Debug)]
pub enum PortInput {
    #[new]
    Connected {
        output: Arc<SyncUnsafeCell<PortOutput>>,
    },
    Disconnected,
}

impl Default for PortInput {
    fn default() -> Self {
        Self::Disconnected
    }
}

pub trait GetPortInputVector {
    fn vector(&self, port: usize, token: &ProcessToken) -> Option<Port<&Vector>>;
}

#[derive(new, Debug)]
pub struct PortInputs {
    pub(crate) input: Arc<Vec<Arc<SyncUnsafeCell<PortInput>>>>,
}

impl GetPortInputVector for PortInputs {
    fn vector(&self, port: usize, token: &ProcessToken) -> Option<Port<&Vector>> {
        self.input
            .get(port)
            .map(|input| match unsafe { &(*input.get()) } {
                PortInput::Connected { output } => match unsafe { &(*output.get()) } {
                    PortOutput::Connected { vectors, .. } => {
                        Port::new(unsafe { vectors.get_unchecked(token.0) })
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

impl GetPortInputs for Ports {
    fn inputs(&self) -> PortInputs {
        PortInputs::new(self.input.clone())
    }
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

#[derive(new, Clone, Debug, Eq, PartialEq)]
pub struct PortInputReference {
    pub instance: Uuid,
    pub port: usize,
}

impl PortInputReference {
    #[must_use]
    pub fn instance_ref(&self) -> ModuleInstanceReference {
        ModuleInstanceReference::new(self.instance)
    }
}

// -------------------------------------------------------------------------------------------------

// Output

#[derive(new, Debug)]
pub enum PortOutput {
    #[new]
    Connected {
        input: Arc<SyncUnsafeCell<PortInput>>,
        vectors: Box<[Vector; 2]>,
    },
    Disconnected,
}

impl Default for PortOutput {
    fn default() -> Self {
        Self::Disconnected
    }
}

pub trait GetPortOutputVector {
    fn vector(&mut self, port: usize, token: &ProcessToken) -> Option<Port<&mut Vector>>;

    fn vectors(
        &mut self,
        port: usize,
        token: &ProcessToken,
    ) -> Option<Port<(&mut Vector, &Vector)>>;
}

#[derive(new, Debug)]
pub struct PortOutputs {
    pub(crate) output: Arc<Vec<Arc<SyncUnsafeCell<PortOutput>>>>,
}

impl GetPortOutputVector for PortOutputs {
    fn vector(&mut self, port: usize, token: &ProcessToken) -> Option<Port<&mut Vector>> {
        self.output
            .get(port)
            .map(|output| match unsafe { &mut (*output.get()) } {
                PortOutput::Connected { vectors, .. } => {
                    let current = unsafe { vectors.get_unchecked_mut(token.0) };

                    Port::new(current)
                }
                PortOutput::Disconnected => Port::Disconnected,
            })
    }

    fn vectors(
        &mut self,
        port: usize,
        token: &ProcessToken,
    ) -> Option<Port<(&mut Vector, &Vector)>> {
        self.output
            .get(port)
            .map(|output| match unsafe { &mut (*output.get()) } {
                PortOutput::Connected { vectors, .. } => {
                    let [current, previous] = unsafe {
                        vectors.get_disjoint_unchecked_mut([token.0, usize::from(token.0 == 0)])
                    };

                    let current: &mut Vector = current;
                    let previous: &Vector = previous;

                    Port::new((current, previous))
                }
                PortOutput::Disconnected => Port::Disconnected,
            })
    }
}

pub trait GetPortOutputs {
    fn outputs(&self) -> PortOutputs;
}

impl GetPortOutputs for Ports {
    fn outputs(&self) -> PortOutputs {
        PortOutputs::new(self.output.clone())
    }
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

#[derive(new, Clone, Debug, Eq, PartialEq)]
pub struct PortOutputReference {
    pub instance: Uuid,
    pub port: usize,
}

impl PortOutputReference {
    #[must_use]
    pub fn instance_ref(&self) -> ModuleInstanceReference {
        ModuleInstanceReference::new(self.instance)
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
            (*self.get()) = PortOutput::new(input.clone(), Box::new([Vector::default(); 2]));
            (*input.get()) = PortInput::new(self.clone());
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
