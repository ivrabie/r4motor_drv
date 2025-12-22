/*
 ## Regs
| Reg(1byte) | Description |  Default Value (4bytes) | Access | Detailed description |
|-------------|-------------|----------------|--------|----------------------|
| 0x00 | Device ID | "4MOT" | RO | Unique identifier for the device |
| 0x01 | Firmware version | "x.x.x" | RO | Current firmware version |
| 0x02 | Control Mode  | 0 | RW | 0: Position control, 1: Velocity control |
| 0x03 | Motor1 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x04 | Motor2 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x05 | Motor3 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x06 | Motor4 direction  | 0 | RW | 0: Stop, 1: Forward, 2: Backward |
| 0x07 | Motor1 pwm duty cycle | 0 | RW | PWM duty cycle for Motor1 |
| 0x08 | Motor2 pwm duty cycle | 0 | RW | PWM duty cycle for Motor2 |
| 0x09 | Motor3 pwm duty cycle | 0 | RW | PWM duty cycle for Motor3 |
| 0x0A | Motor4 pwm duty cycle | 0 | RW | PWM duty cycle for Motor4 |
| 0x0B | Motor1 counts per revolution | 0 | RW | Counts per revolution for Motor1 |
| 0x0C | Motor2 counts per revolution | 0 | RW | Counts per revolution for Motor2 |
| 0x0D | Motor3 counts per revolution | 0 | RW | Counts per revolution for Motor3 |
| 0x0E | Motor4 counts per revolution | 0 | RW | Counts per revolution for Motor4 |
| 0x0F | Motor1 rpm current | 0 | R | Current rpm for Motor1 |
| 0x10 | Motor2 rpm current | 0 | R | Current rpm for Motor2 |
| 0x11 | Motor3 rpm current | 0 | R | Current rpm for Motor3 |
| 0x12 | Motor4 rpm current | 0 | R | Current rpm for Motor4 |
| 0x13 | Motor1 rpm desired | 0 | RW | Desired rpm for Motor1 |
| 0x14 | Motor2 rpm desired | 0 | RW | Desired rpm for Motor2 |
| 0x15 | Motor3 rpm desired | 0 | RW | Desired rpm for Motor3 |
| 0x16 | Motor4 rpm desired | 0 | RW | Desired rpm for Motor4 |
| 0x17 | Motor1 pid kp | 0 | RW | PID Kp for Motor1 |
| 0x18 | Motor1 pid ki | 0 | RW | PID Ki for Motor1 |
| 0x19 | Motor1 pid kd | 0 | RW | PID Kd for Motor1 |
| 0x1A | Motor2 pid kp | 0 | RW | PID Kp for Motor2 |
| 0x1B | Motor2 pid ki | 0 | RW | PID Ki for Motor2 |
| 0x1C | Motor2 pid kd | 0 | RW | PID Kd for Motor2 |
| 0x1D | Motor3 pid kp | 0 | RW | PID Kp for Motor3 |
| 0x1E | Motor3 pid ki | 0 | RW | PID Ki for Motor3 |
| 0x1F | Motor3 pid kd | 0 | RW | PID Kd for Motor3 |
| 0x20 | Motor4 pid kp | 0 | RW | PID Kp for Motor4 |
| 0x21 | Motor4 pid ki | 0 | RW | PID Ki for Motor4 |
| 0x22 | Motor4 pid kd | 0 | RW | PID Kd for Motor4 |
| 0x23 | Internal loop time | 0 | RW | Internal loop time |
| 0x24 | Last error status | 0 | RW | Last error status 1 byte | reg 1 byte | 2 reserved |
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
use std::collections::HashMap;

use crate::spi_proto;

pub const REGISTERS_COUNT: usize = 37;
pub const REGISTERS_SIZE_BYTES: usize = REGISTERS_COUNT * 4;
pub const REGISTERS_PROTO_SIZE: usize = REGISTERS_SIZE_BYTES + spi_proto::PROTOCOL_OVERHEAD;
pub const DEVICE_SUPPORTED_MOTORS: usize = 4;
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMode {
    Position = 0,
    Velocity = 1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotorDirection {
    Stop = 0,
    Forward = 1,
    Backward = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    NoError = 0x00,
    InvalidRegisterAddress = 0x01,
    InvalidRequestLength = 0x02,
    CRCMismatch = 0x03,
    InvalidControlMode = 0x04,
    WriteNotAllowedInThisControlMode = 0x05,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RegisterID {
    DeviceID = 0x00,
    FirmwareVersion = 0x01,
    OperationMode = 0x02,
    Motor1Direction = 0x03,
    Motor2Direction = 0x04,
    Motor3Direction = 0x05,
    Motor4Direction = 0x06,
    Motor1PWMDutyCycle = 0x07,
    Motor2PWMDutyCycle = 0x08,
    Motor3PWMDutyCycle = 0x09,
    Motor4PWMDutyCycle = 0x0A,
    Motor1CountsPerRevolution = 0x0B,
    Motor2CountsPerRevolution = 0x0C,
    Motor3CountsPerRevolution = 0x0D,
    Motor4CountsPerRevolution = 0x0E,
    Motor1RPMCurrent = 0x0F,
    Motor2RPMCurrent = 0x10,
    Motor3RPMCurrent = 0x11,
    Motor4RPMCurrent = 0x12,
    Motor1RPMDesired = 0x13,
    Motor2RPMDesired = 0x14,
    Motor3RPMDesired = 0x15,
    Motor4RPMDesired = 0x16,
    Motor1PIDKp = 0x17,
    Motor1PIDKi = 0x18,
    Motor1PIDKd = 0x19,
    Motor2PIDKp = 0x1A,
    Motor2PIDKi = 0x1B,
    Motor2PIDKd = 0x1C,
    Motor3PIDKp = 0x1D,
    Motor3PIDKi = 0x1E,
    Motor3PIDKd = 0x1F,
    Motor4PIDKp = 0x20,
    Motor4PIDKi = 0x21,
    Motor4PIDKd = 0x22,
    InternalLoopTime = 0x23,
    LastErrorStatus = 0x24,
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

pub enum PidParamType {
    Kp,
    Ki,
    Kd,
}

pub struct PidParams {
    pub kp: u32,
    pub ki: u32,
    pub kd: u32,
}

pub struct Device {
    spi_dev: SpidevDevice,
    device_id: String,
    firmware_version: String,
    control_mode: ControlMode,
    motor_dir: [MotorDirection; 4],
    pwm_duty_cycle: [u32; 4],
    register_raw: [u32; REGISTERS_COUNT],
    register_map: HashMap<RegisterID, RegisterData>,
}

impl TryFrom<u8> for ControlMode {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ControlMode::Position),
            1 => Ok(ControlMode::Velocity),
            _ => Err("Invalid ControlMode value"),
        }
    }
}

impl TryFrom<u8> for MotorDirection {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MotorDirection::Stop),
            1 => Ok(MotorDirection::Forward),
            2 => Ok(MotorDirection::Backward),
            _ => Err("Invalid MotorDirection value"),
        }
    }
}

impl TryFrom<u8> for ErrorCode {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(ErrorCode::NoError),
            0x01 => Ok(ErrorCode::InvalidRegisterAddress),
            0x02 => Ok(ErrorCode::InvalidRequestLength),
            0x03 => Ok(ErrorCode::CRCMismatch),
            0x04 => Ok(ErrorCode::InvalidControlMode),
            0x05 => Ok(ErrorCode::WriteNotAllowedInThisControlMode),
            _ => Err("Invalid ErrorCode value"),
        }
    }
}

impl TryFrom<u8> for RegisterID {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(RegisterID::DeviceID),
            0x01 => Ok(RegisterID::FirmwareVersion),
            0x02 => Ok(RegisterID::OperationMode),
            0x03 => Ok(RegisterID::Motor1Direction),
            0x04 => Ok(RegisterID::Motor2Direction),
            0x05 => Ok(RegisterID::Motor3Direction),
            0x06 => Ok(RegisterID::Motor4Direction),
            0x07 => Ok(RegisterID::Motor1PWMDutyCycle),
            0x08 => Ok(RegisterID::Motor2PWMDutyCycle),
            0x09 => Ok(RegisterID::Motor3PWMDutyCycle),
            0x0A => Ok(RegisterID::Motor4PWMDutyCycle),
            0x0B => Ok(RegisterID::Motor1CountsPerRevolution),
            0x0C => Ok(RegisterID::Motor2CountsPerRevolution),
            0x0D => Ok(RegisterID::Motor3CountsPerRevolution),
            0x0E => Ok(RegisterID::Motor4CountsPerRevolution),
            0x0F => Ok(RegisterID::Motor1RPMCurrent),
            0x10 => Ok(RegisterID::Motor2RPMCurrent),
            0x11 => Ok(RegisterID::Motor3RPMCurrent),
            0x12 => Ok(RegisterID::Motor4RPMCurrent),
            0x13 => Ok(RegisterID::Motor1RPMDesired),
            0x14 => Ok(RegisterID::Motor2RPMDesired),
            0x15 => Ok(RegisterID::Motor3RPMDesired),
            0x16 => Ok(RegisterID::Motor4RPMDesired),
            0x17 => Ok(RegisterID::Motor1PIDKp),
            0x18 => Ok(RegisterID::Motor1PIDKi),
            0x19 => Ok(RegisterID::Motor1PIDKd),
            0x1A => Ok(RegisterID::Motor2PIDKp),
            0x1B => Ok(RegisterID::Motor2PIDKi),
            0x1C => Ok(RegisterID::Motor2PIDKd),
            0x1D => Ok(RegisterID::Motor3PIDKp),
            0x1E => Ok(RegisterID::Motor3PIDKi),
            0x1F => Ok(RegisterID::Motor3PIDKd),
            0x20 => Ok(RegisterID::Motor4PIDKp),
            0x21 => Ok(RegisterID::Motor4PIDKi),
            0x22 => Ok(RegisterID::Motor4PIDKd),
            0x23 => Ok(RegisterID::InternalLoopTime),
            0x24 => Ok(RegisterID::LastErrorStatus),
            _ => Err("Invalid RegisterID value"),
        }
    }
}



fn create_register_map() -> HashMap<RegisterID, RegisterData> {
    use RegisterData as RD;
    use RegisterID::*;

    HashMap::from([
        // Device info registers
        (DeviceID, RD::DeviceID("None".to_string())),
        (FirmwareVersion, RD::FirmwareVersion("0.0.00".to_string())),
        (OperationMode, RD::OperationMode(ControlMode::Position)),
        // Motor 1 registers
        (Motor1Direction, RD::MotorDirection(MotorDirection::Stop)),
        (Motor1PWMDutyCycle, RD::MotorPWMDutyCycle(0)),
        (Motor1CountsPerRevolution, RD::MotorCountsPerRevolution(0)),
        (Motor1RPMCurrent, RD::MotorRPM(0)),
        (Motor1RPMDesired, RD::MotorRPM(0)),
        (Motor1PIDKp, RD::MotorPIDParam(0)),
        (Motor1PIDKi, RD::MotorPIDParam(0)),
        (Motor1PIDKd, RD::MotorPIDParam(0)),
        // Motor 2 registers
        (Motor2Direction, RD::MotorDirection(MotorDirection::Stop)),
        (Motor2PWMDutyCycle, RD::MotorPWMDutyCycle(0)),
        (Motor2CountsPerRevolution, RD::MotorCountsPerRevolution(0)),
        (Motor2RPMCurrent, RD::MotorRPM(0)),
        (Motor2RPMDesired, RD::MotorRPM(0)),
        (Motor2PIDKp, RD::MotorPIDParam(0)),
        (Motor2PIDKi, RD::MotorPIDParam(0)),
        (Motor2PIDKd, RD::MotorPIDParam(0)),
        // Motor 3 registers
        (Motor3Direction, RD::MotorDirection(MotorDirection::Stop)),
        (Motor3PWMDutyCycle, RD::MotorPWMDutyCycle(0)),
        (Motor3CountsPerRevolution, RD::MotorCountsPerRevolution(0)),
        (Motor3RPMCurrent, RD::MotorRPM(0)),
        (Motor3RPMDesired, RD::MotorRPM(0)),
        (Motor3PIDKp, RD::MotorPIDParam(0)),
        (Motor3PIDKi, RD::MotorPIDParam(0)),
        (Motor3PIDKd, RD::MotorPIDParam(0)),
        // Motor 4 registers
        (Motor4Direction, RD::MotorDirection(MotorDirection::Stop)),
        (Motor4PWMDutyCycle, RD::MotorPWMDutyCycle(0)),
        (Motor4CountsPerRevolution, RD::MotorCountsPerRevolution(0)),
        (Motor4RPMCurrent, RD::MotorRPM(0)),
        (Motor4RPMDesired, RD::MotorRPM(0)),
        (Motor4PIDKp, RD::MotorPIDParam(0)),
        (Motor4PIDKi, RD::MotorPIDParam(0)),
        (Motor4PIDKd, RD::MotorPIDParam(0)),
        // System registers
        (InternalLoopTime, RD::InternalLoopTime(0)),
        (LastErrorStatus, RD::LastErrorStatus(ErrorCode::NoError)),
    ])
}

impl Device {
    pub fn new(spi_if: &str) -> Self {
        let mut spi = SpidevDevice::open(spi_if).expect("Failed to open");
        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(500_000) // 1 MHz
            .mode(SpiModeFlags::SPI_MODE_0)
            .lsb_first(false)
            .build();
        println!("{}", options.lsb_first.unwrap());
        spi.configure(&options).expect("Failed to configure");

        Device {
            spi_dev: spi,
            register_raw: [0; REGISTERS_COUNT],
            register_map: create_register_map(),
        }
    }

    fn populate_register_map(&mut self, reg_id: RegisterID, data: &[u8]) {
        use RegisterData as RD;
        use RegisterID::*;
        match reg_id {
            DeviceID => {
                let id_str = String::from_utf8_lossy(data).to_string();
                self.register_map.insert(reg_id, RD::DeviceID(id_str));
            }
            FirmwareVersion => {
                let ver_str = format!("{}{}.{}{}", data[0], data[1], data[2], data[3]);
                self.register_map
                    .insert(reg_id, RD::FirmwareVersion(ver_str));
            }
            OperationMode => {
                let mode = ControlMode::try_from(data[0]).unwrap();
                self.register_map.insert(reg_id, RD::OperationMode(mode));
            }
            Motor1Direction | Motor2Direction | Motor3Direction | Motor4Direction => {
                let dir = MotorDirection::try_from(data[0]).unwrap();
                self.register_map.insert(reg_id, RD::MotorDirection(dir));
            }
            Motor1PWMDutyCycle | Motor2PWMDutyCycle | Motor3PWMDutyCycle | Motor4PWMDutyCycle => {
                let value = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                self.register_map
                    .insert(reg_id, RD::MotorPWMDutyCycle(value));
            }
            Motor1CountsPerRevolution
            | Motor2CountsPerRevolution
            | Motor3CountsPerRevolution
            | Motor4CountsPerRevolution => {
                let value = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                self.register_map
                    .insert(reg_id, RD::MotorCountsPerRevolution(value));
            }
            Motor1RPMCurrent | Motor2RPMCurrent | Motor3RPMCurrent | Motor4RPMCurrent
            | Motor1RPMDesired | Motor2RPMDesired | Motor3RPMDesired | Motor4RPMDesired => {
                let value = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                self.register_map.insert(reg_id, RD::MotorRPM(value));
            }
            Motor1PIDKp | Motor2PIDKp | Motor3PIDKp | Motor4PIDKp | Motor1PIDKi | Motor2PIDKi
            | Motor3PIDKi | Motor4PIDKi | Motor1PIDKd | Motor2PIDKd | Motor3PIDKd | Motor4PIDKd => {
                let value = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                self.register_map.insert(reg_id, RD::MotorPIDParam(value));
            }
            InternalLoopTime => {
                let value = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                self.register_map
                    .insert(reg_id, RD::InternalLoopTime(value));
            }
            LastErrorStatus => {
                let err = ErrorCode::try_from(data[0]).unwrap();
                self.register_map.insert(reg_id, RD::LastErrorStatus(err));
            }
        }
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

    fn req_all_registers(&mut self) {
        let mut reg_dump: [u8; REGISTERS_SIZE_BYTES] = [0; REGISTERS_SIZE_BYTES];
        self.req_regs_dump(RegisterID::DeviceID, &mut reg_dump);
        // Update internal register representation
        for i in 0..REGISTERS_COUNT {
            let reg_addr = RegisterID::try_from(i as u8).unwrap();
            let data_start = i * 4;
            let data_end = data_start + 4;
            let reg_data = &reg_dump[data_start..data_end];
            self.populate_register_map(reg_addr, reg_data);
        }
    }
    fn req_device_info(&mut self) {
        let mut reg_dump: [u8; 8] = [0; 8]; // Device ID (4 bytes) + Firmware Version (4 bytes)
        self.req_regs_dump(RegisterID::DeviceID, &mut reg_dump);
        // Update Device ID
        let device_id_data = &reg_dump[0..4];
        self.populate_register_map(RegisterID::DeviceID, device_id_data);
        // Update Firmware Version
        let firmware_version_data = &reg_dump[4..8];
        self.populate_register_map(RegisterID::FirmwareVersion, firmware_version_data);
    }

    fn req_operation_mode(&mut self) {
        let mut reg_dump: [u8; 4] = [0; 4]; // Operation Mode (4 bytes)
        self.req_regs_dump(RegisterID::OperationMode, &mut reg_dump);
        self.populate_register_map(RegisterID::OperationMode, &reg_dump);
    }

    fn req_pwm_duty_cycle(&mut self, motor_num: u8) {
        assert!(
            motor_num >= 1 && motor_num <= 4,
            "Motor number must be between 1 and 4"
        );
        let reg_id = match motor_num {
            1 => RegisterID::Motor1PWMDutyCycle,
            2 => RegisterID::Motor2PWMDutyCycle,
            3 => RegisterID::Motor3PWMDutyCycle,
            4 => RegisterID::Motor4PWMDutyCycle,
            _ => unreachable!(),
        };
        let mut reg_dump: [u8; 4] = [0; 4]; // PWM Duty Cycle (4 bytes)
        self.req_regs_dump(reg_id, &mut reg_dump);
        self.populate_register_map(reg_id, &reg_dump);
    }

    fn req_motor_dir(&mut self, motor_num: u8) {
        assert!(
            motor_num >= 1 && motor_num <= 4,
            "Motor number must be between 1 and 4"
        );
        let reg_id = match motor_num {
            1 => RegisterID::Motor1Direction,
            2 => RegisterID::Motor2Direction,
            3 => RegisterID::Motor3Direction,
            4 => RegisterID::Motor4Direction,
            _ => unreachable!(),
        };
        let mut reg_dump: [u8; 4] = [0; 4]; // Motor Direction (4 bytes)
        self.req_regs_dump(reg_id, &mut reg_dump);
        self.populate_register_map(reg_id, &reg_dump);
    }

    fn req_motor_counters_per_rev(&mut self, motor_num: u8) {
        assert!(
            motor_num >= 1 && motor_num <= 4,
            "Motor number must be between 1 and 4"
        );
        let reg_id = match motor_num {
            1 => RegisterID::Motor1CountsPerRevolution,
            2 => RegisterID::Motor2CountsPerRevolution,
            3 => RegisterID::Motor3CountsPerRevolution,
            4 => RegisterID::Motor4CountsPerRevolution,
            _ => unreachable!(),
        };
        let mut reg_dump: [u8; 4] = [0; 4]; // Counts Per Revolution (4 bytes)
        self.req_regs_dump(reg_id, &mut reg_dump);
        self.populate_register_map(reg_id, &reg_dump);
    }

    fn req_motor_rpm_curr(&mut self, motor_num: u8) {
        assert!(
            motor_num >= 1 && motor_num <= 4,
            "Motor number must be between 1 and 4"
        );
        let reg_id = match motor_num {
            1 => RegisterID::Motor1RPMCurrent,
            2 => RegisterID::Motor2RPMCurrent,
            3 => RegisterID::Motor3RPMCurrent,
            4 => RegisterID::Motor4RPMCurrent,
            _ => unreachable!(),
        };
        let mut reg_dump: [u8; 4] = [0; 4]; // RPM Current (4 bytes)
        self.req_regs_dump(reg_id, &mut reg_dump);
        self.populate_register_map(reg_id, &reg_dump);
    }

    fn req_motor_rpm_desired(&mut self, motor_num: u8) {
        assert!(
            motor_num >= 1 && motor_num <= 4,
            "Motor number must be between 1 and 4"
        );
        let reg_id = match motor_num {
            1 => RegisterID::Motor1RPMDesired,
            2 => RegisterID::Motor2RPMDesired,
            3 => RegisterID::Motor3RPMDesired,
            4 => RegisterID::Motor4RPMDesired,
            _ => unreachable!(),
        };
        let mut reg_dump: [u8; 4] = [0; 4]; // RPM Desired (4 bytes)
        self.req_regs_dump(reg_id, &mut reg_dump);
        self.populate_register_map(reg_id, &reg_dump);
    }

    fn req_motor_pid_params(&mut self, motor_num: u8) {
        assert!(
            motor_num >= 1 && motor_num <= 4,
            "Motor number must be between 1 and 4"
        );
        let reg_id = match motor_num {
            1 => [
                RegisterID::Motor1PIDKp,
                RegisterID::Motor1PIDKi,
                RegisterID::Motor1PIDKd,
            ],
            2 => [
                RegisterID::Motor2PIDKp,
                RegisterID::Motor2PIDKi,
                RegisterID::Motor2PIDKd,
            ],
            3 => [
                RegisterID::Motor3PIDKp,
                RegisterID::Motor3PIDKi,
                RegisterID::Motor3PIDKd,
            ],
            4 => [
                RegisterID::Motor4PIDKp,
                RegisterID::Motor4PIDKi,
                RegisterID::Motor4PIDKd,
            ],
            _ => unreachable!(),
        };
        let mut reg_dump: [u8; 12] = [0; 12]; // PID Parameter kp(4 bytes) ki(4 bytes) kd(4 bytes) (12 bytes)
        self.req_regs_dump(reg_id[0], &mut reg_dump);
        self.populate_register_map(reg_id[0], &reg_dump[0..4]); // Kp
        self.populate_register_map(reg_id[1], &reg_dump[4..8]); // Ki
        self.populate_register_map(reg_id[2], &reg_dump[8..12]); // Kd
    }

    fn req_internal_loop_time(&mut self) {
        let mut reg_dump: [u8; 4] = [0; 4]; // Internal Loop Time (4 bytes)
        self.req_regs_dump(RegisterID::InternalLoopTime, &mut reg_dump);
        self.populate_register_map(RegisterID::InternalLoopTime, &reg_dump);
    }

    fn req_last_error_status(&mut self) {
        let mut reg_dump: [u8; 4] = [0; 4]; // Last Error Status (4 bytes)
        self.req_regs_dump(RegisterID::LastErrorStatus, &mut reg_dump);
        self.populate_register_map(RegisterID::LastErrorStatus, &reg_dump);
    }

    pub fn print_register_dump(&mut self) {
        use RegisterData::*;

        self.req_all_registers();
        println!("Register Dump:");
        for (reg_id, reg_data) in &self.register_map {
            let reg_addr = *reg_id as u8;
            match reg_data {
                DeviceID(value) => println!("0x{:02X}: Device ID: {}", reg_addr, value),
                FirmwareVersion(value) => {
                    println!("0x{:02X}: Firmware Version: {}", reg_addr, value)
                }
                OperationMode(mode) => println!("0x{:02X}: Control Mode: {:?}", reg_addr, mode),
                MotorDirection(dir) => println!("0x{:02X}: Motor Direction: {:?}", reg_addr, dir),
                MotorPWMDutyCycle(val) => println!("0x{:02X}: PWM Duty Cycle: {}", reg_addr, val),
                MotorCountsPerRevolution(val) => {
                    println!("0x{:02X}: Counts/Rev: {}", reg_addr, val)
                }
                MotorRPM(val) => println!("0x{:02X}: RPM: {}", reg_addr, val),
                MotorPIDParam(val) => println!("0x{:02X}: PID Param: {}", reg_addr, val),
                InternalLoopTime(val) => println!("0x{:02X}: Loop Time: {}", reg_addr, val),
                LastErrorStatus(err) => println!("0x{:02X}: Error Status: {:?}", reg_addr, err),
            }
        }
    }

    pub fn get_device_info(&mut self) -> (String, String) {
        self.req_device_info();
        let device_id = match self.register_map.get(&RegisterID::DeviceID) {
            Some(RegisterData::DeviceID(id)) => id.clone(),
            _ => "Unknown".to_string(),
        };
        let firmware_version = match self.register_map.get(&RegisterID::FirmwareVersion) {
            Some(RegisterData::FirmwareVersion(ver)) => ver.clone(),
            _ => "0.0.00".to_string(),
        };
        (device_id, firmware_version)
    }
    pub fn get_control_mode(&mut self) -> ControlMode {
        self.req_operation_mode();
        match self.register_map.get(&RegisterID::OperationMode) {
            Some(RegisterData::OperationMode(mode)) => *mode,
            _ => ControlMode::Position,
        }
    }

    pub fn get_pwm_duty_cycle(&mut self, motor_num: u8) -> u32 {
        assert!(
            motor_num >= 1 && motor_num <= 4,
            "Motor number must be between 1 and 4"
        );
        let reg_id = match motor_num {
            1 => RegisterID::Motor1PWMDutyCycle,
            2 => RegisterID::Motor2PWMDutyCycle,
            3 => RegisterID::Motor3PWMDutyCycle,
            4 => RegisterID::Motor4PWMDutyCycle,
            _ => unreachable!(),
        };
        self.req_pwm_duty_cycle(motor_num);
        match self.register_map.get(&reg_id) {
            Some(RegisterData::MotorPWMDutyCycle(value)) => *value,
            _ => 0,
        }
    }

    pub fn get_motor_direction(&mut self, motor_num: u8) -> MotorDirection {
        assert!(
            motor_num >= 1 && motor_num <= 4,
            "Motor number must be between 1 and 4"
        );
        let reg_id = match motor_num {
            1 => RegisterID::Motor1Direction,
            2 => RegisterID::Motor2Direction,
            3 => RegisterID::Motor3Direction,
            4 => RegisterID::Motor4Direction,
            _ => unreachable!(),
        };
        self.req_motor_dir(motor_num);
        match self.register_map.get(&reg_id) {
            Some(RegisterData::MotorDirection(dir)) => *dir,
            _ => MotorDirection::Stop,
        }
    }

    pub fn get_motor_counts_per_revolution(&mut self, motor_num: u8) -> u32 {
        assert!(
            motor_num >= 1 && motor_num <= 4,
            "Motor number must be between 1 and 4"
        );
        let reg_id = match motor_num {
            1 => RegisterID::Motor1CountsPerRevolution,
            2 => RegisterID::Motor2CountsPerRevolution,
            3 => RegisterID::Motor3CountsPerRevolution,
            4 => RegisterID::Motor4CountsPerRevolution,
            _ => unreachable!(),
        };
        self.req_motor_counters_per_rev(motor_num);
        match self.register_map.get(&reg_id) {
            Some(RegisterData::MotorCountsPerRevolution(value)) => *value,
            _ => 0,
        }
    }

    pub fn get_motor_rpm_current(&mut self, motor_num: u8) -> u32 {
        assert!(
            motor_num >= 1 && motor_num <= 4,
            "Motor number must be between 1 and 4"
        );
        let reg_id = match motor_num {
            1 => RegisterID::Motor1RPMCurrent,
            2 => RegisterID::Motor2RPMCurrent,
            3 => RegisterID::Motor3RPMCurrent,
            4 => RegisterID::Motor4RPMCurrent,
            _ => unreachable!(),
        };
        self.req_motor_rpm_curr(motor_num);
        match self.register_map.get(&reg_id) {
            Some(RegisterData::MotorRPM(value)) => *value,
            _ => 0,
        }
    }

    pub fn get_motor_rpm_desired(&mut self, motor_num: u8) -> u32 {
        assert!(
            motor_num >= 1 && motor_num <= 4,
            "Motor number must be between 1 and 4"
        );
        let reg_id = match motor_num {
            1 => RegisterID::Motor1RPMDesired,
            2 => RegisterID::Motor2RPMDesired,
            3 => RegisterID::Motor3RPMDesired,
            4 => RegisterID::Motor4RPMDesired,
            _ => unreachable!(),
        };
        self.req_motor_rpm_desired(motor_num);
        match self.register_map.get(&reg_id) {
            Some(RegisterData::MotorRPM(value)) => *value,
            _ => 0,
        }
    }

    pub fn get_motor_rpm_params(&mut self, motor_num: u8) -> PidParams {
        assert!(
            motor_num >= 1 && motor_num <= 4,
            "Motor number must be between 1 and 4"
        );
        let mut pid_params = PidParams {
            kp: 0,
            ki: 0,
            kd: 0,
        };
        for param_type in &[PidParamType::Kp, PidParamType::Ki, PidParamType::Kd] {
            let regs = match motor_num {
                1 => [
                    RegisterID::Motor1PIDKp,
                    RegisterID::Motor1PIDKi,
                    RegisterID::Motor1PIDKd,
                ],
                2 => [
                    RegisterID::Motor2PIDKp,
                    RegisterID::Motor2PIDKi,
                    RegisterID::Motor2PIDKd,
                ],
                3 => [
                    RegisterID::Motor3PIDKp,
                    RegisterID::Motor3PIDKi,
                    RegisterID::Motor3PIDKd,
                ],
                4 => [
                    RegisterID::Motor4PIDKp,
                    RegisterID::Motor4PIDKi,
                    RegisterID::Motor4PIDKd,
                ],
                _ => unreachable!(),
            };
            self.req_motor_pid_params(motor_num);
            for reg_id in regs.iter() {
                match self.register_map.get(reg_id) {
                    Some(RegisterData::MotorPIDParam(value)) => match reg_id {
                        RegisterID::Motor1PIDKp
                        | RegisterID::Motor2PIDKp
                        | RegisterID::Motor3PIDKp
                        | RegisterID::Motor4PIDKp => pid_params.kp = *value,
                        RegisterID::Motor1PIDKi
                        | RegisterID::Motor2PIDKi
                        | RegisterID::Motor3PIDKi
                        | RegisterID::Motor4PIDKi => pid_params.ki = *value,
                        RegisterID::Motor1PIDKd
                        | RegisterID::Motor2PIDKd
                        | RegisterID::Motor3PIDKd
                        | RegisterID::Motor4PIDKd => pid_params.kd = *value,
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        pid_params
    }

    pub fn get_internal_loop_time(&mut self) -> u32 {
        self.req_internal_loop_time();
        match self.register_map.get(&RegisterID::InternalLoopTime) {
            Some(RegisterData::InternalLoopTime(value)) => *value,
            _ => 0,
        }
    }

    pub fn get_last_error_status(&mut self) -> ErrorCode {
        self.req_last_error_status();
        match self.register_map.get(&RegisterID::LastErrorStatus) {
            Some(RegisterData::LastErrorStatus(err)) => *err,
            _ => ErrorCode::NoError,
        }
    }
}
