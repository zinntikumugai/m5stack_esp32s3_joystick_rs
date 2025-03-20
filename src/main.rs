use embedded_hal::delay::DelayNs;
use esp_backtrace as _;
use esp_idf_hal::delay::{Ets, BLOCK};
use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_sys::{esp_task_wdt_config_t, esp_task_wdt_deinit, esp_task_wdt_init};

const JOYSTICK2_ADDR: u8 = 0x63;
const JOYSTICK2_BOOTLOADER_VERSION_REG: u8 = 0xFC;
const JOYSTICK2_FIRMWARE_VERSION_REG: u8 = 0xFE;
const JOYSTICK2_ADC_16ITS_VALUE_XY_REG: u8 = 0x00;
const JOYSTICK2_RGB_COLOR_REG: u8 = 0x30;
const JOYSTICK2_BUTTON_REG: u8 = 0x20;

struct Joystick2<'a> {
    i2c_driver: I2cDriver<'a>,
}

impl<'a> Joystick2<'a> {
    pub fn new(i2c_driver: I2cDriver<'a>) -> Self {
        Self { i2c_driver }
    }

    fn read_bytes(&mut self, addr: u8, buffer: &mut [u8]) {
        self.i2c_driver
            .write(JOYSTICK2_ADDR, &[addr], BLOCK)
            .unwrap();
        self.i2c_driver.read(JOYSTICK2_ADDR, buffer, BLOCK).unwrap();
    }

    fn write_bytes(&mut self, addr: u8, buffer: &[u8]) {
        let mut data = Vec::with_capacity(buffer.len() + 1);
        data.push(addr);
        data.extend_from_slice(buffer);
        self.i2c_driver.write(JOYSTICK2_ADDR, &data, BLOCK).unwrap();
    }

    fn read_button(&mut self) -> u8 {
        let mut buffer = [0u8; 1];
        self.read_bytes(JOYSTICK2_BUTTON_REG, &mut buffer);
        buffer[0]
    }

    pub fn read_bootloader_version(&mut self) -> u8 {
        let mut buffer = [0u8; 1];
        self.read_bytes(JOYSTICK2_BOOTLOADER_VERSION_REG, &mut buffer);
        buffer[0]
    }

    pub fn read_firmware_version(&mut self) -> u8 {
        let mut buffer = [0u8; 1];
        self.read_bytes(JOYSTICK2_FIRMWARE_VERSION_REG, &mut buffer);
        buffer[0]
    }

    pub fn get_joy_adc_16its_value_xy(&mut self) -> (u16, u16) {
        let mut buffer = [0u8; 4];
        self.read_bytes(JOYSTICK2_ADC_16ITS_VALUE_XY_REG, &mut buffer);
        (
            u16::from_le_bytes([buffer[0], buffer[1]]),
            u16::from_le_bytes([buffer[2], buffer[3]]),
        )
    }

    pub fn get_rgb_color(&mut self) -> [u8; 4] {
        let mut buffer = [0u8; 4];
        self.read_bytes(JOYSTICK2_RGB_COLOR_REG, &mut buffer);
        buffer
    }

    pub fn set_rgb_color(&mut self, color: u32) {
        let color_bytes = color.to_le_bytes();
        self.write_bytes(JOYSTICK2_RGB_COLOR_REG, &color_bytes);
    }
}

fn main() -> ! {
    println!("start!");
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

    let peripherals = Peripherals::take().unwrap();
    let gpios = peripherals.pins;
    let mut delay = Ets;

    let sda: AnyIOPin = gpios.gpio2.into();
    let scl: AnyIOPin = gpios.gpio1.into();

    let i2c_config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c_driver = I2cDriver::new(peripherals.i2c0, sda, scl, &i2c_config).unwrap();

    let mut joystick2 = Joystick2::new(i2c_driver);
    let bootloader_version = joystick2.read_bootloader_version();
    println!("joystick2 bootloader_version: {:02X}", bootloader_version);
    let firmware_version = joystick2.read_firmware_version();
    println!("joystick2 firmware_version: {:02X}", firmware_version);

    let color = 0xff00ff;
    joystick2.set_rgb_color(color);
    println!("joystick2 set rgb_color: 0x{:06X}", color);

    let rgb_color = joystick2.get_rgb_color();
    println!("joystick2 get rgb_color: {:?}", rgb_color);
    delay.delay_ms(3000);

    loop {
        delay.delay_ms(100);

        let (x, y) = joystick2.get_joy_adc_16its_value_xy();
        let button = joystick2.read_button();
        println!(
            "joystick2 x: {:04X}, y: {:04X}, button: {:02X}",
            x, y, button
        );
    }
}
