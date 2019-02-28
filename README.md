# ModVM-Rust
ModVM provides a simple framework for building Virtual Machines in Rust. VMs in ModVM are modular, and are to this end made up of two different types of components: Processors and Peripherals.

## Execution Model
The VM is initialized with a vector of Processors and a vector of Peripherals, and automatically constructs the communication channels between each of the processors and the peripherals. The processor then makes requests down these channels in its `exe_ins()` function, which is called in a loop. When any peripheral receives data down a channel, its `handle(request)` function is called, and this returns data for the processor to use.

After initialization, the VM returns a `MachineHandle` struct, which can be used to join to the threads of the VM.

## Useage
To initalize a VM, use its `from` constructor, where processor and peripheral implement the `Processor` and `Peripheral` traits respectively:
```rust
let machine = modVM::Machine::from(vec![Box::new(peripheral)], vec![Box::new(processor)]);

machine.run().unwrap().join_processors();
```
