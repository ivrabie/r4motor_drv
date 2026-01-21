/*
 ## Regs
| Reg(1byte) | Description |  Default Value (4bytes) | Access | Detailed description |
|-------------|-------------|----------------|--------|----------------------|
| 0x00 | Device ID | "4MOT" | RO | Unique identifier for the device |
| 0x01 | Firmware version | "0.0.0.1" | RO | Current firmware version |
| 0x02 | Motor1 Operation Mode  | 0 | RW | 0: Pwm control, 1: Rpm control |
| 0x03 | Motor1 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x04 | Motor1 pwm duty cycle | 0 | RW | PWM duty cycle for Motor1 (0-100) |
| 0x05 | Motor1 counts per revolution | 1 | RW | Counts per revolution for Motor1 |
| 0x06 | Motor1 pid kp | 0 | RW | PID Kp for Motor1 |
| 0x07 | Motor1 pid ki | 0 | RW | PID Ki for Motor1 |
| 0x08 | Motor1 pid kd | 0 | RW | PID Kd for Motor1 |
| 0x09 | Motor1 rpm desired | 0 | RW | Desired rpm for Motor1 (0-200) |
| 0x0A | Motor1 rpm current | 0 | R | Current rpm for Motor1 |
| 0x0B | Motor2 Operation Mode  | 0 | RW | 0: Pwm control, 1: Rpm control |
| 0x0C | Motor2 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x0D | Motor2 pwm duty cycle | 0 | RW | PWM duty cycle for Motor2 (0-100) |
| 0x0E | Motor2 counts per revolution | 1 | RW | Counts per revolution for Motor2 |
| 0x0F | Motor2 pid kp | 0 | RW | PID Kp for Motor2 |
| 0x10 | Motor2 pid ki | 0 | RW | PID Ki for Motor2 |
| 0x11 | Motor2 pid kd | 0 | RW | PID Kd for Motor2 |
| 0x12 | Motor2 rpm desired | 0 | RW | Desired rpm for Motor2 (0-200) |
| 0x13 | Motor2 rpm current | 0 | R | Current rpm for Motor2 |
| 0x14 | Motor3 Operation Mode  | 0 | RW | 0: Pwm control, 1: Rpm control |
| 0x15 | Motor3 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x16 | Motor3 pwm duty cycle | 0 | RW | PWM duty cycle for Motor3 (0-100) |
| 0x17 | Motor3 counts per revolution | 1 | RW | Counts per revolution for Motor3 |
| 0x18 | Motor3 pid kp | 0 | RW | PID Kp for Motor3 |
| 0x19 | Motor3 pid ki | 0 | RW | PID Ki for Motor3 |
| 0x1A | Motor3 pid kd | 0 | RW | PID Kd for Motor3 |
| 0x1B | Motor3 rpm desired | 0 | RW | Desired rpm for Motor3 (0-200) |
| 0x1C | Motor3 rpm current | 0 | R | Current rpm for Motor3 |
| 0x1D | Motor4 Operation Mode  | 0 | RW | 0: Pwm control, 1: Rpm control |
| 0x1E | Motor4 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x1F | Motor4 pwm duty cycle | 0 | RW | PWM duty cycle for Motor4 (0-100) |
| 0x20 | Motor4 counts per revolution | 1 | RW | Counts per revolution for Motor4 |
| 0x21 | Motor4 pid kp | 0 | RW | PID Kp for Motor4 |
| 0x22 | Motor4 pid ki | 0 | RW | PID Ki for Motor4 |
| 0x23 | Motor4 pid kd | 0 | RW | PID Kd for Motor4 |
| 0x24 | Motor4 rpm desired | 0 | RW | Desired rpm for Motor4 (0-200) |
| 0x25 | Motor4 rpm current | 0 | R | Current rpm for Motor4 |
| 0x26 | Internal loop time | 10 | RW | Internal loop time (1-10000) |
| 0x27 | Last error status | 0 | R | Last error status --> clears on read |

# Errors
| Error code | Description |
|-------------|-------------|
| 0x00 | No error |
| 0x01 | Invalid register address |
| 0x02 | Invalid request length |
| 0x03 | Invalid register value range |
| 0x04 | Not allowed read only |
| 0x05 | Invalid control mode |
| 0x06 | Write not allowed in this control mode |
| 0x07 | CRC validation failed |

# Notes
- PWM duty cycle becomes read-only when motor is in RPM control mode
- Error status register clears to NoError when read
*/
use clap::ValueEnum;
use linux_embedded_hal::{
    spidev::{SpiModeFlags, SpidevOptions},
    SpidevDevice,
};
use crate::{device, spi_proto};
use num_enum::TryFromPrimitive;

