#![feature(duration_as_u128)]
extern crate modVM;

use modVM::*;
use modVM::machines::Machine;
use self::Query::*;
use self::Response::*;

use std::num::Wrapping;

use std::time::Instant;

#[derive(Clone)]
struct BrainF {
    pointer: u16,
    counter: u16,
    stack: Vec<u16>,
    input: Vec<u16>,
    cycles: u128,
    requests: u128,
}

impl Processor<u16> for BrainF {
    fn metadata(&self) -> Metadata {
        Metadata {
            model: String::from("BrainF Processor v.1.0.0"),
        }
    }

    fn exe_ins(&mut self, channels: &Vec<FrontEnd<u16>>) -> Result<(), u16> {
        let ins = channels[0].query(LoadRequest(self.counter)).unwrap();

        self.cycles += 1;
        self.requests += 1;

        fn change_by(channel: &FrontEnd<u16>, index: u16, value: i16) -> Result<(), u16> {
            let current = channel.query(LoadRequest(index)).unwrap();

            match current {
                Data(x) => {
                    channel.query(SaveRequest(index, (Wrapping(x as i16) + Wrapping(value)).0 as u16)).unwrap();
                    Ok(())
                },
                Good => Ok(()),
                Fail(_) => Err(0),
            }
        }

        fn inc(x: u16) -> u16 {
            (Wrapping(x) + Wrapping(1)).0
        }

        fn dec(x: u16) -> u16 {
            (Wrapping(x) - Wrapping(1)).0
        }

        let ins = match ins {
            Data(x) => x,
            Good => return Err(1),
            Fail(_) => return Err(0),
        };

        self.counter = inc(self.counter);
        
        match ins {
            0 => {
                self.pointer = inc(self.pointer);
                Ok(())
            },
            1 => {
                self.pointer = dec(self.pointer);
                Ok(())
            },
            2 => {
                self.requests += 2;
                change_by(&channels[1], self.pointer, 1)
            },
            3 => {
                self.requests += 2;
                change_by(&channels[1], self.pointer, -1)
            },
            4 => {
                let data = channels[1].query(LoadRequest(self.pointer)).unwrap();

                self.requests += 1;

                match data {
                    Data(x) => {
                        print!("{}", x as u8 as char);
                        Ok(())
                    },
                    Good => Err(1),
                    Fail(_) => Err(0),
                }
            },
            5 => {
                let data = self.input.pop();

                match data {
                    Some(x) => {
                        channels[1].query(SaveRequest(self.pointer, x)).unwrap();
                        self.requests += 1;
                        Ok(())
                    }
                    None => Err(2),
                }
            },
            6 => {
                let data = channels[1].query(LoadRequest(self.pointer)).unwrap();

                self.requests += 1;

                let value = match data {
                    Data(x) => x,
                    Good => return Err(1),
                    Fail(_) => return Err(0),
                };

                if value != 0 {
                    self.stack.push(dec(self.counter));
                    Ok(())
                } else {
                    let mut lev = 1;
                    while lev != 0 {
                        self.requests += 1;
                        match channels[0].query(LoadRequest(self.counter)).unwrap() {
                            Data(x) => {
                                match x {
                                    6 => lev = inc(lev),
                                    7 => lev = dec(lev),
                                    _ => {},
                                }
                            }
                            Good => return Err(1),
                            Fail(_) => return Err(0),
                        };
                        self.counter = inc(self.counter);
                    }
                    Ok(())
                }
            },
            7 => {
                let data = channels[1].query(LoadRequest(self.pointer)).unwrap();

                self.requests += 1;

                match data {
                    Data(x) => {
                        if x == 0 {
                            Ok(())
                        } else {
                            self.counter = match self.stack.pop() {
                                Some(x) => x,
                                None => return Err(3),
                            };
                            Ok(())
                        }
                    },
                    Good => Err(1),
                    Fail(_) => Err(0),
                }
            },
            8 => {
                println!("===\nFinished program\n{} cycles {} requests", self.cycles, self.requests);
                Err(8)
            }
            _ => Ok(()),
        }
    }
}

struct MemT1 {
    mem: Box<[u16; 65536]>,
}

impl Peripheral<u16> for MemT1 {
    fn metadata(&self) -> Metadata {
        Metadata {
            model: String::from("Tier 1 Memory v.1.0.0"),
        }
    }

    fn handle(&mut self, incoming: Query<u16>) -> Result<Response<u16>, u16> {
        Ok(match incoming {
            LoadRequest(x) => {
                match self.mem.get(x as usize) {
                    Some(y) => {
                        Data(*y)
                    },
                    None => Fail(0),
                }
            },
            SaveRequest(x, y) => {
                match self.mem.get_mut(x as usize) {
                    Some(z) => {
                        *z = y;
                        Good
                    },
                    None => Fail(0),
                }
            },
        })
    }
}

impl Clone for MemT1 {
    fn clone(&self) -> MemT1 {
        MemT1 {
            mem: self.mem.clone(),
        }
    }
}

fn parse_bfc(v: &mut Box<[u16; 65536]>, s: &str) {
    let mut index = 0;
    for i in s.chars() {
        match i {
            '>' => v[index] = 0,
            '<' => v[index] = 1,
            '+' => v[index] = 2,
            '-' => v[index] = 3,
            '.' => v[index] = 4,
            ',' => v[index] = 5,
            '[' => v[index] = 6,
            ']' => v[index] = 7,
            '~' => v[index] = 8,
            _ => continue,
        }
        index += 1;
    }
}

const VERSION: &str = "1.1.0";

fn main() {
    let memory = MemT1 {
        mem: Box::new([0; 65536]),
    };

    let mut rom = MemT1 {
        mem: Box::new([0; 65536]),
    };

    parse_bfc(&mut rom.mem,
        r#"
Calculate the value 256 and test if it's zero
If the interpreter errors on overflow this is where it'll happen
++++++++[>++++++++<-]>[<++++>-]
+<[>-<
    Not zero so multiply by 256 again to get 65536
    [>++++<-]>[<++++++++>-]<[>++++++++<-]
    +>[>
        # Print "32"
        ++++++++++[>+++++<-]>+.-.[-]<
    <[-]<->] <[>>
        # Print "16"
        +++++++[>+++++++<-]>.+++++.[-]<
<<-]] >[>
    # Print "8"
    ++++++++[>+++++++<-]>.[-]<
<-]<
# Print " bit cells\n"
+++++++++++[>+++>+++++++++>+++++++++>+<<<<-]>-.>-.+++++++.+++++++++++.<.
>>.++.+++++++..<-.>>-
Clean up used cells.
[[-]<]
~"#
    );

    let mut processor = BrainF {
        pointer: 0,
        counter: 0,
        input: vec![],
        stack: vec![],
        cycles: 0,
        requests: 0,
    };

    processor.input.push(13);
    processor.input.push(7);

    println!("\nBFVM/v{}\nOutput:", VERSION);

    let machine = Machine::from(vec![Box::new(rom), Box::new(memory)], vec![Box::new(processor)]);

    let start = Instant::now();

    machine.run().unwrap().join_processors();

    println!("VM ran for {}ms", start.elapsed().as_millis());
}
