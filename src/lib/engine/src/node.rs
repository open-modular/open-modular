use crate::{
    module::ModuleDefinition,
    port::{
        self,
        Input,
        Output,
        Port,
    },
};

// =================================================================================================
// Node
// =================================================================================================

#[derive(Debug)]
pub struct Node {
    inputs: Vec<Port<Input>>,
    outputs: Vec<Port<Output>>,
}

impl Node {
    #[must_use]
    pub fn from_definition(definition: &ModuleDefinition) -> Self {
        let inputs = definition.inputs.iter().map(|_| Port::default()).collect();
        let outputs = definition.outputs.iter().map(|_| Port::default()).collect();

        Self { inputs, outputs }
    }
}

// -------------------------------------------------------------------------------------------------

// Connected

pub trait GetConnected {
    fn connected(&self) -> bool;
}

impl GetConnected for Node {
    fn connected(&self) -> bool {
        self.inputs.iter().any(port::GetConnected::connected)
            || self.outputs.iter().any(port::GetConnected::connected)
    }
}

// -------------------------------------------------------------------------------------------------

// Input

pub trait GetInput {
    fn input(&self, input: usize) -> Option<&Port<Input>>;
}

impl<T> GetInput for T
where
    T: AsRef<Node>,
{
    fn input(&self, input: usize) -> Option<&Port<Input>> {
        self.as_ref().inputs.get(input)
    }
}

pub(crate) trait GetInputMut {
    fn input_mut(&mut self, input: usize) -> Option<&mut Port<Input>>;
}

impl<T> GetInputMut for T
where
    T: AsMut<Node>,
{
    fn input_mut(&mut self, input: usize) -> Option<&mut Port<Input>> {
        self.as_mut().inputs.get_mut(input)
    }
}

// -------------------------------------------------------------------------------------------------

// Output

pub trait GetOutput {
    fn output(&self, output: usize) -> Option<&Port<Output>>;
}

impl<T> GetOutput for T
where
    T: AsRef<Node>,
{
    fn output(&self, output: usize) -> Option<&Port<Output>> {
        self.as_ref().outputs.get(output)
    }
}

pub trait GetOutputMut {
    fn output_mut(&mut self, output: usize) -> Option<&mut Port<Output>>;
}

impl<T> GetOutputMut for T
where
    T: AsMut<Node>,
{
    fn output_mut(&mut self, output: usize) -> Option<&mut Port<Output>> {
        self.as_mut().outputs.get_mut(output)
    }
}
