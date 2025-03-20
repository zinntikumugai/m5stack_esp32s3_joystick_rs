use embedded_graphics::image::Image;
use embedded_hal::delay::DelayNs;
use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::*;
use esp_idf_hal::units::FromValueType;

use embedded_graphics::mono_font::iso_8859_1::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use esp_backtrace as _;
use esp_idf_sys::{esp_task_wdt_config_t, esp_task_wdt_deinit, esp_task_wdt_init};
use mipidsi::interface::SpiInterface;
use mipidsi::options::ColorOrder;
use mipidsi::{models, Builder};
use tinybmp::Bmp;

const IMG_BYTES: &[u8] = include_bytes!("./img.bmp");

fn main() -> ! {
    // 起動時にウォッチドッグタイマーを無効化
    unsafe {
        esp_task_wdt_deinit(); // 既存のTWDTをリセット

        let config = esp_task_wdt_config_t {
            timeout_ms: 20000,
            idle_core_mask: 0, // すべてのコアを無効化
            trigger_panic: false,
        };
        esp_task_wdt_init(&config);
    }

    println!("start!");
    let peripherals = Peripherals::take().unwrap();
    let gpios = peripherals.pins;

    let mut rst = PinDriver::output(gpios.gpio34).unwrap();
    let dc = PinDriver::output(gpios.gpio33).unwrap();
    let mut backlight = PinDriver::output(gpios.gpio16).unwrap();
    let sclk = gpios.gpio17;
    let sda = gpios.gpio21;
    let cs = gpios.gpio15;

    let mut delay = Ets;

    let spi = peripherals.spi2;

    println!("hard reset start");
    rst.set_low().unwrap();
    delay.delay_ms(100);
    rst.set_high().unwrap();
    delay.delay_ms(2000);

    // configuring the spi interface
    let config = config::Config::new().baudrate(50.MHz().into());

    let driver =
        SpiDriver::new::<SPI2>(spi, sclk, sda, None::<Gpio15>, &SpiDriverConfig::new()).unwrap();

    let device = SpiDeviceDriver::new(driver, Some(cs), &config).unwrap();

    // display interface abstraction from SPI and DC
    let mut buffer = [0u8; 32];
    let di = SpiInterface::new(device, dc, &mut buffer);

    let mut display = Builder::new(models::GC9107, di)
        .reset_pin(rst)
        .color_order(ColorOrder::Bgr)
        .init(&mut delay)
        .unwrap();

    println!("setup!");

    // turn on the backlight
    backlight.set_high().unwrap();

    let text_style_a = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(Rgb565::BLUE)
        .build();
    let text_a = Text::with_baseline(
        "Hello World.",
        Point::new(0, 32),
        text_style_a,
        Baseline::Top,
    );

    let bmp = Bmp::from_slice(IMG_BYTES).unwrap();
    let img = Image::new(&bmp, Point::new(0, 40));
    println!("setup2!");

    // draw image on black background
    display.clear(Rgb565::BLACK).unwrap();

    img.draw(&mut display).unwrap();

    text_a.draw(&mut display).unwrap();

    println!("Image printed!");

    loop {
        println!("Hello, world!");
        delay.delay_ms(1000);
    }
}
