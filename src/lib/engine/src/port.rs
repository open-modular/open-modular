use std::{
    cell::SyncUnsafeCell,
    sync::Arc,
};

use bon::Builder;
use open_modular_core::Vector;
use uuid::Uuid;

use crate::{
    module::ProcessToken,
    processor::InstanceRef,
};

// =================================================================================================
// Port
// =================================================================================================

#[derive(Debug, Default)]
pub struct Port<P>(pub(crate) Arc<SyncUnsafeCell<P>>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PortRef {
    Input(InputRef),
    Output(OutputRef),
}

impl From<InputRef> for PortRef {
    fn from(input_ref: InputRef) -> Self {
        Self::Input(input_ref)
    }
}

impl From<OutputRef> for PortRef {
    fn from(output_ref: OutputRef) -> Self {
        Self::Output(output_ref)
    }
}

// Input

#[derive(Debug, Default)]
pub struct Input {
    output: Option<Arc<SyncUnsafeCell<Output>>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputRef(pub (Uuid, usize));

impl InputRef {
    #[must_use]
    pub fn instance_ref(&self) -> InstanceRef {
        InstanceRef(self.0.0)
    }
}

#[derive(Builder, Debug)]
#[builder(derive(Debug), on(String, into))]
pub struct InputDefinition {
    pub name: Option<String>,
    pub description: Option<String>,
}

impl<S> From<InputDefinitionBuilder<S>> for InputDefinition
where
    S: input_definition_builder::IsComplete,
{
    fn from(builder: InputDefinitionBuilder<S>) -> InputDefinition {
        builder.build()
    }
}

// Output

#[derive(Debug, Default)]
pub struct Output {
    input: Option<Arc<SyncUnsafeCell<Input>>>,
    vectors: [Vector; 2],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputRef(pub (Uuid, usize));

impl OutputRef {
    #[must_use]
    pub fn instance_ref(&self) -> InstanceRef {
        InstanceRef(self.0.0)
    }
}

#[derive(Builder, Debug)]
#[builder(derive(Debug), on(String, into))]
pub struct OutputDefinition {
    pub name: Option<String>,
    pub description: Option<String>,
}

impl<S> From<OutputDefinitionBuilder<S>> for OutputDefinition
where
    S: output_definition_builder::IsComplete,
{
    fn from(builder: OutputDefinitionBuilder<S>) -> OutputDefinition {
        builder.build()
    }
}

// -------------------------------------------------------------------------------------------------

// Connected

pub trait GetConnected {
    fn connected(&self) -> bool;
}

impl<P> GetConnected for Port<P>
where
    P: GetConnected,
{
    fn connected(&self) -> bool {
        unsafe { (*self.0.get()).connected() }
    }
}

impl GetConnected for Input {
    fn connected(&self) -> bool {
        self.output.is_some()
    }
}

impl GetConnected for Output {
    fn connected(&self) -> bool {
        self.input.is_some()
    }
}

// -------------------------------------------------------------------------------------------------

// Vectors

pub trait GetInputVector {
    fn input_vector(&self, token: &ProcessToken) -> Option<&Vector>;
}

impl<P> GetInputVector for Port<P>
where
    P: GetInputVector,
{
    fn input_vector(&self, token: &ProcessToken) -> Option<&Vector> {
        unsafe { (*self.0.get()).input_vector(token) }
    }
}

impl GetInputVector for Input {
    fn input_vector(&self, token: &ProcessToken) -> Option<&Vector> {
        self.output
            .as_ref()
            .map(|output| unsafe { (*output.get()).output_vector(token) })
    }
}

pub trait GetOutputVector {
    fn output_vector(&self, token: &ProcessToken) -> &Vector;
}

impl<P> GetOutputVector for Port<P>
where
    P: GetOutputVector,
{
    fn output_vector(&self, token: &ProcessToken) -> &Vector {
        unsafe { (*self.0.get()).output_vector(token) }
    }
}

impl GetOutputVector for Output {
    fn output_vector(&self, token: &ProcessToken) -> &Vector {
        unsafe { self.vectors.get_unchecked(token.0) }
    }
}

pub trait GetOutputVectorPrevious {
    fn output_vector_previous(&self, token: &ProcessToken) -> &Vector;
}

impl<P> GetOutputVectorPrevious for Port<P>
where
    P: GetOutputVectorPrevious,
{
    fn output_vector_previous(&self, token: &ProcessToken) -> &Vector {
        unsafe { (*self.0.get()).output_vector_previous(token) }
    }
}

impl GetOutputVectorPrevious for Output {
    fn output_vector_previous(&self, token: &ProcessToken) -> &Vector {
        unsafe { self.vectors.get_unchecked(usize::from(token.0 == 0)) }
    }
}

pub trait GetOutputVectorMut {
    fn output_vector_mut(&mut self, token: &ProcessToken) -> &mut Vector;
}

impl<P> GetOutputVectorMut for Port<P>
where
    P: GetOutputVectorMut,
{
    fn output_vector_mut(&mut self, token: &ProcessToken) -> &mut Vector {
        unsafe { (*self.0.get()).output_vector_mut(token) }
    }
}

impl GetOutputVectorMut for Output {
    fn output_vector_mut(&mut self, token: &ProcessToken) -> &mut Vector {
        unsafe { self.vectors.get_unchecked_mut(token.0) }
    }
}

// -------------------------------------------------------------------------------------------------

// Connection

pub(crate) trait Connect<P> {
    fn connect(&mut self, other: &mut P);
}

impl Connect<Port<Output>> for Port<Input> {
    fn connect(&mut self, output: &mut Port<Output>) {
        unsafe {
            (*self.0.get()).output = Some(output.0.clone());
        }

        unsafe {
            (*output.0.get()).input = Some(self.0.clone());
        }
    }
}

impl Connect<Port<Input>> for Port<Output> {
    fn connect(&mut self, input: &mut Port<Input>) {
        unsafe {
            (*self.0.get()).input = Some(input.0.clone());
        }

        unsafe {
            (*input.0.get()).output = Some(self.0.clone());
        }
    }
}

// -------------------------------------------------------------------------------------------------

// Disconnection

#[allow(dead_code)]
pub(crate) trait Disconnect {
    fn disconnect(&mut self);
}

impl<P> Disconnect for Port<P>
where
    P: Disconnect,
{
    fn disconnect(&mut self) {
        unsafe {
            (*self.0.get()).disconnect();
        }
    }
}

impl Disconnect for Input {
    fn disconnect(&mut self) {
        if let Some(output) = self.output.as_ref() {
            unsafe {
                (*output.get()).input = None;
            }
        }

        self.output = None;
    }
}

impl Disconnect for Output {
    fn disconnect(&mut self) {
        if let Some(input) = self.input.as_ref() {
            unsafe {
                (*input.get()).output = None;
            }
        }

        self.input = None;
    }
}
