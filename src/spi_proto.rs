use std::thread::sleep;

use crc::{Crc, CRC_16_IBM_SDLC};
use embedded_hal::spi::SpiDevice;
use log::debug;
use num_enum::TryFromPrimitive;

pub const PROTOCOL_HEADER_SIZE: usize = 5; // 1 byte reg, 2 bytes len, 2 bytes CRC
pub const PROTOCOL_HEADER_CRC_OFFSET: usize = 3;
pub const PROTOCOL_CRC_SIZE: usize = 2;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive)]
pub enum SpiPackOpType {
    Read = 0u8,
    Write = 1u8,
}

pub fn execute_spi_transaction(spidev: &mut impl SpiDevice, buffer: &mut [u8]) {
    let operation = SpiPackOpType::try_from((buffer[0] >> 7u8) & 0x01u8).unwrap();
    let (header, data_crc) = buffer.split_at_mut(PROTOCOL_HEADER_SIZE);
    spidev.write(header).unwrap();
    sleep(std::time::Duration::from_millis(50));
    if operation == SpiPackOpType::Write {
        spidev.write(data_crc).unwrap();
        sleep(std::time::Duration::from_millis(50));
    } else {
        spidev.read(data_crc).unwrap();
        sleep(std::time::Duration::from_millis(50));
        // spidev.transaction(&mut spi_ops).expect("SPI transaction failed");
        debug!("Received data: {:x?}", data_crc);
    }
}

pub fn populate_header(reg: u8, rw: SpiPackOpType, rw_len: u16, header: &mut [u8]) {
    assert!(
        header.len() == PROTOCOL_HEADER_SIZE,
        "Header size should be exaclty {} received {}",
        PROTOCOL_HEADER_SIZE,
        header.len()
    );
    let (header, crc) = header.split_at_mut(PROTOCOL_HEADER_CRC_OFFSET);
    header[0] = (reg & 0x7f) | ((rw as u8) << 7);
    header[1..PROTOCOL_HEADER_CRC_OFFSET as usize].copy_from_slice(&rw_len.to_le_bytes());
    populate_crc(header, crc);
}

pub fn populate_crc(buff: &[u8], crc: &mut [u8]) {
    assert!(
        crc.len() == PROTOCOL_CRC_SIZE,
        "CRC buffer size should be exaclty 2 received {}",
        crc.len()
    );
    let crc_algo = Crc::<u16>::new(&CRC_16_IBM_SDLC);
    let crc_val = crc_algo.checksum(&buff[0..buff.len()]);
    crc.copy_from_slice(&crc_val.to_le_bytes());
}

pub fn validate_crc(buff: &[u8], recv_crc: u16) -> bool {
    let crc_algo = Crc::<u16>::new(&CRC_16_IBM_SDLC);
    let crc = crc_algo.checksum(&buff[0..buff.len()]);
    crc == recv_crc
}
