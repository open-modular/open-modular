use bon::Builder;
use uuid::Uuid;

use crate::{
    port::{
        PortInputDefinition,
        PortInputDefinitionBuilder,
        PortInputs,
        PortOutputDefinition,
        PortOutputDefinitionBuilder,
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
