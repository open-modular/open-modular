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
        ModuleInstanceReference,
    },
    processor::ProcessToken,
};

// =================================================================================================
// Port
// =================================================================================================

/// Port is a generic return type, used to distinguish cases where something
/// which may be connected or disconnected holds different data depending on
/// state (in this case, no data is held when disconnected).
///
/// This type may be aliased as an internal value type or used as a direct
/// return type.
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

// -------------------------------------------------------------------------------------------------

// Connect

pub(crate) trait PortConnect<P> {
    fn connect(&self, other: &P);
}

impl PortConnect<Arc<SyncUnsafeCell<PortInput>>> for Arc<SyncUnsafeCell<PortOutput>> {
    fn connect(&self, input: &Arc<SyncUnsafeCell<PortInput>>) {
        unsafe {
            (*self.get()) = PortOutput::new(Box::new([Vector::default(); 2]));
            (*input.get()) = PortInput::new(self.clone());
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Disconnect

pub(crate) trait PortDisconnect {
    fn disconnect(&mut self);
}

// -------------------------------------------------------------------------------------------------

// Input

pub(crate) type PortInput = Port<Arc<SyncUnsafeCell<PortOutput>>>;

pub(crate) trait PortInputGet {
    fn port(&self, port: usize) -> Option<&Arc<SyncUnsafeCell<PortInput>>>;
}

impl PortInputGet for PortInputs {
    fn port(&self, port: usize) -> Option<&Arc<SyncUnsafeCell<PortInput>>> {
        self.inputs.get(port)
    }
}

// -------------------------------------------------------------------------------------------------

// Input Definition

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

// -------------------------------------------------------------------------------------------------

// Input Reference

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

// Input Vector

pub trait PortInputVectorGet {
    fn vector(&self, port: usize, token: &ProcessToken) -> Option<Port<&Vector>>;
}

impl PortInputVectorGet for PortInputs {
    fn vector(&self, port: usize, token: &ProcessToken) -> Option<Port<&Vector>> {
        self.inputs
            .get(port)
            .map(|input| match unsafe { &(*input.get()) } {
                PortInput::Connected(output) => match unsafe { &(*output.get()) } {
                    PortOutput::Connected(vectors) => {
                        Port::new(unsafe { vectors.get_unchecked(usize::from(token.0 == 0)) })
                    }
                    PortOutput::Disconnected => Port::Disconnected,
                },
                PortInput::Disconnected => Port::Disconnected,
            })
    }
}

// -------------------------------------------------------------------------------------------------

// Inputs

#[derive(new, Debug)]
pub struct PortInputs {
    inputs: Vec<Arc<SyncUnsafeCell<PortInput>>>,
}

impl PortInputs {
    #[must_use]
    pub fn from_definition(definition: &ModuleDefinition) -> Self {
        let input = definition.inputs.iter().map(|_| Arc::default()).collect();

        Self::new(input)
    }
}

// -------------------------------------------------------------------------------------------------

// Output

pub(crate) type PortOutput = Port<Box<[Vector; 2]>>;

pub(crate) trait PortOutputGet {
    fn port(&self, port: usize) -> Option<&Arc<SyncUnsafeCell<PortOutput>>>;
}

impl PortOutputGet for PortOutputs {
    fn port(&self, port: usize) -> Option<&Arc<SyncUnsafeCell<PortOutput>>> {
        self.outputs.get(port)
    }
}

// -------------------------------------------------------------------------------------------------

// Output Definition

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

// -------------------------------------------------------------------------------------------------

// Output Reference

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

// Output Vector

pub trait PortOutputVectorGet {
    fn vector(&mut self, port: usize, token: &ProcessToken) -> Option<Port<&mut Vector>>;

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
        self.outputs
            .get(port)
            .map(|output| match unsafe { &mut (*output.get()) } {
                PortOutput::Connected(vectors) => {
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

// -------------------------------------------------------------------------------------------------

// Outputs

#[derive(new, Debug)]
pub struct PortOutputs {
    outputs: Vec<Arc<SyncUnsafeCell<PortOutput>>>,
}

impl PortOutputs {
    #[must_use]
    pub fn from_definition(definition: &ModuleDefinition) -> Self {
        let output = definition.outputs.iter().map(|_| Arc::default()).collect();

        Self::new(output)
    }
}

// -------------------------------------------------------------------------------------------------

// Reference

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PortReference {
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
