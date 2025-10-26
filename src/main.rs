
use embedded_hal::spi::{Operation as SpiOperation, SpiDevice};
use linux_embedded_hal::{SpidevDevice,  spidev::{SpidevOptions, SpiModeFlags}};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut spi = SpidevDevice::open("/dev/spidev0.0").expect("Failed to open");
    
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(500_000)  // 1 MHz
        .mode(SpiModeFlags::SPI_MODE_0)
        .lsb_first(false)
        .build();
    println!("{}", options.lsb_first.unwrap());
    spi.configure(&options).expect("Failed to configure");
    let mut read_buf = [0u8; 5];
    // let mut spi_ops = [
    //     SpiOperation::Write(&[0xCE]),
    //     SpiOperation::DelayNs(25000),
    //     SpiOperation::Read(& mut read_buf)
    // ];
    // spi.transaction(&mut spi_ops).expect("Failed to send");
    spi.write(&[0xCE]).unwrap();
    sleep(Duration::from_micros(1500));
    spi.read(&mut read_buf).unwrap();
    println!("{:?}", read_buf);
}