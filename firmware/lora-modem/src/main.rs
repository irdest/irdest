#![no_std]
#![no_main]

use cortex_m_rt::entry;
use stm32f4xx_hal as hal;
use stm32f4xx_hal::prelude::*;

use cortex_m_semihosting::*;
use hal::block;
use hal::gpio::GpioExt;
use hal::pac;
use hal::rcc::RccExt;
use hal::serial::config::Config;
use hal::spi::Spi;
use hal::time::U32Ext;

#[allow(unused_imports)]
use panic_semihosting; // When a panic occurs, dump it to openOCD

// Configuring this wrong can get you in trouble with the law,
// check your local lora band frequency before building
const FREQUENCY: i64 = 868;
const MTU: usize = 255;
const BAUDRATE: u32 = 9600;

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

    hprintln!("Configuring UART OK");

    let mut lora = match sx127x_lora::LoRa::new(spi, cs, reset, FREQUENCY, cp.SYST.delay(&clocks)) {
        Ok(l) => l,
        Err(e) => panic!("{:?}", e),
    };

    lora.set_tx_power(17, 1);

    hprintln!("Configuring Radio OK");

    let mut buffer: [u8; MTU] = [0; MTU];
    let mut index = 0;

    loop {
        //hprint!("l");
        match uart_rx.read() {
            Ok(c) => {
                buffer[index] = c;
                index += 1;
                if index == MTU {
                    let transmit = lora.transmit_payload_busy(buffer, index);
                    match transmit {
                        Ok(_) => hprintln!("tx_c"),
                        Err(e) => hprintln!("tx_err:{:?}", e),
                    }
                    index = 0;
                }
            }
            Err(WouldBlock) => (), //hprintln!("b"),
            Err(e) => hprintln!("s_err:{:?}", e),
        }

        let poll = lora.poll_irq(Some(1)); //TODO: Figure out how long this really is and how long it should be.
        match poll {
            Ok(size) => {
                let buffer = lora.read_packet().unwrap(); // Received buffer. NOTE: 255 bytes are always returned
                for i in 0..size {
                    block!(uart_tx.write(buffer[i]));
                }
            }
            Err(_e) => {} //hprintln!("Timeout"),
        }
    }
}
