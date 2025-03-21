use bon::Builder;
use fancy_constructor::new;
use uuid::Uuid;

use crate::{
    port::{
        PortInputDefinition,
        PortInputDefinitionBuilder,
        PortInputReference,
        PortInputs,
        PortOutputDefinition,
        PortOutputDefinitionBuilder,
        PortOutputReference,
        PortOutputs,
    },
    processor::Process,
};

// =================================================================================================
// Module
// =================================================================================================

#[rustfmt::skip]
pub trait Module:
      AsMut<PortOutputs>
    + AsRef<PortInputs>
    + Process
{
}

// -------------------------------------------------------------------------------------------------

// Define

pub trait ModuleDefine {
    fn define(module: ModuleDefinitionBuilder) -> impl Into<ModuleDefinition>;
}

#[derive(Builder, Debug)]
#[builder(derive(Debug), on(String, into))]
pub struct ModuleDefinition {
    #[builder(field)]
    pub inputs: Vec<PortInputDefinition>,
    #[builder(field)]
    pub outputs: Vec<PortOutputDefinition>,
    pub name: String,
    pub description: Option<String>,
    pub usage: Option<String>,
}

impl<S> ModuleDefinitionBuilder<S>
where
    S: module_definition_builder::State,
{
    pub fn with_input<F, I>(mut self, input: F) -> ModuleDefinitionBuilder<S>
    where
        F: FnOnce(PortInputDefinitionBuilder) -> I,
        I: Into<PortInputDefinition>,
    {
        let builder = PortInputDefinition::builder();
        let definition = input(builder).into();

        self.inputs.push(definition);
        self
    }

    pub fn with_output<F, I>(mut self, output: F) -> ModuleDefinitionBuilder<S>
    where
        F: FnOnce(PortOutputDefinitionBuilder) -> I,
        I: Into<PortOutputDefinition>,
    {
        let builder = PortOutputDefinition::builder();
        let definition = output(builder).into();

        self.outputs.push(definition);
        self
    }
}

impl<S> From<ModuleDefinitionBuilder<S>> for ModuleDefinition
where
    S: module_definition_builder::IsComplete,
{
    fn from(builder: ModuleDefinitionBuilder<S>) -> ModuleDefinition {
        builder.build()
    }
}

// -------------------------------------------------------------------------------------------------

// Identify

pub trait ModuleIdentify {
    fn id() -> Uuid;
}

// -------------------------------------------------------------------------------------------------

// Instance Reference

#[derive(new, Clone, Debug, Eq, PartialEq)]
pub struct ModuleInstanceReference {
    pub instance: Uuid,
}

impl ModuleInstanceReference {
    #[must_use]
    pub fn input_ref(&self, input: usize) -> PortInputReference {
        PortInputReference::new(self.instance, input)
    }

    #[must_use]
    pub fn output_ref(&self, output: usize) -> PortOutputReference {
        PortOutputReference::new(self.instance, output)
    }
}

// -------------------------------------------------------------------------------------------------

// Instantiate

pub trait ModuleInstantiate {
    type Context;

    fn instantiate(
        context: Self::Context,
        port_inputs: PortInputs,
        port_outputs: PortOutputs,
    ) -> Self;
}

// -------------------------------------------------------------------------------------------------

// Source

pub trait ModuleSource {
    type Context;

    fn get(id: &Uuid, context: Self::Context) -> Self;
}

// -------------------------------------------------------------------------------------------------

// Macros

pub use open_modular_engine_macros::{
    module,
    module_enum,
};
