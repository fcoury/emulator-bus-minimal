## Modeling Bus and its components in Rust emulator

[Rust Forum Link](https://users.rust-lang.org/t/modeling-bus-and-its-components-in-rust-emulator/92583)

I am writing an emulator and am having a hard time modeling a solution for this problem. Below is what I have so far. I am sure there's a better way to model this problem, and that's exactly what I am looking for.

I am trying to model the following scenario:

Structs:

- Machine: is the high level abstraction of the hardware, talks via the BUS
- BUS: the orchestrator, talks to the CPU via stepping to next instruction and to the VDP via writing to a given port
- CPU: the CPU, talks to memory via the BUS (not represented here) and to the VDP via the BUS by writing to a given I/O port
- VDP: the graphics chip with VRAM and registers, needs to talk to the CPU to enable/disable interrupts

Flow:

- Machine asks CPU to step to next instruction via the BUS
- CPU reads a given memory address and writes to a given VDP port via the BUS
- VDP receives the write and enables interrupts on CPU via the BUS

Problem:

`thread 'main' panicked at 'already borrowed: BorrowMutError', src/main.rs:61:18`

```rust
    fn step(&mut self) {
        println!("CPU step");
        self.bus.borrow_mut().write_port(); // <- here
    }
```

What I tried so far:

- Weak BUS references to CPU/VDP, returning the concrete implementation to the Machine, but the strong reference ended up being dropped
- Returning and keeping the Rc<RefCell<Bus>> to the Machine, and we follow on this reentrant problem
- ChatGPT-4 which suggested extracting traits:

```rust
trait BusTrait {
    fn write_port(&mut self);
    fn enable_interrupt(&mut self);
}

trait CpuTrait {
    fn step(&mut self);
    fn enable_interrupt(&mut self);
}

trait VdpTrait {
    fn write_port(&mut self);
}
```

Is this the best way to go about this problem?

Minimal replication code:

https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=df7b9eb0c96a9c50e4d94f2049f801ad

```rust
use std::{cell::RefCell, rc::Rc};

struct Machine {
    bus: Rc<RefCell<Bus>>,
}

impl Machine {
    fn new() -> Self {
        let bus = Rc::new(RefCell::new(Bus::default()));
        let cpu = Cpu::new(bus.clone());
        let vdp = Vdp::new(bus.clone());

        bus.borrow_mut().cpu = Some(cpu);
        bus.borrow_mut().vdp = Some(vdp);

        Self { bus }
    }

    fn step(&self) {
        self.bus.borrow_mut().step();
    }
}

#[derive(Default)]
struct Bus {
    cpu: Option<Cpu>,
    vdp: Option<Vdp>,
}

impl Bus {
    fn step(&mut self) {
        if let Some(cpu) = &mut self.cpu {
            cpu.step();
        }
    }

    fn write_port(&mut self) {
        if let Some(vdp) = &mut self.vdp {
            vdp.write_port();
        }
    }

    fn enable_interrupt(&mut self) {
        if let Some(cpu) = &mut self.cpu {
            cpu.enable_interrupt();
        }
    }
}

struct Cpu {
    bus: Rc<RefCell<Bus>>,
}

impl Cpu {
    fn new(bus: Rc<RefCell<Bus>>) -> Self {
        Self { bus }
    }

    fn step(&mut self) {
        println!("CPU step");
        self.bus.borrow_mut().write_port();
    }

    fn enable_interrupt(&mut self) {
        println!("CPU interrupt");
    }
}

struct Vdp {
    bus: Rc<RefCell<Bus>>,
}

impl Vdp {
    fn new(bus: Rc<RefCell<Bus>>) -> Self {
        Self { bus }
    }

    fn write_port(&mut self) {
        println!("VDP enable IRQ");
        self.bus.borrow_mut().enable_interrupt();
    }
}

fn main() {
    let machine = Machine::new();
    machine.step();
}
```
