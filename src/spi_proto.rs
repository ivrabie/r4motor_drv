use crc::{Crc, CRC_16_IBM_SDLC};
use embedded_hal::spi::{Operation as SpiOperation, SpiDevice};


pub const PROTOCOL_OVERHEAD : usize = 5; // 1 byte reg, 2 bytes len, 2 bytes CRC
pub const PROTOCOL_DATA_OFFSET: usize = 3;
pub const PROTOCOL_CRC_SIZE: usize = 2;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpiPackOpType {
    Read = 0u8,
    Write = 1u8,
}


impl TryFrom<u8> for SpiPackOpType {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SpiPackOpType::Read),
            1 => Ok(SpiPackOpType::Write),
            other => Err(other),
        }
    }
}
pub fn execute_spi_transaction(spidev: &mut impl SpiDevice, 
                                buffer: &mut [u8]) { 


    let operation = SpiPackOpType::try_from((buffer[0] >> 7u8) & 0x01u8).unwrap();

    if operation == SpiPackOpType::Write {
        let mut spi_ops = [
            SpiOperation::Write(&buffer),
        ];
        spidev.transaction(&mut spi_ops).expect("SPI transaction failed");
    } else {
        let (header, data_crc) = buffer.split_at_mut(PROTOCOL_OVERHEAD);
        let mut spi_ops = [
            SpiOperation::Write(header),
            SpiOperation::DelayNs(25000),
            SpiOperation::Read(data_crc),
        ];
        spidev.transaction(&mut spi_ops).expect("SPI transaction failed");
    }
}


pub fn populate_header(reg:u8, rw: SpiPackOpType, rw_len: u16, header: &mut [u8]) {
    assert!(header.len() == PROTOCOL_DATA_OFFSET,
            "Header size should be exaclty {} received {}",
            PROTOCOL_DATA_OFFSET, header.len());
    header[0] = (reg & 0x7f) | (rw as u8) << 7;
    header[1..PROTOCOL_DATA_OFFSET as usize].copy_from_slice(&rw_len.to_le_bytes());
}


pub fn populate_crc(buff: &[u8], crc: &mut[u8])
{
    assert!(crc.len() == PROTOCOL_CRC_SIZE,
            "CRC buffer size should be exaclty 2 received {}",
            crc.len());
    let crc_algo = Crc::<u16>::new(&CRC_16_IBM_SDLC);
    let crc_val = crc_algo.checksum(&buff[0..buff.len()]);
    crc.copy_from_slice(&crc_val.to_le_bytes());
}

pub fn validate_crc(buff: &[u8], recv_crc: u16) -> bool {
    let crc_algo = Crc::<u16>::new(&CRC_16_IBM_SDLC);
    let crc = crc_algo.checksum(&buff[0..buff.len()]);
    crc == recv_crc
}






