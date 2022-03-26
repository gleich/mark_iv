#![no_std]
#![no_main]

use core::fmt::Write;

use cortex_m_rt::entry;

use defmt::info;
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_time::rate::Extensions;
use embedded_time::{duration::*, Clock};

use embedded_hal::timer::CountDown;

use panic_probe as _;

use rp_pico::hal::pac;

use rp_pico::hal;

use defmt_rtt as _;

use embedded_graphics::prelude::*;

use ssd1306::{prelude::*, Ssd1306};
use tinybmp::Bmp;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = hal::Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure two pins as being I²C, not GPIO
    let sda_pin = pins.gpio16.into_mode::<hal::gpio::FunctionI2C>();
    let scl_pin = pins.gpio17.into_mode::<hal::gpio::FunctionI2C>();

    // Create the I²C driver, using the two pre-configured pins. This will fail
    // at compile time if the pins are in the wrong mode, or if this I²C
    // peripheral isn't available on these pins!
    let i2c = hal::I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut pac.RESETS,
        clocks.peripheral_clock,
    );

    // Create the I²C display interface:
    let interface = ssd1306::I2CDisplayInterface::new(i2c);

    // Create a driver instance and initialize:
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut delay = timer.count_down();

    let bmp = Bmp::from_slice(include_bytes!("./logo.bmp")).expect("Failed to load BMP image");

    let mut y = 0;
    let mut up = false;
    loop {
        let im: Image<Bmp<Rgb565>> = Image::new(&bmp, Point::new(0, y));
        im.draw(&mut display.color_converted()).unwrap();
        display.flush().unwrap();
        if y == -70 || y == 0 {
            delay.start(5.seconds());
            let _ = nb::block!(delay.wait());
            up = !up;
        }
        info!("y = {}", y);
        if up {
            y -= 1;
        } else {
            y += 1;
        }
    }
}
