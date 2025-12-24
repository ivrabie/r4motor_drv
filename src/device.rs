/*
 ## Regs
| Reg(1byte) | Description |  Default Value (4bytes) | Access | Detailed description |
|-------------|-------------|----------------|--------|----------------------|
| 0x00 | Device ID | "4MOT" | RO | Unique identifier for the device |
| 0x01 | Firmware version | "x.x.x" | RO | Current firmware version |
| 0x02 | Motor1 Control Mode  | 0 | RW | 0: Pwm control, 1: Rpm control |
| 0x03 | Motor1 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x04 | Motor1 pwm duty cycle | 0 | RW | PWM duty cycle for Motor1 |
| 0x05 | Motor1 counts per revolution | 0 | RW | Counts per revolution for Motor1 |
| 0x06 | Motor1 rpm current | 0 | R | Current rpm for Motor1 |
| 0x07 | Motor1 rpm desired | 0 | RW | Desired rpm for Motor1 |
| 0x08 | Motor1 pid kp | 0 | RW | PID Kp for Motor1 |
| 0x09 | Motor1 pid ki | 0 | RW | PID Ki for Motor1 |
| 0x0A | Motor1 pid kd | 0 | RW | PID Kd for Motor1 |
| 0x0B | Motor2 Control Mode  | 0 | RW | 0: Pwm control, 1: Rpm control |
| 0x0C | Motor2 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x0D | Motor2 pwm duty cycle | 0 | RW | PWM duty cycle for Motor2 |
| 0x0E | Motor2 counts per revolution | 0 | RW | Counts per revolution for Motor2 |
| 0x0F | Motor2 rpm current | 0 | R | Current rpm for Motor2 |
| 0x10 | Motor2 rpm desired | 0 | RW | Desired rpm for Motor2 |
| 0x11 | Motor2 pid kp | 0 | RW | PID Kp for Motor2 |
| 0x12 | Motor2 pid ki | 0 | RW | PID Ki for Motor2 |
| 0x13 | Motor2 pid kd | 0 | RW | PID Kd for Motor2 |
| 0x14 | Motor3 Control Mode  | 0 | RW | 0: Pwm control, 1: Rpm control |
| 0x15 | Motor3 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x16 | Motor3 pwm duty cycle | 0 | RW | PWM duty cycle for Motor3 |
| 0x17 | Motor3 counts per revolution | 0 | RW | Counts per revolution for Motor3 |
| 0x18 | Motor3 rpm current | 0 | R | Current rpm for Motor3 |
| 0x19 | Motor3 rpm desired | 0 | RW | Desired rpm for Motor3 |
| 0x1A | Motor3 pid kp | 0 | RW | PID Kp for Motor3 |
| 0x1B | Motor3 pid ki | 0 | RW | PID Ki for Motor3 |
| 0x1C | Motor3 pid kd | 0 | RW | PID Kd for Motor3 |
| 0x1D | Motor4 Control Mode  | 0 | RW | 0: Pwm control, 1: Rpm control |
| 0x1E | Motor4 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x1F | Motor4 pwm duty cycle | 0 | RW | PWM duty cycle for Motor4 |
| 0x20 | Motor4 counts per revolution | 0 | RW | Counts per revolution for Motor4 |
| 0x21 | Motor4 rpm current | 0 | R | Current rpm for Motor4 |
| 0x22 | Motor4 rpm desired | 0 | RW | Desired rpm for Motor4 |
| 0x23 | Motor4 pid kp | 0 | RW | PID Kp for Motor4 |
| 0x24 | Motor4 pid ki | 0 | RW | PID Ki for Motor4 |
| 0x25 | Motor4 pid kd | 0 | RW | PID Kd for Motor4 |
| 0x26 | Internal loop time | 0 | RW | Internal loop time |
| 0x27 | Last error status | 0 | RW | Last error status 1 byte | reg 1 byte | 2 reserved |
### Errors
| Error code | Description |
|-------------|-------------|
| 0x00 | No error |
| 0x01 | Invalid register address |
| 0x02 | Invalid request length |
| 0x03 | CRC mismatch |
| 0x04 | Invalid control mode |
| 0x05 | Write not allowed in this control mode |*/

use linux_embedded_hal::{
    spidev::{SpiModeFlags, SpidevOptions},
    SpidevDevice,
};
use crate::spi_proto;
use num_enum::TryFromPrimitive;

