use esp_idf_hal::delay::BLOCK;
use esp_idf_hal::i2c::I2cDriver;
use std::sync::{Arc, Mutex};

const JOYSTICK2_ADDR: u8 = 0x63;
const JOYSTICK2_BOOTLOADER_VERSION_REG: u8 = 0xFC;
const JOYSTICK2_FIRMWARE_VERSION_REG: u8 = 0xFE;
const JOYSTICK2_ADC_16ITS_VALUE_XY_REG: u8 = 0x00;
const JOYSTICK2_RGB_COLOR_REG: u8 = 0x30;
const JOYSTICK2_BUTTON_REG: u8 = 0x20;

pub struct Joystick2<'a> {
    i2c_driver: Arc<Mutex<I2cDriver<'a>>>,
}

impl<'a> Joystick2<'a> {
    pub fn new(i2c_driver: Arc<Mutex<I2cDriver<'a>>>) -> Self {
        Self { i2c_driver }
    }

    fn read_bytes(&mut self, addr: u8, buffer: &mut [u8]) {
        let mut driver = self.i2c_driver.lock().unwrap();
        driver.write(JOYSTICK2_ADDR, &[addr], BLOCK).unwrap();
        driver.read(JOYSTICK2_ADDR, buffer, BLOCK).unwrap();
    }

    fn write_bytes(&mut self, addr: u8, buffer: &[u8]) {
        let mut driver = self.i2c_driver.lock().unwrap();
        let mut data = Vec::with_capacity(buffer.len() + 1);
        data.push(addr);
        data.extend_from_slice(buffer);
        driver.write(JOYSTICK2_ADDR, &data, BLOCK).unwrap();
    }

    pub fn read_button(&mut self) -> u8 {
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
