use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

#[derive(Debug)]
enum Message {
    CpuStep,
    ReadByte(u16),
    VdpEnableLineInterrupt,
    CpuEnableInterrupt,
}

#[derive(Debug)]
enum Response {
    ByteRead(u8),
}

struct Machine {
    bus: Bus,
}

impl Machine {
    fn new() -> Self {
        let (request_sender, request_receiver) = channel();
        let (response_sender, response_receiver) = channel();
        let response_receiver = Arc::new(Mutex::new(response_receiver));

        let cpu = Cpu::new(request_sender.clone(), response_receiver.clone());
        let vdp = Vdp::new(request_sender.clone());
        let ppi = Ppi {};
        let bus = Bus::new(
            cpu,
            vdp,
            ppi,
            request_sender,
            response_receiver,
            request_receiver,
            response_sender,
        );
        Self { bus }
    }

    fn step(&mut self) {
        println!("Machine step");
        self.bus.step();
    }
}

struct Bus {
    cpu: Cpu,
    vdp: Vdp,
    ppi: Ppi,
    request_sender: Sender<Message>,
    response_receiver: Arc<Mutex<Receiver<Response>>>,
    request_receiver: Receiver<Message>,
    response_sender: Sender<Response>,
}

impl Bus {
    fn new(
        cpu: Cpu,
        vdp: Vdp,
        ppi: Ppi,
        request_sender: Sender<Message>,
        response_receiver: Arc<Mutex<Receiver<Response>>>,
        request_receiver: Receiver<Message>,
        response_sender: Sender<Response>,
    ) -> Self {
        Self {
            cpu,
            vdp,
            ppi,
            request_sender,
            response_receiver,
            request_receiver,
            response_sender,
        }
    }

    fn step(&mut self) {
        println!("Bus step");
        self.request_sender.send(Message::CpuStep).unwrap();

        loop {
            if let Ok(message) = self.request_receiver.recv() {
                println!("Bus recv: {:?}", message);
                match message {
                    Message::CpuStep => {
                        self.cpu.step();
                    }
                    Message::ReadByte(addr) => {
                        println!("Bus read_byte {:x}", addr);
                        let data = self.ppi.read_byte(addr);
                        self.response_sender.send(Response::ByteRead(data)).unwrap();
                    }
                    Message::VdpEnableLineInterrupt => {
                        self.vdp.enable_line_interrupt();
                    }
                    Message::CpuEnableInterrupt => {
                        self.cpu.enable_interrupt();
                    }
                };
            }
        }
    }
}

struct Cpu {
    ext_cpu: ExtCpu<Io>,
}

impl Cpu {
    fn new(
        request_sender: Sender<Message>,
        response_receiver: Arc<Mutex<Receiver<Response>>>,
    ) -> Self {
        let io = Io::new(request_sender, response_receiver);
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
    request_sender: Sender<Message>,
}

impl Vdp {
    fn new(request_sender: Sender<Message>) -> Self {
        Self { request_sender }
    }

    fn enable_line_interrupt(&self) {
        println!("Vdp enable_line_interrupt");
        self.request_sender
            .send(Message::CpuEnableInterrupt)
            .unwrap();
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
    request_sender: Sender<Message>,
    response_receiver: Arc<Mutex<Receiver<Response>>>,
}

impl Io {
    fn new(
        request_sender: Sender<Message>,
        response_receiver: Arc<Mutex<Receiver<Response>>>,
    ) -> Self {
        Self {
            request_sender,
            response_receiver,
        }
    }
}

impl ExtCpuIo for Io {
    fn read(&self, addr: u16) -> u8 {
        let sender = self.request_sender.clone();
        println!("ExtCpuIo read: {:x}", addr);
        sender.send(Message::ReadByte(addr)).unwrap();

        let response_receiver = self.response_receiver.lock().unwrap();
        let response = response_receiver.recv().unwrap();
        println!("ExtCpuIo read response: {:?}", response);
        match response {
            Response::ByteRead(data) => data,
        }
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
        self.request_sender
            .send(Message::VdpEnableLineInterrupt)
            .unwrap();
    }
}

fn main() {
    let mut machine = Machine::new();
    machine.step();
}