pub const REGISTERS_COUNT: usize = 40;
pub const REGISTERS_SIZE_BYTES: usize = REGISTERS_COUNT * 4;
pub const REGISTERS_PROTO_SIZE: usize = REGISTERS_SIZE_BYTES + spi_proto::PROTOCOL_OVERHEAD;
pub const DEVICE_SUPPORTED_MOTORS: usize = 4;
pub const DEVICE_MOTOR_BLOCK_COUNT: usize = 9; // Number of registers per motor
pub const DEVICE_REG_SIZE_BYTES: usize = 4; // Each register is 4 bytes
pub const DEVICE_MOTOR_BLOCK_SIZE_BYTES: usize = DEVICE_MOTOR_BLOCK_COUNT * DEVICE_REG_SIZE_BYTES; // Size of each motor block in bytes
pub const DEVICE_ALL_MOTORS_BLOCK_SIZE_BYTES: usize = DEVICE_SUPPORTED_MOTORS * DEVICE_MOTOR_BLOCK_SIZE_BYTES;

// New global constants for  dump processing
pub const DEVICE_ID_REG_SIZE: usize = 4;
pub const FIRMWARE_VERSION_REG_SIZE: usize = 4;
pub const SYSTEM_INFO_SIZE: usize = DEVICE_ID_REG_SIZE + FIRMWARE_VERSION_REG_SIZE;
pub const MOTORS_DATA_START_OFFSET: usize = SYSTEM_INFO_SIZE;
pub const INTERNAL_LOOP_TIME_OFFSET: usize = MOTORS_DATA_START_OFFSET + DEVICE_ALL_MOTORS_BLOCK_SIZE_BYTES;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
pub enum ControlMode {
    Pwm = 0,
    Rpm = 1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
pub enum MotorDirection {
    Stop = 0,
    Forward = 1,
    Backward = 2,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
pub enum ErrorCode {
    NoError = 0x00,
    InvalidRegisterAddress = 0x01,
    InvalidRequestLength = 0x02,
    CRCMismatch = 0x03,
    InvalidControlMode = 0x04,
    WriteNotAllowedInThisControlMode = 0x05,
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
    Motor1RPMCurrent = 0x06,
    Motor1RPMDesired = 0x07,
    Motor1PIDKp = 0x08,
    Motor1PIDKi = 0x09,
    Motor1PIDKd = 0x0A,
    Motor2OperationMode = 0x0B,
    Motor2Direction = 0x0C,
    Motor2PWMDutyCycle = 0x0D,
    Motor2CountsPerRevolution = 0x0E,
    Motor2RPMCurrent = 0x0F,
    Motor2RPMDesired = 0x10,
    Motor2PIDKp = 0x11,
    Motor2PIDKi = 0x12,
    Motor2PIDKd = 0x13,
    Motor3OperationMode = 0x14,
    Motor3Direction = 0x15,
    Motor3PWMDutyCycle = 0x16,
    Motor3CountsPerRevolution = 0x17,
    Motor3RPMCurrent = 0x18,
    Motor3RPMDesired = 0x19,
    Motor3PIDKp = 0x1A,
    Motor3PIDKi = 0x1B,
    Motor3PIDKd = 0x1C,
    Motor4OperationMode = 0x1D,
    Motor4Direction = 0x1E,
    Motor4PWMDutyCycle = 0x1F,
    Motor4CountsPerRevolution = 0x20,
    Motor4RPMCurrent = 0x21,
    Motor4RPMDesired = 0x22,
    Motor4PIDKp = 0x23,
    Motor4PIDKi = 0x24,
    Motor4PIDKd = 0x25,
    InternalLoopTime = 0x26,
    LastErrorStatus = 0x27,
}

pub enum RegisterData {
    DeviceID(String),
    FirmwareVersion(String),
    OperationMode(ControlMode),
    MotorDirection(MotorDirection),
    MotorPWMDutyCycle(u32),
    MotorCountsPerRevolution(u32),
    MotorRPM(u32),
    MotorPIDParam(u32),
    InternalLoopTime(u32),
    LastErrorStatus(ErrorCode),
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
    pub kp: u32,
    pub ki: u32,
    pub kd: u32,
}


pub struct MotorCfg {
    pub mode: ControlMode,
    pub direction: MotorDirection,
    pub pwm_duty_cycle: u32,
    pub counts_per_revolution: u32,
    pub rpm_desired: u32,
    pub pid_params: PidParams,
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
    pub pwm_duty_cycle: u32,
    pub counts_per_revolution: u32,
    pub rpm_current: u32,
    pub rpm_desired: u32,
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
    pub internal_loop_time_ms: u32,
    pub last_error_status: ErrorCode,
}

pub struct Device {
    spi_dev: SpidevDevice,
}

impl RegisterID {
    pub fn to_motor_num(&self) -> Option<MotorID> {
        let first = RegisterID::Motor1OperationMode as u8;
        let last = RegisterID::Motor4PIDKd as u8;
        let value = *self as u8;
        if value < first || value > last {
            return None;
        }

        let index = value - first;
        let motor_idx = index / DEVICE_MOTOR_BLOCK_COUNT as u8;
        MotorID::try_from(motor_idx).ok()
    }

