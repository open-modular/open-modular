use bon::Builder;
use uuid::Uuid;

use crate::port::{
    PortInputDefinition,
    PortInputDefinitionBuilder,
    PortOutputDefinition,
    PortOutputDefinitionBuilder,
    Ports,
};

// =================================================================================================
// Module
// =================================================================================================

#[rustfmt::skip]
pub trait Module:
      AsMut<Ports>
    + AsRef<Ports>
    + Process
{
}

// -------------------------------------------------------------------------------------------------

pub trait ModuleSource {
    type Context;

    fn instantiate(id: &Uuid, context: Self::Context) -> Self;
}

// -------------------------------------------------------------------------------------------------

// Definition

pub trait Define {
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

// Identification

pub trait Identify {
    fn id() -> Uuid;
}

// -------------------------------------------------------------------------------------------------

// Instantiation

pub trait Instantiate {
    type Context;

    fn instantiate(node: Ports, runtime: Self::Context) -> Self;
}

// -------------------------------------------------------------------------------------------------

// Processing

pub trait Process {
    fn process(&mut self, args: &ProcessArgs);
}

#[derive(Debug, Default)]
pub struct ProcessArgs {
    pub token: ProcessToken,
}

#[derive(Debug, Default)]
pub struct ProcessToken(pub(crate) usize);

// -------------------------------------------------------------------------------------------------

// Macros

pub use open_modular_engine_macros::{
    module,
    module_enum,
};
