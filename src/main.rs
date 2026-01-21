use clap::{ArgAction, Parser, Subcommand};

use crate::device::{Device, MotorRegisterOffset};

mod device;
mod spi_proto;

pub const MOTOR_INDEX_RANGE_I64: std::ops::RangeInclusive<i64> = 1..=4;
pub const MOTOR_INDEX_RANGE: std::ops::RangeInclusive<u8> = 1..=4;

fn for_each_motor_status<F>(
    dev: &mut Device,
    motors: &[u8],
    all: bool,
    mut f: F,
) -> Result<(), String>
where
    F: FnMut(usize, device::MotorStatus),
{
    if all {
        if !motors.is_empty() {
            return Err("Use --all or provide motor numbers, not both.".to_string());
        }
        let statuses = dev.get_motor_dump_all();
        for (idx, status) in statuses.iter().enumerate() {
            f(idx, *status);
        }
        return Ok(());
    }

    if motors.is_empty() {
        return Err("No motors specified. Pass --all or a motor list.".to_string());
    }

    let max = device::DEVICE_SUPPORTED_MOTORS as u8;
    for motor in motors {
        if *motor == 0 || *motor > max {
            return Err(format!("Invalid motor {} (valid: 1..={}).", motor, max));
        }
        let idx = (*motor - 1) as usize;
        let motor_id = device::MotorID::try_from(idx as u8).unwrap();
        let status = dev.get_motor_dump(motor_id);
        f(idx, status);
    }
    Ok(())
}

#[derive(Subcommand)]
enum Commands {
    /// Get firmware version
    FwVers,
    /// Get device ID
    DeviceId,
    /// Get all device status
    DevAll,
    /// Get internal loop time in ms
    GetLoopTime,
    /// Get last error status
    GetLastError,
    /// Get motor control mode
    GetMotMode {
        /// Motors to query: 1..=4, or use --all
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(MOTOR_INDEX_RANGE_I64))]
        motors: Vec<u8>,
        /// Query all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
    },
    /// Set motor control mode
    SetMotMode {
        /// Motors to set: 1..=4, or use --all
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(MOTOR_INDEX_RANGE_I64))]
        motors: Vec<u8>,
        /// Set all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
        /// Control mode: Auto -> Automatic, Pwm -> PWM control, Rpm -> RPM control
        #[arg(short = 'm', long = "mode")]
        mode: device::ControlMode,
    },
    /// Get motor direction status
    GetMotDir {
        /// Motors to query: 1..=4, or use --all
    #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(MOTOR_INDEX_RANGE_I64))]
        motors: Vec<u8>,
        /// Query all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
    },
    /// Set motor direction
    SetMotDir {
        /// Motors to set: 1..=4, or use --all
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(MOTOR_INDEX_RANGE_I64))]
        motors: Vec<u8>,
        /// Set all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
        /// Direction: Fw -> Forward, Bw -> Backward, Stop -> Stop
        #[arg(short = 'd', long = "dir")]
        direction: device::MotorDirection,
    },
    /// Get motor pwm value
    GetMotPwm {
        /// Motors to query: 1..=4, or use --all
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(MOTOR_INDEX_RANGE_I64))]
        motors: Vec<u8>,
        /// Query all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
    },
    /// Set motor pwm value
    /// Pwm value range: 0 - 100
    SetMotPwm {
        /// Motors to set: 1..=4, or use --all
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(MOTOR_INDEX_RANGE_I64))]
        motors: Vec<u8>,
        /// Set all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
        #[arg(short = 'p', long = "pwm", value_parser = clap::value_parser!(i32).range(0..=100))]
        pwm_value: i32,
    },
    /// Get Rpm parameters 
    GetRpmParams {
        /// Motors to query: 1..=4, or use --all
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(MOTOR_INDEX_RANGE_I64))]
        motors: Vec<u8>,
        /// Query all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
    },
    /// Set Rpm desired value
    SetRpmParams {
        /// Motors to set: 1..=4, or use --all
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(MOTOR_INDEX_RANGE_I64))]
        motors: Vec<u8>,
        /// Set all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
        /// Desired RPM value
        #[arg(short = 'r', long = "rpm")]
        rpm_value: i32,
    },
    /// Get PID parameters
    GetPidParams {
        /// Motors to query: 1..=4, or use --all
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(MOTOR_INDEX_RANGE_I64))]
        motors: Vec<u8>,
        /// Query all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
    },
    /// Set PID parameters
    SetPidParams {
        /// Motors to set: 1..=4, or use --all
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(MOTOR_INDEX_RANGE_I64))]
        motors: Vec<u8>,
        /// Set all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
        /// PID Kp parameter (optional)
        #[arg(long = "kp")]
        kp: Option<i32>,
        /// PID Ki parameter (optional)
        #[arg(long = "ki")]
        ki: Option<i32>,
        /// PID Kd parameter (optional)
        #[arg(long = "kd")]
        kd: Option<i32>,
    },
}

