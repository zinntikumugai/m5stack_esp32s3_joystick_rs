use esp_idf_hal::delay::BLOCK;
use esp_idf_hal::i2c::I2cDriver;
use std::sync::{Arc, Mutex};

const SCROLL_ADDR: u8 = 0x40;
const SCROLL_ENCODER_REG: u8 = 0x10;
const SCROLL_BUTTON_REG: u8 = 0x20;
const SCROLL_RGB_COLOR_REG: u8 = 0x30;
const SCROLL_RSET_REG: u8 = 0x40;
const SCROLL_INC_ENCODER_REG: u8 = 0x50;
const SCROLL_BOOTLOADER_VERSION_REG: u8 = 0xFC;
const SCROLL_FIRMWARE_VERSION_REG: u8 = 0xFD;

pub struct Scroll<'a> {
    i2c_driver: Arc<Mutex<I2cDriver<'a>>>,
}

impl<'a> Scroll<'a> {
    pub fn new(i2c_driver: Arc<Mutex<I2cDriver<'a>>>) -> Self {
        Self { i2c_driver }
    }

    fn read_bytes(&mut self, addr: u8, buffer: &mut [u8]) {
        let mut driver = self.i2c_driver.lock().unwrap();
        driver.write(SCROLL_ADDR, &[addr], BLOCK).unwrap();
        driver.read(SCROLL_ADDR, buffer, BLOCK).unwrap();
    }

    fn write_bytes(&mut self, addr: u8, buffer: &[u8]) {
        let mut driver = self.i2c_driver.lock().unwrap();
        let mut data = Vec::with_capacity(buffer.len() + 1);
        data.push(addr);
        data.extend_from_slice(buffer);
        driver.write(SCROLL_ADDR, &data, BLOCK).unwrap();
    }

    pub fn get_firmware_version(&mut self) -> u8 {
        let mut buffer = [0u8; 1];
        self.read_bytes(SCROLL_FIRMWARE_VERSION_REG, &mut buffer);
        buffer[0]
    }

    pub fn get_bootloader_version(&mut self) -> u8 {
        let mut buffer = [0u8; 1];
        self.read_bytes(SCROLL_BOOTLOADER_VERSION_REG, &mut buffer);
        buffer[0]
    }

    pub fn get_encoder_value(&mut self) -> u16 {
        let mut buffer = [0u8; 2];
        self.read_bytes(SCROLL_ENCODER_REG, &mut buffer);
        u16::from_le_bytes([buffer[0], buffer[1]])
    }

    pub fn get_button_value(&mut self) -> u8 {
        let mut buffer = [0u8; 1];
        self.read_bytes(SCROLL_BUTTON_REG, &mut buffer);
        buffer[0]
    }

    pub fn get_rgb_color(&mut self) -> [u8; 4] {
        let mut buffer = [0u8; 4];
        self.read_bytes(SCROLL_RGB_COLOR_REG, &mut buffer);
        [buffer[0], buffer[1], buffer[2], buffer[3]]
    }

    pub fn set_rgb_color(&mut self, color: u32) {
        let color_bytes = color.to_le_bytes();
        self.write_bytes(SCROLL_RGB_COLOR_REG, &color_bytes);
    }

    pub fn reset_encoder(&mut self) {
        self.write_bytes(SCROLL_RSET_REG, &[0x01]);
    }
}
