use crate::{
    client::Archon,
    config::Config,
};
use eyre::Result;
use std::{
    pin::Pin,
    sync::mpsc::Receiver,
};

#[derive(Debug, Default)]
struct PipelineBuilder<T: Stage> {
    input_reciever: Option<Receiver<Pin<Box<T::Input>>>>,

    // stages: Vec<impl Stage>
    output_reciever: Option<Receiver<Pin<Box<T::Output>>>>,
    /// The inner [Config], used to configure [Archon]'s parameters
    // config: Config,
    pipeline: Archon,
}

impl<T: Stage> PipelineBuilder<T> {
    pub fn input_receiver(&mut self) -> Option<Receiver<Pin<Box<T::Input>>>> {
        self.input_reciever.take()
    }

    pub fn output_receiver(self) -> Option<Receiver<Pin<Box<T::Output>>>> {
        self.output_reciever
    }
}

impl<T: Stage> PipelineBuilder<T> {
    fn build(&mut self, mut stage: T) -> Result<&mut Self> {
        let input_receiver = self.input_receiver();
        let reciever = stage.build(&mut self.pipeline, input_receiver)?;

        self.output_reciever = reciever;

        Ok(self)
    }
}

/// Stage trait for building Archon Pipeline
pub trait Stage {
    /// Stage build Input
    type Input;
    /// Stage build output
    type Output;

    /// Build stage and return receiver channel
    fn build(
        &mut self,
        pipeline: &mut Archon,
        reciever: Option<Receiver<Pin<Box<Self::Input>>>>,
    ) -> Result<Option<Receiver<Pin<Box<Self::Output>>>>>;
}
