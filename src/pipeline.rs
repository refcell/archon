use std::{
    pin::Pin,
    sync::mpsc::Receiver,
};

#[derive(Debug, Default)]
struct PipelineBuilder<T: Stage> {
    // stages: Vec<impl Stage>
    reciever: Option<Receiver<Pin<Box<T::Output>>>>,
}

impl<T: Stage> PipelineBuilder<T> {
    fn build(self, mut stage: T) -> Self {
        let reciever = stage.build();

        Self { reciever }
    }
}

/// Stage trait for building Archon Pipeline
pub trait Stage {
    /// Stage build output
    type Output;

    /// Build stage and return receiver channel
    fn build(&mut self) -> Option<Receiver<Pin<Box<Self::Output>>>>;
}
