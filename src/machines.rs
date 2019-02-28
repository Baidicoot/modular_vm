use crate::*;
use std::thread;
use std::thread::JoinHandle;

/* A collection of functions for running machines */

fn pairs<A, B>(input: Vec<Vec<(A, B)>>) -> Result<(Vec<Vec<A>>, Vec<Vec<B>>), &'static str> {
    let len = input.len();

    let head_len = match input.last() {
        Some(x) => x.len(),
        None => return Err("Cannot make pairs for a zero length object!"),
    };

    if head_len == 0 {
        return Err("Cannot make pairs for a zero length object!");
    }

    let mut outa = vec![];
    let mut outb = vec![];

    for _ in 0..len {
        outa.push(vec![]);
    }

    for _ in 0..head_len {
        outb.push(vec![]);
    }

    for (r, row) in input.into_iter().enumerate() {
        for (i, (a, b)) in row.into_iter().enumerate() {
            outa[r].push(a);
            outb[i].push(b);
        }
    }

    Ok((outa, outb))
}

pub struct Machine<BASE>
    where BASE: 'static + Send,
{
    peripherals: Vec<Box<dyn Peripheral<BASE> + Send>>,
    processors: Vec<Box<dyn Processor<BASE> + Send>>,
}

impl<BASE> Machine<BASE>
    where BASE: std::fmt::Display + Send,

{
    pub fn from(peripherals: Vec<Box<dyn Peripheral<BASE> + Send>>, processors: Vec<Box<dyn Processor<BASE> + Send>>) -> Machine<BASE> {
        Machine {
            peripherals,
            processors,
        }
    }

    pub fn run(self) -> Result<MachineHandle, &'static str> {
        let mut channels = vec![];

        for _ in 0..self.peripherals.len() {
            let mut group = vec![];
            for _ in 0..self.processors.len() {
                group.push(TwoWayChannel::construct());
            }
            channels.push(group);
        }

        let (backend, frontend) = pairs(channels)?;

        let mut peripheral_handles = vec![];

        for (no, (mut component, channels)) in self.peripherals.into_iter().zip(backend.into_iter()).enumerate() {
            peripheral_handles.push(thread::Builder::new()
                .name(format!("Peripheral Number {}: {}", no, component.metadata().model))
                .spawn(move || {
                    loop {
                        for channel in channels.iter() {
                            if let Ok(query) = channel.try_recv() {
                                channel.send(component.handle(query)).unwrap();
                            } else {
                                continue
                            }
                        }
                    }
                })
                .unwrap()
                );
        }

        let mut processor_handles = vec![];

        for (no, (mut component, channels)) in self.processors.into_iter().zip(frontend.into_iter()).enumerate() {
            processor_handles.push(thread::Builder::new()
                .name(format!("Processor Number {}: {}", no, component.metadata().model))
                .spawn(move || {
                    loop {
                        match component.exe_ins(&channels) {
                            Ok(()) => {},
                            Err(x) => {
                                println!("`{}` exited with code: {}", thread::current().name().unwrap(), x);
                                break
                            }
                        }
                    }
                })
                .unwrap()
                );
        }

        Ok(MachineHandle {
            peripheral_handles,
            processor_handles,
        })
    }
}

pub struct MachineHandle {
    peripheral_handles: Vec<JoinHandle<()>>,
    processor_handles: Vec<JoinHandle<()>>,
}

impl MachineHandle {
    pub fn join(self) {
        for handle in self.processor_handles.into_iter() {
            handle.join().unwrap();
        }
        for handle in self.peripheral_handles.into_iter() {
            handle.join().unwrap();
        }
    }

    pub fn join_processors(self) {
        for handle in self.processor_handles.into_iter() {
            handle.join().unwrap();
        }
    }

    pub fn join_peripherals(self) {
        for handle in self.peripheral_handles.into_iter() {
            handle.join().unwrap();
        }
    }
}