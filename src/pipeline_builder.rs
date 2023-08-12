use crate::client::Archon;

use eyre::Result;
use std::{
    pin::Pin,
    sync::mpsc::{
        channel,
        Receiver,
    },
};

/// Builder for [Archon] Pipeline
///
/// Accepts actors with impl trait [Stage] and builds the stages.
#[derive(Debug)]
pub struct PipelineBuilder<'a, T: Stage = ()> {
    /// Archon Pipeline
    pipeline: &'a mut Archon,
    receiver: Option<Receiver<Pin<Box<T::Output>>>>,
}

impl<'a, T: Stage> PipelineBuilder<'a, T> {
    /// Constructs a new PipelineBuilder with an [Archon] Instance.
    pub fn new(pipeline: &mut Archon) -> PipelineBuilder<()> {
        PipelineBuilder {
            pipeline,
            receiver: Default::default(),
        }
    }

    /// Returns the Archon receiver from the build stages
    pub fn build(self) -> Receiver<Pin<Box<T::Output>>> {
        self.receiver.unwrap()
    }

    /// Builds an actor stage returning the receiver
    pub fn channel<S: Stage>(self, mut stage: S) -> PipelineBuilder<'a, S> {
        let (_, receiver) = channel::<Pin<Box<S::Input>>>();

        // Remove unwrap? breaks the .channel() chain to
        let receiver = stage.build(self.pipeline, Some(receiver)).unwrap();

        PipelineBuilder {
            pipeline: self.pipeline,
            receiver: Some(receiver),
        }
    }
}

/// Stage trait for building [Archon] Pipeline
pub trait Stage {
    /// Input receiver channel
    type Input;

    /// Ouptu receiver channel
    type Output;

    /// Builds actor stage and return receiver channel
    fn build(
        &mut self,
        pipeline: &mut Archon,
        recevier: Option<Receiver<Pin<Box<Self::Input>>>>,
    ) -> Result<Receiver<Pin<Box<Self::Output>>>>;
}

/// Stage Impl for ()
impl Stage for () {
    type Input = ();
    type Output = ();

    fn build(
        &mut self,
        _pipeline: &mut Archon,
        _recevier: Option<Receiver<Pin<Box<Self::Input>>>>,
    ) -> Result<Receiver<Pin<Box<Self::Output>>>> {
        let (_, receiver) = channel::<Pin<Box<Self::Input>>>();

        Ok(receiver)
    }
}