    pub fn from_motor_id(motor_id: &MotorID, offset: u8) -> Self {
        debug_assert!(
            (offset as usize) < DEVICE_MOTOR_BLOCK_COUNT,
            "motor register offset out of range: {}",
            offset
        );
        let base = RegisterID::Motor1OperationMode as u8
            + (*motor_id as u8) * DEVICE_MOTOR_BLOCK_COUNT as u8
            + offset;
        RegisterID::try_from(base).unwrap()
    }
}

impl Device {
    pub fn new(spi_if: &str) -> Self {
        let mut spi = SpidevDevice::open(spi_if).expect("Failed to open");
        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(500_000) // 500 kHz
            .mode(SpiModeFlags::SPI_MODE_0)
            .lsb_first(false)
            .build();
        debug_assert_eq!(options.lsb_first, Some(false));
        spi.configure(&options).expect("Failed to configure");

        Device {
            spi_dev: spi,
        }
    }

    fn extract_u32_from_bytes(data: &[u8]) -> u32 {
        debug_assert!(
            data.len() >= DEVICE_REG_SIZE_BYTES,
            "expected at least {} bytes",
            DEVICE_REG_SIZE_BYTES
        );
        u32::from_le_bytes([data[0], data[1], data[2], data[3]])
    }

    fn parse_system_info(reg_dump: &[u8]) -> SystemInfo {
        debug_assert!(
            reg_dump.len() >= SYSTEM_INFO_SIZE,
            "system info dump length mismatch"
        );
        let device_id = String::from_utf8_lossy(&reg_dump[0..DEVICE_ID_REG_SIZE]).to_string();
        let firmware_version = format!(
            "{:02}.{:02}",
            reg_dump[DEVICE_ID_REG_SIZE],
            reg_dump[DEVICE_ID_REG_SIZE + 1]
        );

        SystemInfo {
            device_id,
            firmware_version,
        }
    }

    fn populate_motor_status(&mut self, motor_num: MotorID, data: &[u8]) -> MotorStatus {
        debug_assert_eq!(
            data.len(),
            DEVICE_MOTOR_BLOCK_SIZE_BYTES,
            "motor {:?} register dump length mismatch",
            motor_num
        );

        let mut status = MotorStatus::default();
        let mut regs = data.chunks_exact(DEVICE_REG_SIZE_BYTES);

        status.mode = ControlMode::try_from(regs.next().unwrap()[0]).unwrap();
        status.direction = MotorDirection::try_from(regs.next().unwrap()[0]).unwrap();
        status.pwm_duty_cycle = Self::extract_u32_from_bytes(regs.next().unwrap());
        status.counts_per_revolution = Self::extract_u32_from_bytes(regs.next().unwrap());
        status.rpm_current = Self::extract_u32_from_bytes(regs.next().unwrap());
        status.rpm_desired = Self::extract_u32_from_bytes(regs.next().unwrap());
        status.pid_params.kp = Self::extract_u32_from_bytes(regs.next().unwrap());
        status.pid_params.ki = Self::extract_u32_from_bytes(regs.next().unwrap());
        status.pid_params.kd = Self::extract_u32_from_bytes(regs.next().unwrap());

        status
    }

