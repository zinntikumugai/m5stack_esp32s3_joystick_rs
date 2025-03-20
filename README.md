# m5stack_esp32s3_inputs_rs

- m5stack Atom (ESP32S3)
- 1 to 3 Hub Unit
    - JoyStick2 Unit
    - Scroll Unit

```bash
# internal container
cargo build
espflash flash /tmp/target/xtensa-esp32s3-espidf/debug/m5stack_esp32s3_inputs_rs
espmonitor /dev/ttyACM0
```