#[derive(Parser)]
#[command(name = "rust-4motor-drv")]
#[command(about = "SPI Motor Driver", long_about = None)]
struct Cli {
    /// SPI device path
    #[arg(short, long, default_value = "/dev/spidev0.0")]
    spi_if: String,

    #[command(subcommand)]
    command: Commands,
}

fn set_motors_reg_value(dev: &mut Device, motors: &[u8], all: bool, reg_offset: MotorRegisterOffset, value: i32) {
    
    if !all && motors.is_empty() {
        eprintln!("No motors specified. Pass --all or a motor list.");
        return;
    }
    let motors = if all {
        MOTOR_INDEX_RANGE.collect::<Vec<u8>>()
    } else {
        motors.to_vec()
    };
    for motor in motors {
        let idx = (motor - 1) as usize;
        let motor_id = device::MotorID::try_from(idx as u8).unwrap();
        let reg_id = device::RegisterID::from_motor_id(motor_id, reg_offset);
        let reg_data = value.to_le_bytes();
        dev.req_reg_write(reg_id, &reg_data);
    }
}

fn main() {
    let cli = Cli::parse();

    let mut dev = Device::new(&cli.spi_if);
    match &cli.command {
        Commands::FwVers => {
            let sys_info = dev.get_device_info();
            println!("Firmware Version: {}", sys_info.firmware_version);
        }
        Commands::DeviceId => {
            let sys_info = dev.get_device_info();
            println!("Device ID: {}", sys_info.device_id);
        }
        Commands::DevAll => {
            let full_info = dev.get_all_registers();
            Device::print_full_device_info(&full_info);
        }
        Commands::GetLoopTime => {
            let loop_time = dev.get_internal_loop_time_ms();
            println!("Internal Loop Time: {} ms", loop_time);
        }
        Commands::GetLastError => {
            let error_status = dev.get_last_error_status();
            println!("Last Error Status: {:?}", error_status);
        }
        Commands::GetMotMode { motors, all } => {
            if let Err(err) = for_each_motor_status(&mut dev, motors, *all, |idx, status| {
                println!("Motor {} Control Mode: {:?}", idx + 1, status.mode);
            }) {
                eprintln!("{err}");
            }
        }
        Commands::SetMotMode { motors, all, mode } => {
            let mode_val = *mode as i32;
            set_motors_reg_value(&mut dev, motors, *all, MotorRegisterOffset::OperationMode, mode_val);
        }
        Commands::GetMotDir { motors, all } => {
            if let Err(err) = for_each_motor_status(&mut dev, motors, *all, |idx, status| {
                println!("Motor {} Direction: {:?}", idx + 1, status.direction);
            }) {
                eprintln!("{err}");
            }
        }
        Commands::SetMotDir {
            motors,
            all,
            direction,
        } => {
            let dir = *direction as i32;
            set_motors_reg_value(&mut dev, motors, *all, MotorRegisterOffset::Direction, dir);
        }
        Commands::GetMotPwm { motors, all } => {
            if let Err(err) = for_each_motor_status(&mut dev, motors, *all, |idx, status| {
                println!("Motor {} PWM: {}", idx + 1, status.pwm_duty_cycle);
            }) {
                eprintln!("{err}");
            }
        }
        Commands::SetMotPwm {
            motors,
            all,
            pwm_value,
        } => {
            set_motors_reg_value(&mut dev, motors, *all, MotorRegisterOffset::PWMDutyCycle, *pwm_value);
        }
        Commands::GetRpmParams { motors, all } => {
            if let Err(err) = for_each_motor_status(&mut dev, motors, *all, |idx, status| {
                println!("Motor {} RPM Parameters: desired {:?}, current {:?}", idx + 1, status.rpm_desired, status.rpm_current);
            }) {
                eprintln!("{err}");
            }
        }
        Commands::SetRpmParams {
            motors,
            all,
            rpm_value,
        } => {
            set_motors_reg_value(&mut dev, motors, *all, MotorRegisterOffset::RPMDesired, *rpm_value);
        }
        Commands::GetPidParams { motors, all } => {
            if let Err(err) = for_each_motor_status(&mut dev, motors, *all, |idx, status| {
                println!("Motor {} PID Parameters: {:?}", idx + 1, status.pid_params);
            }) {
                eprintln!("{err}");
            }
        }
        Commands::SetPidParams {
            motors,
            all,
            kp,
            ki,
            kd,
        } => {
            // Validate that at least one PID parameter is provided
            if kp.is_none() && ki.is_none() && kd.is_none() {
                eprintln!("At least one PID parameter (--kp, --ki, or --kd) must be specified.");
                return;
            }
            if let Some(kp_val) = kp {
                set_motors_reg_value(&mut dev, motors, *all, MotorRegisterOffset::PIDKp, *kp_val);
            }
            if let Some(ki_val) = ki {
                set_motors_reg_value(&mut dev, motors, *all, MotorRegisterOffset::PIDKi, *ki_val);
            }
            if let Some(kd_val) = kd {
                set_motors_reg_value(&mut dev, motors, *all, MotorRegisterOffset::PIDKd, *kd_val);
            }
        }
    }
}
