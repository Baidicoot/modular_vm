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
The `Processor<BASE>` trait requires the `exe_ins()` function to be implemented on the struct. This function must take in a Vector of `FrontEnd` and `&mut self`, and return `Result<(), BASE>`.

A simple implementation would be:
```rust
impl Processor<u8> for ProcessorEightBit {
    fn exe_ins(&mut self, channels: &Vec<FrontEnd<u16>>) {
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