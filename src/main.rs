use embedded_hal::delay::DelayNs;
use esp_backtrace as _;
use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_sys::{esp_task_wdt_config_t, esp_task_wdt_deinit, esp_task_wdt_init};
use std::sync::{Arc, Mutex};

mod joystick2;
mod scroll;
use joystick2::Joystick2;
use scroll::Scroll;

fn init_wdt() {
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
}

fn main() -> ! {
    println!("start!");
    init_wdt();

    let peripherals = Peripherals::take().unwrap();
    let gpios = peripherals.pins;
    let mut delay = Ets;

    // I2C
    let sda: AnyIOPin = gpios.gpio2.into();
    let scl: AnyIOPin = gpios.gpio1.into();

    let i2c_config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c_driver = Arc::new(Mutex::new(
        I2cDriver::new(peripherals.i2c0, sda, scl, &i2c_config).unwrap(),
    ));

    // Device
    let mut joystick2 = Joystick2::new(i2c_driver.clone());
    let mut scroll = Scroll::new(i2c_driver);

    println!("joystick2");
    let bootloader_version = joystick2.read_bootloader_version();
    println!("joystick2 bootloader_version: {:02X}", bootloader_version);
    let firmware_version = joystick2.read_firmware_version();
    println!("joystick2 firmware_version: {:02X}", firmware_version);

    let color = 0xff00ff;
    joystick2.set_rgb_color(color);
    println!("joystick2 set rgb_color: 0x{:06X}", color);

    let rgb_color = joystick2.get_rgb_color();
    println!("joystick2 get rgb_color: {:?}", rgb_color);

    println!("scroll");
    let bootloader_version = scroll.get_bootloader_version();
    println!("scroll bootloader_version: {:02X}", bootloader_version);
    let firmware_version = scroll.get_firmware_version();
    println!("scroll firmware_version: {:02X}", firmware_version);

    let rgb_color = 0xff00ff;
    scroll.set_rgb_color(rgb_color);
    println!("scroll set rgb_color: 0x{:06X}", rgb_color);

    delay.delay_ms(3000);

    loop {
        delay.delay_ms(100);

        let (x, y) = joystick2.get_joy_adc_16its_value_xy();
        let button = joystick2.read_button();
        println!(
            "joystick2 x: {:04X}, y: {:04X}, button: {:02X}",
            x, y, button
        );

        let encoder_value = scroll.get_encoder_value();
        let button = scroll.get_button_value();
        let rgb_color = scroll.get_rgb_color();
        println!(
            "scroll encoder_value: {:04X}, button: {:02X}, rgb_color: {:?}",
            encoder_value, button, rgb_color
        );

        if button == 0x00 {
            scroll.reset_encoder();
            println!("scroll reset encoder");
        }
    }
}