    fn req_regs_dump(&mut self, reg_id: RegisterID, reg_dump: &mut [u8]) {
        assert!(
            reg_dump.len() + spi_proto::PROTOCOL_OVERHEAD <= REGISTERS_PROTO_SIZE,
            "Requested length exceeds register dump size"
        );
        // Placeholder for actual implementation to request register dump from device
        let mut register_dump: [u8; REGISTERS_PROTO_SIZE] = [0; REGISTERS_PROTO_SIZE];
        let (header_slice, crc_slice) = register_dump.split_at_mut(spi_proto::PROTOCOL_DATA_OFFSET);
        let crc_slice = &mut crc_slice[0..spi_proto::PROTOCOL_CRC_SIZE];
        spi_proto::populate_header(
            reg_id as u8,
            spi_proto::SpiPackOpType::Read,
            reg_dump.len() as u16,
            header_slice,
        );
        spi_proto::populate_crc(header_slice, crc_slice);
        // Here you would send the request via SPI and read the response
        println!("Register dump requested.");
        spi_proto::execute_spi_transaction(&mut self.spi_dev, &mut register_dump);

        // Process the received register dump
        println!("Register dump received.");
        let recv_crc_start_idx = reg_dump.len() + spi_proto::PROTOCOL_OVERHEAD;
        let recv_crc = u16::from_le_bytes([
            register_dump[recv_crc_start_idx],
            register_dump[recv_crc_start_idx + 1],
        ]);
        let is_valid_crc = spi_proto::validate_crc(&register_dump[0..recv_crc_start_idx], recv_crc);
        if is_valid_crc {
            println!("CRC validation passed.");
            reg_dump
                .copy_from_slice(&register_dump[spi_proto::PROTOCOL_OVERHEAD..recv_crc_start_idx]);
        } else {
            println!("CRC validation failed.");
        }
    }

    fn get_all_registers(&mut self) -> DeviceFullInfo {
        let mut reg_dump: [u8; REGISTERS_SIZE_BYTES] = [0; REGISTERS_SIZE_BYTES];
        self.req_regs_dump(RegisterID::DeviceID, &mut reg_dump);
        let sys_info = Self::parse_system_info(&reg_dump);

        let mut motors_status = [MotorStatus::default(); DEVICE_SUPPORTED_MOTORS];
        let motors_data = &reg_dump
            [MOTORS_DATA_START_OFFSET..MOTORS_DATA_START_OFFSET + DEVICE_ALL_MOTORS_BLOCK_SIZE_BYTES];
        for (motor_idx, motor_regs) in motors_data
            .chunks_exact(DEVICE_MOTOR_BLOCK_SIZE_BYTES)
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

    pub fn get_device_info(&mut self) -> SystemInfo {
        let mut reg_dump: [u8; SYSTEM_INFO_SIZE] = [0; SYSTEM_INFO_SIZE];
        self.req_regs_dump(RegisterID::DeviceID, &mut reg_dump);

        Self::parse_system_info(&reg_dump)
    }

    fn get_motor_dump(&mut self, motor_num: MotorID) -> MotorStatus {
        let reg_id = RegisterID::from_motor_id(&motor_num, 0);
        let mut reg_dump: [u8; DEVICE_MOTOR_BLOCK_SIZE_BYTES] = [0; DEVICE_MOTOR_BLOCK_SIZE_BYTES];
        self.req_regs_dump(reg_id, &mut reg_dump);
        self.populate_motor_status(motor_num, &reg_dump)
    }

    fn get_motor_dump_all(&mut self) -> [MotorStatus; DEVICE_SUPPORTED_MOTORS] {
        let mut motor_statuses = [MotorStatus::default(); DEVICE_SUPPORTED_MOTORS];
        let mut reg_dump: [u8; DEVICE_ALL_MOTORS_BLOCK_SIZE_BYTES] = [0; DEVICE_ALL_MOTORS_BLOCK_SIZE_BYTES];
        self.req_regs_dump(RegisterID::Motor1OperationMode, &mut reg_dump);

        for (motor_idx, motor_regs) in reg_dump
            .chunks_exact(DEVICE_MOTOR_BLOCK_SIZE_BYTES)
            .enumerate()
        {
            let motor_id = MotorID::try_from(motor_idx as u8).unwrap();
            motor_statuses[motor_idx] = self.populate_motor_status(motor_id, motor_regs);
        }
        motor_statuses
    }

    fn get_internal_loop_time_ms(&mut self) -> u32 {
        let mut reg_dump: [u8; DEVICE_REG_SIZE_BYTES] = [0; DEVICE_REG_SIZE_BYTES];
        self.req_regs_dump(RegisterID::InternalLoopTime, &mut reg_dump);
        Self::extract_u32_from_bytes(&reg_dump)
    }

    fn get_last_error_status(&mut self) -> ErrorCode {
        let mut reg_dump: [u8; DEVICE_REG_SIZE_BYTES] = [0; DEVICE_REG_SIZE_BYTES];
        self.req_regs_dump(RegisterID::LastErrorStatus, &mut reg_dump);
        ErrorCode::try_from(Self::extract_u32_from_bytes(&reg_dump)).unwrap()
    }

    pub fn print_full_device_info(dev_info: &mut DeviceFullInfo) { 
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
