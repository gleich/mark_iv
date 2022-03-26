#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;
use embedded_time::duration::*;
use embedded_time::rate::Extensions;
use rp_pico::hal;
use rp_pico::hal::pac;
use ssd1306::prelude::*;
use ssd1306::Ssd1306;
use tinybmp::Bmp;
use {defmt_rtt as _, panic_probe as _};

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

	let pins = rp_pico::Pins::new(
		pac.IO_BANK0,
		pac.PADS_BANK0,
		sio.gpio_bank0,
		&mut pac.RESETS,
	);

	let sda_pin = pins.gpio16.into_mode::<hal::gpio::FunctionI2C>();
	let scl_pin = pins.gpio17.into_mode::<hal::gpio::FunctionI2C>();

	let i2c = hal::I2C::i2c0(
		pac.I2C0,
		sda_pin,
		scl_pin,
		400.kHz(),
		&mut pac.RESETS,
		clocks.peripheral_clock,
	);

	let interface = ssd1306::I2CDisplayInterface::new(i2c);

	let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
		.into_buffered_graphics_mode();
	display.init().unwrap();

	let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS);
	let mut delay = timer.count_down();

	let bmp = Bmp::from_slice(include_bytes!("./logo.bmp")).expect("Failed to load BMP image");

	pins.led
		.into_push_pull_output()
		.set_high()
		.expect("Failed to set board LED to high");

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
