#![no_std]
#![no_main]

use stm32f4xx_hal as hal;
use cortex_m_rt::entry; // The runtime
use stm32f4xx_hal::prelude::*; // STM32F1 specific functions


use cortex_m_semihosting::*;
use hal::block;
use hal::pac;
use hal::gpio::GpioExt;
use hal::rcc::RccExt;
use hal::spi::Spi;
use hal::serial::config::Config;

#[allow(unused_imports)]
use panic_semihosting; // When a panic occurs, dump it to openOCD

const FREQUENCY: i64 = 868;

#[entry]
fn main() -> ! {
    // Get handles to the hardware objects. These functions can only be called
    // once, so that the borrowchecker can ensure you don't reconfigure
    // something by accident.
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();    

    let clocks = rcc.cfgr.sysclk(64.MHz()).pclk1(32.MHz()).freeze();

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    // let gpiod = dp.GPIOD.split();
    // let gpiof = dp.GPIOF.split();

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
        &clocks
    );

    let uart_tx = gpioa.pa2.into_alternate();
    let uart_rx = gpioa.pa3.into_alternate();
    
    let (mut uart_tx, mut uart_rx) = dp.USART2.serial((uart_tx, uart_rx), Config::default().baudrate(9600.bps()), &clocks).unwrap().split();

    hprintln!("Configuring UART OK");
    let bstr = ['L', 'O', 'R', 'A', '\r', '\n'];
    for c in bstr {
        block!(uart_tx.write(c as u8));
    }

    let mut lora = match sx127x_lora::LoRa::new(
        spi, cs, reset, FREQUENCY,
        cp.SYST.delay(&clocks)) {
            Ok(l) => l,
            Err(e) => panic!("{:?}", e),
        };

    lora.set_tx_power(17,1);

    hprintln!("Configuring Radio OK");
    
    let mut buffer: [u8; 255] = [0; 255];
    let mut index = 0;

    loop {
        match uart_rx.read() {
            Ok(c) => {
                //hprintln!("got {}", c);
                if c == 13 {
                    let transmit = lora.transmit_payload_busy(buffer,index + 1);
                    match transmit {
                        Ok(_) => hprintln!("y"),
                        Err(e) => hprintln!("Transmit Error {:?}", e),
                    }
                    index = 0;
                } else {
                    buffer[index] = c;
                    index += 1;
                }
            },
            Err(WouldBlock) => (),
            Err(e) => hprintln!("Serial Read Error {:?}", e),
        }
        
        let poll = lora.poll_irq(Some(30)); //30 Second timeout
        match poll {
            Ok(size) =>{
               let buffer = lora.read_packet().unwrap(); // Received buffer. NOTE: 255 bytes are always returned
               for i in 0..size{
                    block!(uart_tx.write(buffer[i]));
               }
               block!(uart_tx.write('\r' as u8));
               block!(uart_tx.write('\n' as u8));
            },
            Err(_e) => {}, //hprintln!("Timeout"),
        }
    }
}
