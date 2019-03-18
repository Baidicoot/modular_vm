# ModVM-Rust
ModVM provides a simple framework for building Virtual Machines in Rust. VMs in ModVM are modular, and are to this end made up of two different types of components: Processors and Peripherals.

## Execution Model

The VM is initialized with a vector of Processors and a vector of Peripherals, and automatically constructs the communication channels between each of the processors and the peripherals. The processor then makes requests down these channels in its `exe_ins()` function, which is called in a loop. When any peripheral receives data down a channel, its `handle(request)` function is called, and this returns data for the processor to use.

After initialization, the VM returns a `MachineHandle` struct, which can be used to join to the threads of the VM.

## Useage
### The TwoWayChannel Struct

The `modVM` crate implements the `TwoWayChannel<I, O>` struct for easy communication between VM components.

`TwoWayChannel` is always instantiated in pairs, and each pair ('end') contains two channels for communicating in both directions down the channel. It accepts the following methods:

+ `send` - sends data down the channel.
+ `recv` - waits for next item, then returns it.
+ `try_recv` - tests to find an item in the channel's queue; if it finds it, it returns it.
+ `iter` - looped version of `recv`.
+ `try_iter` - looped version of `try_recv`.
+ `query` - sends data down the channel, then waits until it gets a reply, then returns it.

Internally, `Processor`s make requests with type `modVM::Query` and `Peripheral`s respond with `modVM::Response`.

### The Processor Trait

The `Processor<BASE>` trait requires the `exe_ins()` function to be implemented on the struct. This function must take in a Vector of `FrontEnd<BASE>` (a wrapper for `TwoWayChannel<Query<BASE>>`) and `&mut self`, and return `Result<(), BASE>`. The processor terminates if any of its functions return an `Err`.

A simple implementation would be:

```rust
impl Processor<u8> for ProcessorEightBit {
    fn exe_ins(&mut self, channels: &Vec<FrontEnd<u16>>) -> Result<(), u8> {
        // get instruction:
        let ins = channels[0].query(LoadRequest(self.counter)).unwrap();

        match ins {
            
            /* snip */

            // instruction didn't match:
            _ => Err(0),
        }
    }
}
```
### The Peripheral Trait

The `Peripheral<BASE>` trait works in a similar fashion to a webserver: its primary purpose is to send and receive requests for data. This is achived through the `handle()` function, which takes in a `Query` enum request and outputs a `Result<Response<BASE>, BASE>`, which, if `Ok` is unwrapped and sent back down the channel. This versatility gives ModVM amazing scope for quickly creating different components for VMs. The `handle()` function is called every time the peripheral receives a request.

A simple implementation would be:
```rust
impl Peripheral<u8> for MemEightBit {
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
```

However `Peripheral`s are not only limited to memory. For example, you could configure one to act as a GPU, or handle I/O. To aid the user in this regard, the optional `cycle` function can be used to run processes that need to be active each cycle. This function can only mutate the component's state, however.

### Shared Functions

All the traits listed above share certain functions: `metadata`, `boot` and `halt`. Metada`metadata` specifies data about the VM, and returns a `Metadata` struct. This is used for identification of threads. `boot` and `halt` are fairly self-explanitory; They run on startup and after termination, and in a `Processor` the functions have access to the channels, so they can make requests for data. However, of these, only the `metadata` function is required for compilation.

Here is an implementation for all these functions on a `Processor`:
```rust
impl Processor<u8> for ProcessorEightBit {
    fn boot(&mut self, channels: &Vec<FrontEnd<u8>>) -> Result<(), BASE> {
        channels[0].query(SaveRequest(PRINT, self.bootcode)).unwrap(); // print the bootcode
        Ok(())
    }

    fn halt(&mut self, channels: &Vec<FrontEnd<u8>>) -> Result<(), BASE> {
        channels[0].query(SaveRequest(PRINT, self.error)).unwrap(); // print the error message

        for i in channels {
            i.send(LoadRequest(HALT)); // halt all peripherals
        }
    }
}
```

And for a peripheral:
```rust
impl Peripheral<u8> for MemEightBit {
    fn boot(&mut self) -> Result<(), BASE> {
        /* initialise things */
        Ok(())
    }

    fn halt(&mut self) -> Result<(), BASE> {
        /* halt processes */
        Ok(())
    }
}
```