pub const REGISTERS_COUNT: usize = 40;
pub const REGISTERS_SIZE_BYTES: usize = REGISTERS_COUNT * 4;
pub const REGISTERS_PROTO_SIZE: usize = REGISTERS_SIZE_BYTES + spi_proto::PROTOCOL_HEADER_SIZE + spi_proto::PROTOCOL_CRC_SIZE;
pub const DEVICE_SUPPORTED_MOTORS: usize = 4;
pub const DEVICE_MOTOR_BLOCK_COUNT_READ: usize = 9; // Number of registers per motor
pub const DEVICE_MOTOR_BLOCK_COUNT_WRITE: usize = 8; // Number of writable registers per motor
pub const DEVICE_REG_SIZE_BYTES: usize = 4; // Each register is 4 bytes
pub const DEVICE_MOTOR_BLOCK_SIZE_BYTES_READ: usize = DEVICE_MOTOR_BLOCK_COUNT_READ * DEVICE_REG_SIZE_BYTES; // Size of each motor block in bytes
pub const DEVICE_ALL_MOTORS_BLOCK_SIZE_BYTES_READ: usize = DEVICE_SUPPORTED_MOTORS * DEVICE_MOTOR_BLOCK_SIZE_BYTES_READ;
pub const DEVICE_MOTOR_BLOCK_SIZE_BYTES_WRITE: usize = DEVICE_MOTOR_BLOCK_COUNT_WRITE * DEVICE_REG_SIZE_BYTES; // Size of each writable motor block in bytes
pub const DEVICE_PID_PARAMS_SIZE_BYTES: usize = 3 * DEVICE_REG_SIZE_BYTES; // Size of PID params block in bytes
// New global constants for  dump processing
pub const DEVICE_ID_REG_SIZE: usize = 4;
pub const FIRMWARE_VERSION_REG_SIZE: usize = 4;
pub const SYSTEM_INFO_SIZE: usize = DEVICE_ID_REG_SIZE + FIRMWARE_VERSION_REG_SIZE;
pub const MOTORS_DATA_START_OFFSET: usize = SYSTEM_INFO_SIZE;
pub const INTERNAL_LOOP_TIME_OFFSET: usize = MOTORS_DATA_START_OFFSET + DEVICE_ALL_MOTORS_BLOCK_SIZE_BYTES_READ;

#[repr(i32)]
#[derive(Debug, Clone, ValueEnum, Copy, PartialEq, Eq, TryFromPrimitive)]
pub enum ControlMode {
    Pwm = 0,
    Rpm = 1,
}

