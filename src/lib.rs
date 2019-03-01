#![allow(dead_code)]
#![allow(unused_variables)]

use std::sync::mpsc::{self, *};

pub mod machines;

pub use self::machines::Machine;

/* The Main datatypes for modular_vm */

#[derive(Debug)]
pub enum Query<BASE> {
    LoadRequest(BASE),
    SaveRequest(BASE, BASE),
}

#[derive(Debug)]
pub enum Response<BASE> {
    Data(BASE),
    Good,
    Fail(BASE),
}

pub struct TwoWayChannel<B, T> {
    out: Sender<B>,
    back: Receiver<T>
}

impl<I, O> TwoWayChannel<I, O> {
    pub fn construct() -> (TwoWayChannel<I, O>, TwoWayChannel<O, I>) {
        let (senderout, receiverout) = mpsc::channel();
        let (senderin, receiverin) = mpsc::channel();

        (
            TwoWayChannel {
                out: senderout,
                back: receiverin,
            },

            TwoWayChannel {
                out: senderin,
                back: receiverout,
            }
        )
    }

    pub fn send(&self, data: I) -> Result<(), SendError<I>> {
        self.out.send(data)
    }

    pub fn recv(&self) -> Result<O, RecvError> {
        self.back.recv()
    }

    pub fn try_recv(&self) -> Result<O, TryRecvError> {
        self.back.try_recv()
    }

    pub fn iter(&self) -> Iter<O> {
        self.back.iter()
    }

    pub fn try_iter(&self) -> TryIter<O> {
        self.back.try_iter()
    }

    pub fn query(&self, data: I) -> Result<O, RecvError> {
        self.send(data).unwrap();
        self.recv()
    }
}

pub struct Metadata {
    pub model: String,
}

pub type FrontEnd<BASE> = TwoWayChannel<Query<BASE>, Response<BASE>>;

pub type BackEnd<BASE> = TwoWayChannel<Response<BASE>, Query<BASE>>;

/* Peripherals and Processors for a normal machine */

pub trait Peripheral<BASE> {
    fn handle(&mut self, incoming: Query<BASE>) -> Result<Response<BASE>, BASE>;

    fn metadata(&self) -> Metadata {
        Metadata {
            model: String::from("Peripheral"),
        }
    }

    fn cycle(&mut self) -> Result<(), BASE> {
        Ok(())
    }

    fn boot(&mut self) -> Result<(), BASE> {
        Ok(())
    }

    fn halt(&mut self) -> Result<(), BASE> {
        Ok(())
    }
}

pub trait Processor<BASE> {
    fn exe_ins(&mut self, channels: &Vec<FrontEnd<BASE>>) -> Result<(), BASE>;

    fn metadata(&self) -> Metadata {
        Metadata {
            model: String::from("Processor"),
        }
    }

    fn boot(&mut self, channels: &Vec<FrontEnd<BASE>>) -> Result<(), BASE> {
        Ok(())
    }

    fn halt(&mut self, channels: &Vec<FrontEnd<BASE>>) -> Result<(), BASE> {
        Ok(())
    }
}

/* Processors for a Processor Network machine */

pub trait ProcessorNode<BASE> {
    fn exe_ins(&mut self, channels: &Vec<FrontEnd<BASE>>) -> Result<(), BASE>;

    fn metadata(&self) -> Metadata {
        Metadata {
            model: String::from("ProcessorNode"),
        }
    }

    fn handle(&mut self, incoming: Query<BASE>) -> Result<Response<BASE>, BASE>;

    fn boot(&mut self, channels: &Vec<FrontEnd<BASE>>) -> Result<(), BASE> {
        Ok(())
    }

    fn halt(&mut self, channels: &Vec<FrontEnd<BASE>>) -> Result<(), BASE> {
        Ok(())
    }
}

/* Nodes for a Node Network machine */

pub trait Node<BASE> {
    fn cycle(&mut self, channels: &Vec<FrontEnd<BASE>>) -> Result<(), BASE>;

    fn handle(&mut self, incoming: Query<BASE>) -> Result<Response<BASE>, BASE>;

    fn metadata(&self) -> Metadata {
        Metadata {
            model: String::from("Node"),
        }
    }

    fn boot(&mut self, channels: &Vec<FrontEnd<BASE>>) -> Result<(), BASE> {
        Ok(())
    }

    fn halt(&mut self, channels: &Vec<FrontEnd<BASE>>) -> Result<(), BASE> {
        Ok(())
    }
}