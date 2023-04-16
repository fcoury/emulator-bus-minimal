use std::{cell::RefCell, rc::Rc};

enum Message {
    CpuStep,
    ReadByte(u16),
    ReadByteResponse(u8),
    VdpEnableLineInterrupt,
    CpuEnableInterrupt,
}

struct Machine {
    bus: Bus,
}

impl Machine {
    fn new() -> Self {
        let queue = Rc::new(RefCell::new(Vec::new()));
        let cpu = Cpu::new(queue.clone());
        let vdp = Vdp::new(queue.clone());
        let ppi = Ppi {};
        let bus = Bus::new(cpu, vdp, ppi, queue);
        Self { bus }
    }

    fn step(&mut self) {
        self.bus.step();
    }
}

struct Bus {
    cpu: Cpu,
    vdp: Vdp,
    ppi: Ppi,
    queue: Rc<RefCell<Vec<Message>>>,
}

impl Bus {
    fn new(cpu: Cpu, vdp: Vdp, ppi: Ppi, queue: Rc<RefCell<Vec<Message>>>) -> Self {
        Self {
            cpu,
            vdp,
            ppi,
            queue,
        }
    }

    fn step(&mut self) {
        println!("Bus step");
        self.queue.borrow_mut().push(Message::CpuStep);

        loop {
            let Some(message) = self.queue.borrow_mut().pop() else {
                break;
            };

            match message {
                Message::CpuStep => {
                    self.cpu.step();
                }
                Message::ReadByte(addr) => {
                    let data = self.ppi.read_byte(addr);
                    self.queue
                        .borrow_mut()
                        .push(Message::ReadByteResponse(data));
                }
                Message::ReadByteResponse(data) => {
                    // How do I get the data to the External CPU?
                }
                Message::VdpEnableLineInterrupt => {
                    self.vdp.enable_line_interrupt();
                }
                Message::CpuEnableInterrupt => {
                    self.cpu.enable_interrupt();
                }
            }
        }
    }
}

struct Cpu {
    ext_cpu: ExtCpu<Io>,
}

impl Cpu {
    fn new(queue: Rc<RefCell<Vec<Message>>>) -> Self {
        let io = Io::new(queue);
        let ext_cpu = ExtCpu { io };
        Self { ext_cpu }
    }

    fn step(&mut self) {
        println!("Cpu step");
        self.ext_cpu.step();
    }

    fn enable_interrupt(&self) {
        println!("Cpu enable_interrupt");
        self.ext_cpu.enable_interrupt();
    }
}

struct Ppi;

impl Ppi {
    fn read_byte(&self, addr: u16) -> u8 {
        println!("Ppi read_byte {:x}", addr);
        0xfe
    }
}

struct Vdp {
    queue: Rc<RefCell<Vec<Message>>>,
}

impl Vdp {
    fn new(queue: Rc<RefCell<Vec<Message>>>) -> Self {
        Self { queue }
    }

    fn enable_line_interrupt(&self) {
        println!("Vdp enable_line_interrupt");
        self.queue.borrow_mut().push(Message::CpuEnableInterrupt);
    }
}

trait ExtCpuIo {
    fn read(&self, addr: u16) -> u8;
    fn write(&self, addr: u16, data: u8);
    fn read_port(&self, port: u8) -> u8;
    fn write_port(&self, port: u8, data: u8);
}

struct ExtCpu<ExtCpuIo> {
    io: ExtCpuIo,
}

impl<T: ExtCpuIo> ExtCpu<T> {
    fn step(&mut self) {
        println!("ExtCpu step");
        if self.io.read(0) == 0 {
            self.io.write_port(0, 1);
        }
    }

    fn enable_interrupt(&self) {
        println!("ExtCpu enable_interrupt");
        self.io.write_port(1, 1);
    }
}

struct Io {
    queue: Rc<RefCell<Vec<Message>>>,
}

impl Io {
    fn new(queue: Rc<RefCell<Vec<Message>>>) -> Self {
        Self { queue }
    }
}

impl ExtCpuIo for Io {
    fn read(&self, addr: u16) -> u8 {
        println!("ExtCpuIo read: {:x}", addr);
        self.queue.borrow_mut().push(Message::ReadByte(0x0042));
        todo!("How do I get the u8 to respond the External CPU?")
    }

    fn write(&self, addr: u16, data: u8) {
        println!("write: {:x} {:x}", addr, data);
    }

    fn read_port(&self, port: u8) -> u8 {
        println!("ExtCpuIo read_port: {:x}", port);
        todo!()
    }

    fn write_port(&self, port: u8, data: u8) {
        println!("ExtCpuIo write_port: {:x} {:x}", port, data);
        if port == 0x00 {
            self.queue
                .borrow_mut()
                .push(Message::VdpEnableLineInterrupt);
        }
    }
}

fn main() {
    let mut machine = Machine::new();
    machine.step();
}