#[repr(i32)]
#[derive(Debug, Clone, ValueEnum, Copy, PartialEq, Eq, TryFromPrimitive)]
pub enum MotorDirection {
    Stop = 0,
    Fw = 1,
    Bw = 2,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
pub enum ErrorCode {
    NoError = 0x00,
    InvalidRegisterAddress = 0x01,
    InvalidRequestLength = 0x02,
    InvalidRegisterValueRange = 0x03,
    NowAllowedReadOnly = 0x04,
    InvalidControlMode = 0x05,
    WriteNotAllowedInThisControlMode = 0x06,
    CrcValidationFailed = 0x07,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum RegisterID {
    DeviceID = 0x00,
    FirmwareVersion = 0x01,
    Motor1OperationMode = 0x02,
    Motor1Direction = 0x03,
    Motor1PWMDutyCycle = 0x04,
    Motor1CountsPerRevolution = 0x05,
    Motor1PIDKp = 0x06,
    Motor1PIDKi = 0x07,
    Motor1PIDKd = 0x08,
    Motor1RPMDesired = 0x09,
    Motor1RPMCurrent = 0x0A,
    Motor2OperationMode = 0x0B,
    Motor2Direction = 0x0C,
    Motor2PWMDutyCycle = 0x0D,
    Motor2CountsPerRevolution = 0x0E,
    Motor2PIDKp = 0x0F,
    Motor2PIDKi = 0x10,
    Motor2PIDKd = 0x11,
    Motor2RPMDesired = 0x12,
    Motor2RPMCurrent = 0x13,
    Motor3OperationMode = 0x14,
    Motor3Direction = 0x15,
    Motor3PWMDutyCycle = 0x16,
    Motor3CountsPerRevolution = 0x17,
    Motor3PIDKp = 0x18,
    Motor3PIDKi = 0x19,
    Motor3PIDKd = 0x1A,
    Motor3RPMDesired = 0x1B,
    Motor3RPMCurrent = 0x1C,
    Motor4OperationMode = 0x1D,
    Motor4Direction = 0x1E,
    Motor4PWMDutyCycle = 0x1F,
    Motor4CountsPerRevolution = 0x20,
    Motor4PIDKp = 0x21,
    Motor4PIDKi = 0x22,
    Motor4PIDKd = 0x23,
    Motor4RPMDesired = 0x24,
    Motor4RPMCurrent = 0x25,
    InternalLoopTime = 0x26,
    LastErrorStatus = 0x27,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
pub enum MotorRegisterOffset {
    OperationMode = 0,
    Direction = 1,
    PWMDutyCycle = 2,
    CountsPerRevolution = 3,
    PIDKp = 4,
    PIDKi = 5,
    PIDKd = 6,
    RPMDesired = 7,
    RPMCurrent = 8,
}
#[repr(u8)]
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
pub enum MotorID {
    Motor1,
    Motor2,
    Motor3,
    Motor4,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PidParams {
    pub kp: i32,
    pub ki: i32,
    pub kd: i32,
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub device_id: String,
    pub firmware_version: String,
}
#[derive(Debug, Clone, Copy)]
pub struct MotorStatus {
    pub mode: ControlMode,
    pub direction: MotorDirection,
    pub pwm_duty_cycle: i32,
    pub counts_per_revolution: i32,
    pub rpm_current: i32,
    pub rpm_desired: i32,
    pub pid_params: PidParams,
}

#[derive(Debug, Clone, Copy)]
pub struct MotorCfg {
    pub mode: ControlMode,
    pub direction: MotorDirection,
    pub pwm_duty_cycle: i32,
    pub counts_per_revolution: i32,
    pub rpm_desired: i32,
    pub pid_params: PidParams,
}

impl Default for MotorStatus {
    fn default() -> Self {
        Self {
            mode: ControlMode::Pwm,
            direction: MotorDirection::Stop,
            pwm_duty_cycle: 0,
            counts_per_revolution: 0,
            rpm_current: 0,
            rpm_desired: 0,
            pid_params: PidParams::default(),
        }
    }
}

pub struct DeviceFullInfo {
    pub system_info: SystemInfo,
    pub motors_status: [MotorStatus; DEVICE_SUPPORTED_MOTORS],
    pub internal_loop_time_ms: i32,
    pub last_error_status: ErrorCode,
}

pub struct DeviceCfg {
    pub motors_cfg: [MotorCfg; DEVICE_SUPPORTED_MOTORS],
    pub internal_loop_time_ms: i32,
}

pub struct Device {
    spi_dev: SpidevDevice,
}

impl RegisterID {
    pub fn from_motor_id(motor_id: MotorID, offset: MotorRegisterOffset) -> Self {
        debug_assert!(
            (offset as usize) < DEVICE_MOTOR_BLOCK_COUNT_READ,
            "motor register offset out of range: {}",
            offset as usize
        );
        let base = RegisterID::Motor1OperationMode as u8
            + (motor_id as u8) * DEVICE_MOTOR_BLOCK_COUNT_READ as u8
            + offset as u8;
        RegisterID::try_from(base).unwrap()
    }
}

impl Device {
    pub fn new(spi_if: &str) -> Self {
        let mut spi = SpidevDevice::open(spi_if).expect("Failed to open");
        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(1_000_000) // 1 MHz
            .mode(SpiModeFlags::SPI_MODE_0)
            .lsb_first(false)
            .build();
        debug_assert_eq!(options.lsb_first, Some(false));
        spi.configure(&options).expect("Failed to configure");

        Device {
            spi_dev: spi,
        }
    }

    fn extract_u32_from_bytes(data: &[u8]) -> i32 {
        debug_assert!(
            data.len() >= DEVICE_REG_SIZE_BYTES,
            "expected at least {} bytes",
            DEVICE_REG_SIZE_BYTES
        );
        i32::from_le_bytes([data[0], data[1], data[2], data[3]])
    }

    fn parse_system_info(reg_dump: &[u8]) -> SystemInfo {
        debug_assert!(
            reg_dump.len() >= SYSTEM_INFO_SIZE,
            "system info dump length mismatch"
        );
        let device_id = String::from_utf8_lossy(&reg_dump[0..DEVICE_ID_REG_SIZE]).to_string();
        let firmware_version = format!(
            "{}{}.{}{}",
            reg_dump[DEVICE_ID_REG_SIZE],
            reg_dump[DEVICE_ID_REG_SIZE + 1],
            reg_dump[DEVICE_ID_REG_SIZE + 2],
            reg_dump[DEVICE_ID_REG_SIZE + 3],
        );

        SystemInfo {
            device_id,
            firmware_version,
        }
    }

    fn populate_motor_status(&mut self, motor_num: MotorID, data: &[u8]) -> MotorStatus {
        debug_assert_eq!(
            data.len(),
            DEVICE_MOTOR_BLOCK_SIZE_BYTES_READ,
            "motor {:?} register dump length mismatch",
            motor_num
        );

        let mut status = MotorStatus::default();
        let mut regs = data.chunks_exact(DEVICE_REG_SIZE_BYTES);

        status.mode = Self::extract_u32_from_bytes(regs.next().unwrap()).try_into().unwrap();
        status.direction = Self::extract_u32_from_bytes(regs.next().unwrap()).try_into().unwrap();
        status.pwm_duty_cycle = Self::extract_u32_from_bytes(regs.next().unwrap());
        status.counts_per_revolution = Self::extract_u32_from_bytes(regs.next().unwrap());
        status.pid_params.kp = Self::extract_u32_from_bytes(regs.next().unwrap());
        status.pid_params.ki = Self::extract_u32_from_bytes(regs.next().unwrap());
        status.pid_params.kd = Self::extract_u32_from_bytes(regs.next().unwrap());
        status.rpm_desired = Self::extract_u32_from_bytes(regs.next().unwrap());
        status.rpm_current = Self::extract_u32_from_bytes(regs.next().unwrap());

        status
    }

    pub fn req_regs_dump(&mut self, reg_id: RegisterID, reg_dump: &mut [u8]) {
        assert!(
            reg_dump.len() + spi_proto::PROTOCOL_HEADER_SIZE <= REGISTERS_PROTO_SIZE,
            "Requested length exceeds register dump size"
        );
        // Placeholder for actual implementation to request register dump from device
        let mut proto_buff: [u8; REGISTERS_PROTO_SIZE] = [0; REGISTERS_PROTO_SIZE];
        let header = &mut proto_buff[0..spi_proto::PROTOCOL_HEADER_SIZE];
        spi_proto::populate_header(
            reg_id as u8,
            spi_proto::SpiPackOpType::Read,
            reg_dump.len() as u16,
            header,
        );
        // Here you would send the request via SPI and read the response
        println!("Register dump requested.");
        println!("reg dump len: {}", reg_dump.len());
        let write_buff_size = reg_dump.len() + spi_proto::PROTOCOL_HEADER_SIZE + spi_proto::PROTOCOL_CRC_SIZE;
        let register_dump_slice = &mut proto_buff[0..write_buff_size];
        spi_proto::execute_spi_transaction(&mut self.spi_dev, register_dump_slice);

        // Process the received register dump
        println!("Register dump received.");
        let recv_crc_start_idx = reg_dump.len() + spi_proto::PROTOCOL_HEADER_SIZE;
        let recv_crc = u16::from_le_bytes([
            proto_buff[recv_crc_start_idx],
            proto_buff[recv_crc_start_idx + 1],
        ]);
        let is_valid_crc = spi_proto::validate_crc(&proto_buff[spi_proto::PROTOCOL_HEADER_SIZE..recv_crc_start_idx], recv_crc);
        if is_valid_crc {
            println!("CRC validation passed.");
            reg_dump
                .copy_from_slice(&proto_buff[spi_proto::PROTOCOL_HEADER_SIZE..recv_crc_start_idx]);
        } else {
            println!("CRC validation failed received.");
        }
    }

    pub fn req_reg_write(&mut self, reg_id: RegisterID, regs: &[u8]) {
        assert!(
            regs.len() <= REGISTERS_SIZE_BYTES,
            "Requested length exceeds register dump size"
        );
        assert!(
            regs.len() % 4 == 0,
            "Requested len should be multiple of 4, a register have 4 bytes"
        );

        println!("Writing to register ID: {:?}, data length: {}", reg_id, regs.len());
        let mut proto_buff: [u8; REGISTERS_PROTO_SIZE] = [0; REGISTERS_PROTO_SIZE];
        let proto_buff_size_used = spi_proto::PROTOCOL_HEADER_SIZE + regs.len() + spi_proto::PROTOCOL_CRC_SIZE;
        let mut proto_buff_used = &mut proto_buff[0..proto_buff_size_used];
        assert!(
            proto_buff_size_used <= REGISTERS_PROTO_SIZE,
            "Protocol buffer size exceeded"
        ); 
        
        let (header, data_crc) = proto_buff_used.split_at_mut(spi_proto::PROTOCOL_HEADER_SIZE);

        spi_proto::populate_header(
            reg_id as u8,
            spi_proto::SpiPackOpType::Write,
            regs.len() as u16,
            header,
        );

        let (data, crc) = data_crc.split_at_mut(regs.len());
        data.copy_from_slice(regs);
        spi_proto::populate_crc(data, crc); 
        spi_proto::execute_spi_transaction(&mut self.spi_dev,
                                             &mut proto_buff_used);
        println!("Resiter id {:?} write request sent, error code: {:?}", reg_id, self.get_last_error_status());
    }

    

    pub fn get_all_registers(&mut self) -> DeviceFullInfo {
        let mut reg_dump: [u8; REGISTERS_SIZE_BYTES] = [0; REGISTERS_SIZE_BYTES];
        self.req_regs_dump(RegisterID::DeviceID, &mut reg_dump);
        let sys_info = Self::parse_system_info(&reg_dump);

        let mut motors_status = [MotorStatus::default(); DEVICE_SUPPORTED_MOTORS];
        let motors_data = &reg_dump
            [MOTORS_DATA_START_OFFSET..MOTORS_DATA_START_OFFSET + DEVICE_ALL_MOTORS_BLOCK_SIZE_BYTES_READ];
        for (motor_idx, motor_regs) in motors_data
            .chunks_exact(DEVICE_MOTOR_BLOCK_SIZE_BYTES_READ)
            .enumerate()
        {
            let motor_id = MotorID::try_from(motor_idx as u8).unwrap();
            motors_status[motor_idx] = self.populate_motor_status(motor_id, motor_regs);
        }

        let internal_loop_time_ms = Self::extract_u32_from_bytes(
            &reg_dump[INTERNAL_LOOP_TIME_OFFSET..INTERNAL_LOOP_TIME_OFFSET + DEVICE_REG_SIZE_BYTES],
        );

        DeviceFullInfo {
            system_info: sys_info,
            motors_status,
            internal_loop_time_ms,
            last_error_status: ErrorCode::NoError,
        }
    }

    pub fn set_all_registers(&mut self, device_cfg: &DeviceCfg) {
        self.set_motor_all(&device_cfg.motors_cfg);
        self.set_internal_loop_time_ms(device_cfg.internal_loop_time_ms);
    }

    pub fn get_device_info(&mut self) -> SystemInfo {
        let mut reg_dump: [u8; SYSTEM_INFO_SIZE] = [0; SYSTEM_INFO_SIZE];
        self.req_regs_dump(RegisterID::DeviceID, &mut reg_dump);

        Self::parse_system_info(&reg_dump)
    }

    pub fn get_motor_dump(&mut self, motor_num: MotorID) -> MotorStatus {
        let reg_id = RegisterID::from_motor_id(motor_num, device::MotorRegisterOffset::OperationMode);
        let mut reg_dump: [u8; DEVICE_MOTOR_BLOCK_SIZE_BYTES_READ] = [0; DEVICE_MOTOR_BLOCK_SIZE_BYTES_READ];
        self.req_regs_dump(reg_id, &mut reg_dump);
        self.populate_motor_status(motor_num, &reg_dump)
    }

    pub fn set_motor_cfg(&mut self, motor_num: MotorID, motor_cfg: &MotorCfg) {
        let mut regs_buff: [u8; DEVICE_MOTOR_BLOCK_COUNT_WRITE] = [0; DEVICE_MOTOR_BLOCK_COUNT_WRITE];
        let mut offset = 0;
        
        regs_buff[offset.. offset + DEVICE_REG_SIZE_BYTES].copy_from_slice(&(motor_cfg.mode as i32).to_le_bytes());
        offset += DEVICE_REG_SIZE_BYTES;

        regs_buff[offset.. offset + DEVICE_REG_SIZE_BYTES].copy_from_slice(&(motor_cfg.direction as i32).to_le_bytes());
        offset += DEVICE_REG_SIZE_BYTES;

        regs_buff[offset.. offset + DEVICE_REG_SIZE_BYTES]
            .copy_from_slice(&motor_cfg.pwm_duty_cycle.to_le_bytes());
        offset += DEVICE_REG_SIZE_BYTES;

        regs_buff[offset.. offset + DEVICE_REG_SIZE_BYTES]
            .copy_from_slice(&motor_cfg.counts_per_revolution.to_le_bytes());
        offset += DEVICE_REG_SIZE_BYTES;

        regs_buff[offset.. offset + DEVICE_REG_SIZE_BYTES]
            .copy_from_slice(&motor_cfg.pid_params.kp.to_le_bytes());
        offset += DEVICE_REG_SIZE_BYTES;

        regs_buff[offset.. offset + DEVICE_REG_SIZE_BYTES]
            .copy_from_slice(&motor_cfg.pid_params.ki.to_le_bytes());
        offset += DEVICE_REG_SIZE_BYTES;

        regs_buff[offset.. offset + DEVICE_REG_SIZE_BYTES]
            .copy_from_slice(&motor_cfg.pid_params.kd.to_le_bytes());
        offset += DEVICE_REG_SIZE_BYTES;

        regs_buff[offset.. offset + DEVICE_REG_SIZE_BYTES]
            .copy_from_slice(&motor_cfg.rpm_desired.to_le_bytes());
        let reg_id = RegisterID::from_motor_id(motor_num, device::MotorRegisterOffset::OperationMode);
        self.req_reg_write(reg_id, &regs_buff);

    }

    pub fn get_motor_dump_all(&mut self) -> [MotorStatus; DEVICE_SUPPORTED_MOTORS] {
        let mut motor_statuses = [MotorStatus::default(); DEVICE_SUPPORTED_MOTORS];
        let mut reg_dump: [u8; DEVICE_ALL_MOTORS_BLOCK_SIZE_BYTES_READ] = [0; DEVICE_ALL_MOTORS_BLOCK_SIZE_BYTES_READ];
        self.req_regs_dump(RegisterID::Motor1OperationMode, &mut reg_dump);

        for (motor_idx, motor_regs) in reg_dump
            .chunks_exact(DEVICE_MOTOR_BLOCK_SIZE_BYTES_READ)
            .enumerate()
        {
            let motor_id = MotorID::try_from(motor_idx as u8).unwrap();
            motor_statuses[motor_idx] = self.populate_motor_status(motor_id, motor_regs);
        }
        motor_statuses
    }

    pub fn set_motor_all(&mut self, motor_cfgs: &[MotorCfg; DEVICE_SUPPORTED_MOTORS]) {
        for (motor_idx, motor_cfg) in motor_cfgs.iter().enumerate() {
            let motor_id = MotorID::try_from(motor_idx as u8).unwrap();
            self.set_motor_cfg(motor_id, motor_cfg);
        }
    }

    pub fn get_internal_loop_time_ms(&mut self) -> i32 {
        let mut reg_dump: [u8; DEVICE_REG_SIZE_BYTES] = [0; DEVICE_REG_SIZE_BYTES];
        self.req_regs_dump(RegisterID::InternalLoopTime, &mut reg_dump);
        Self::extract_u32_from_bytes(&reg_dump)
    }

    pub fn set_internal_loop_time_ms(&mut self, loop_time_ms: i32) {
        let reg_data = loop_time_ms.to_le_bytes();
        self.req_reg_write(RegisterID::InternalLoopTime, &reg_data);
    }

    pub fn get_last_error_status(&mut self) -> ErrorCode {
        let mut reg_dump: [u8; DEVICE_REG_SIZE_BYTES] = [0; DEVICE_REG_SIZE_BYTES];
        self.req_regs_dump(RegisterID::LastErrorStatus, &mut reg_dump);
        ErrorCode::try_from(Self::extract_u32_from_bytes(&reg_dump)).unwrap()
    }

    pub fn set_pid_params(&mut self, motor_num: MotorID, pid_params: &PidParams) {
        let mut pids_params_buff = [0u8; DEVICE_PID_PARAMS_SIZE_BYTES];
        let mut offset = 0;
        pids_params_buff[offset.. offset + DEVICE_REG_SIZE_BYTES]
            .copy_from_slice(&pid_params.kp.to_le_bytes());
        offset += DEVICE_REG_SIZE_BYTES;
        pids_params_buff[offset.. offset + DEVICE_REG_SIZE_BYTES]
            .copy_from_slice(&pid_params.ki.to_le_bytes());
        offset += DEVICE_REG_SIZE_BYTES;
        pids_params_buff[offset.. offset + DEVICE_REG_SIZE_BYTES]
            .copy_from_slice(&pid_params.kd.to_le_bytes());

        let reg_id_kp = RegisterID::from_motor_id(motor_num, MotorRegisterOffset::PIDKp);
        self.req_reg_write(reg_id_kp, &pids_params_buff);
    }

    pub fn set_pid_params_all(&mut self, pid_params_list: &[PidParams; DEVICE_SUPPORTED_MOTORS]) {
        for (motor_idx, pid_params) in pid_params_list.iter().enumerate() {
            let motor_id = MotorID::try_from(motor_idx as u8).unwrap();
            self.set_pid_params(motor_id, pid_params);
        }
    }

    pub fn print_full_device_info(dev_info: &DeviceFullInfo) { 
        println!("Device full info:");
        println!("----------------");
        println!("Device ID: {}", dev_info.system_info.device_id);
        println!("Firmware Version: {}", dev_info.system_info.firmware_version);
        println!("Internal Loop Time: {}", dev_info.internal_loop_time_ms);
        println!("Last Error Status: {:?}", dev_info.last_error_status);
        println!("----------------");
        for (idx, status) in dev_info.motors_status.iter().enumerate() {
            let motor_num = idx + 1;
            println!("Motor {} Status:", motor_num);
            println!("  [{}]Control Mode: {:?}", motor_num, status.mode);
            println!("  [{}]Direction: {:?}", motor_num, status.direction);
            println!("  [{}]PWM Duty Cycle: {}", motor_num, status.pwm_duty_cycle);
            println!(
                "  [{}]Counts per Revolution: {}",
                motor_num, status.counts_per_revolution
            );
            println!("  [{}]RPM Current: {}", motor_num, status.rpm_current);
            println!("  [{}]RPM Desired: {}", motor_num, status.rpm_desired);
            println!(
                "  [{}]PID Params: Kp={}, Ki={}, Kd={}",
                motor_num, status.pid_params.kp, status.pid_params.ki, status.pid_params.kd
            );
            println!("----------------");
        }
    }
}
