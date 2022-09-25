#![no_std]
#![no_main]

use cortex_m_rt::entry;
use stm32f4xx_hal as hal;
use stm32f4xx_hal::prelude::*;

use core::cell::RefCell;
use cortex_m::interrupt::free as critical;
use cortex_m::interrupt::Mutex;

use cortex_m_semihosting::*;

use hal::block;
use hal::gpio::GpioExt;
use hal::pac;
use hal::rcc::RccExt;
use hal::serial::config::Config;
use hal::spi::Spi;
use hal::time::U32Ext;
use stm32f4xx_hal::pac::{interrupt, Interrupt};

#[allow(unused_imports)]
use panic_semihosting; // When a panic occurs, dump it to openOCD

// Configuring this wrong can get you in trouble with the law,
// check your local lora band frequency before building
const FREQUENCY: i64 = 868;
// This is hardware dependant, don't change this.
const MTU: usize = 255;
// This value can be configured for faster serial but needs to
// also be changed in ratmand's config
const BAUDRATE: u32 = 9600;

struct Datapacket {
    index: usize,
    data: [u8; MTU],
}

impl Datapacket {
    const fn new() -> Self {
        Self {
            index: 0,
            data: [0; MTU],
        }
    }
}

static G_BUFFER: Mutex<RefCell<Datapacket>> = Mutex::new(RefCell::new(Datapacket::new()));
static G_UART_RX: Mutex<RefCell<Option<hal::serial::Rx<hal::pac::USART2>>>> =
    Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.sysclk(64.MHz()).pclk1(32.MHz()).freeze();

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();

    let sck = gpioa.pa5.into_alternate();
    let miso = gpioa.pa6.into_alternate();
    let mosi = gpioa.pa7.into_alternate().internal_pull_up(true);
    let mut reset = gpioc.pc7.into_push_pull_output();
    let mut cs = gpiob.pb6.into_push_pull_output();

    reset.set_high();
    cs.set_high();

    let spi = Spi::new(
        dp.SPI1,
        (sck, miso, mosi),
        sx127x_lora::MODE,
        8.MHz(),
        &clocks,
    );

    let uart_tx = gpioa.pa2.into_alternate();
    let uart_rx = gpioa.pa3.into_alternate();

    let (mut uart_tx, mut uart_rx) = dp
        .USART2
        .serial(
            (uart_tx, uart_rx),
            Config::default().baudrate(BAUDRATE.bps()),
            &clocks,
        )
        .unwrap()
        .split();

    uart_rx.listen();

    critical(move |lock| *G_UART_RX.borrow(lock).borrow_mut() = Some(uart_rx));

    unsafe {
        cortex_m::peripheral::NVIC::unmask(Interrupt::USART2);
    }

    hprintln!("Configuring UART OK");

    let mut lora = match sx127x_lora::LoRa::new(spi, cs, reset, FREQUENCY, cp.SYST.delay(&clocks)) {
        Ok(l) => l,
        Err(e) => panic!("{:?}", e),
    };

    lora.set_tx_power(17, 1);

    hprintln!("Configuring Radio OK");

    loop {
        let mut i = 0;
        critical(|lock| {
            let mut d = G_BUFFER.borrow(lock).borrow_mut();
            i = d.index;
            if d.index >= MTU {
                let transmit = lora.transmit_payload_busy(d.data, d.index);
                match transmit {
                    Ok(_) => hprintln!("tx_c"),
                    Err(e) => hprintln!("tx_err:{:?}", e),
                }
                d.index = 0;
            }
        });

        let poll = lora.poll_irq(Some(1));
        match poll {
            Ok(size) => {
                if size != 255 {
                    hprintln!("s_err {}", size);
                }
                let buffer = lora.read_packet().unwrap(); // Received buffer. NOTE: 255 bytes are always returned
                for i in 0..size {
                    block!(uart_tx.write(buffer[i]));
                }
            }
            Err(_e) => {}
        }
    }
}

#[interrupt]
fn USART2() {
    static mut UART_RX: Option<hal::serial::Rx<hal::pac::USART2>> = None;

    let uart_rx = UART_RX.get_or_insert_with(|| {
        critical(|lock| {
            // Move serial device here, leaving a None in its place
            G_UART_RX.borrow(lock).replace(None).unwrap()
        })
    });

    critical(|lock| {
        let mut d = G_BUFFER.borrow(lock).borrow_mut();
        loop {
            match uart_rx.read() {
                Ok(c) => {
                    let i = d.index;
                    if i == 0 {
                        if c == 202 {
                            d.data[i] = c;
                            d.index += 1;
                        }
                    } else {
                        if i < MTU {
                            d.data[i] = c;
                            d.index += 1;
                        }
                    }
                }
                Err(hal::nb::Error::WouldBlock) => break,
                Err(e) => hprintln!("s_err:{:?}", e),
            }
        }
    });
}
