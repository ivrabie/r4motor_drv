use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use log::{error, info};
use std::thread::sleep;
use std::time::Duration;

use crate::device::{Device, MotorRegisterOffset};

mod device;
mod spi_proto;

pub const MOTOR_INDEX_RANGE_I64: std::ops::RangeInclusive<i64> = 1..=4;
pub const MOTOR_INDEX_RANGE: std::ops::RangeInclusive<u8> = 1..=4;

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "kebab_case")]
enum MonitorParam {
    Mode,
    #[value(alias = "dir")]
    Direction,
    Pwm,
    #[value(alias = "rpm")]
    RpmDesired,
    RpmCurrent,
    #[value(alias = "kp")]
    PidKp,
    #[value(alias = "ki")]
    PidKi,
    #[value(alias = "kd")]
    PidKd,
    #[value(alias = "counts-per-revolution")]
    CountsPerRev,
}

const ALL_MOTOR_PARAMS: [MonitorParam; 9] = [
    MonitorParam::Mode,
    MonitorParam::Direction,
    MonitorParam::Pwm,
    MonitorParam::RpmDesired,
    MonitorParam::RpmCurrent,
    MonitorParam::PidKp,
    MonitorParam::PidKi,
    MonitorParam::PidKd,
    MonitorParam::CountsPerRev,
];

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "kebab_case")]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for log::LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}

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
    /// Get motor parameters
    Get {
        /// Motors to query: 1..=4, or use --all
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(MOTOR_INDEX_RANGE_I64))]
        motors: Vec<u8>,
        /// Query all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
        /// Get all motor parameters
        #[arg(long, action = ArgAction::SetTrue)]
        all_params: bool,
        /// Parameters to get
        #[arg(long, num_args = 1.., value_enum)]
        params: Vec<MonitorParam>,
    },
    /// Set motor parameters
    Set {
        /// Motors to set: 1..=4, or use --all
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(MOTOR_INDEX_RANGE_I64))]
        motors: Vec<u8>,
        /// Set all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
        /// Control mode: Pwm -> PWM control, Rpm -> RPM control
        #[arg(short = 'm', long = "mode")]
        mode: Option<device::ControlMode>,
        /// Direction: Fw -> Forward, Bw -> Backward, Stop -> Stop
        #[arg(short = 'd', long = "dir")]
        dir: Option<device::MotorDirection>,
        /// Pwm value range: 0 - 100
        #[arg(short = 'p', long = "pwm", value_parser = clap::value_parser!(i32).range(0..=100))]
        pwm: Option<i32>,
        /// Desired RPM value
        #[arg(short = 'r', long = "rpm", value_parser = clap::value_parser!(i32).range(0..))]
        rpm: Option<i32>,
        /// Counts per revolution
        #[arg(long = "counts-per-rev", value_parser = clap::value_parser!(i32).range(1..))]
        counts_per_rev: Option<i32>,
        /// PID Kp parameter (optional)
        #[arg(long = "kp")]
        kp: Option<f32>,
        /// PID Ki parameter (optional)
        #[arg(long = "ki")]
        ki: Option<f32>,
        /// PID Kd parameter (optional)
        #[arg(long = "kd")]
        kd: Option<f32>,
    },
    /// Monitor motor parameters
    Monitor {
        // Motors to monitor: 1..=4, or use --all
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(MOTOR_INDEX_RANGE_I64))]
        motors: Vec<u8>,

        /// Monitor all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,

        // Cyclicity in milliseconds
        #[arg(long = "period-ms", value_parser = clap::value_parser!(u64).range(1..))]
        period_ms: u64,
        /// Parameters to monitor
        #[arg(long, num_args = 1.., value_enum, required = true)]
        params: Vec<MonitorParam>,
        /// Number of cycles to run (optional)
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
        count: Option<u64>,
    },
}

#[derive(Parser)]
#[command(name = "rust-4motor-drv")]
#[command(about = "SPI Motor Driver", long_about = None)]
struct Cli {
    /// SPI device path
    #[arg(short, long, default_value = "/dev/spidev0.0")]
    spi_if: String,

    /// Logging level
    #[arg(long, value_enum, default_value = "info")]
    log_level: LogLevel,

    #[command(subcommand)]
    command: Commands,
}

fn set_motors_reg_value(
    dev: &mut Device,
    motors: &[u8],
    all: bool,
    reg_offset: MotorRegisterOffset,
    value: i32,
) {
    if !all && motors.is_empty() {
        error!("No motors specified. Pass --all or a motor list.");
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

fn set_motors_reg_value_u32(
    dev: &mut Device,
    motors: &[u8],
    all: bool,
    reg_offset: MotorRegisterOffset,
    value: u32,
) {
    if !all && motors.is_empty() {
        error!("No motors specified. Pass --all or a motor list.");
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

fn set_motors_reg_value_f32(
    dev: &mut Device,
    motors: &[u8],
    all: bool,
    reg_offset: MotorRegisterOffset,
    value: f32,
) {
    if !all && motors.is_empty() {
        error!("No motors specified. Pass --all or a motor list.");
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

fn format_motor_params(status: device::MotorStatus, params: &[MonitorParam]) -> String {
    let mut fields = Vec::with_capacity(params.len());
    for param in params {
        match param {
            MonitorParam::Mode => fields.push(format!("mode={:?}", status.mode)),
            MonitorParam::Direction => fields.push(format!("direction={:?}", status.direction)),
            MonitorParam::Pwm => fields.push(format!("pwm={}", status.pwm_duty_cycle)),
            MonitorParam::RpmDesired => fields.push(format!("rpm-desired={}", status.rpm_desired)),
            MonitorParam::RpmCurrent => fields.push(format!("rpm-current={}", status.rpm_current)),
            MonitorParam::PidKp => fields.push(format!("pid-kp={}", status.pid_params.kp)),
            MonitorParam::PidKi => fields.push(format!("pid-ki={}", status.pid_params.ki)),
            MonitorParam::PidKd => fields.push(format!("pid-kd={}", status.pid_params.kd)),
            MonitorParam::CountsPerRev => {
                fields.push(format!("counts-per-rev={}", status.counts_per_revolution))
            }
        }
    }
    fields.join(" ")
}

fn main() {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.log_level.into())
        .init();

    let mut dev = Device::new(&cli.spi_if);
    match &cli.command {
        Commands::FwVers => {
            let sys_info = dev.get_device_info();
            info!("Firmware Version: {}", sys_info.firmware_version);
        }
        Commands::DeviceId => {
            let sys_info = dev.get_device_info();
            info!("Device ID: {}", sys_info.device_id);
        }
        Commands::DevAll => {
            let full_info = dev.get_all_registers();
            Device::print_full_device_info(&full_info);
        }
        Commands::GetLoopTime => {
            let loop_time = dev.get_internal_loop_time_ms();
            info!("Internal Loop Time: {} ms", loop_time);
        }
        Commands::GetLastError => {
            let error_status = dev.get_last_error_status();
            info!("Last Error Status: {:?}", error_status);
        }
        Commands::Get {
            motors,
            all,
            all_params,
            params,
        } => {
            let selected_params: &[MonitorParam] = if *all_params {
                &ALL_MOTOR_PARAMS
            } else {
                if params.is_empty() {
                    error!("At least one parameter must be specified, or use --all-params.");
                    return;
                }
                params.as_slice()
            };
            if let Err(err) = for_each_motor_status(&mut dev, motors, *all, |idx, status| {
                let fields = format_motor_params(status, selected_params);
                info!("motor={} {}", idx + 1, fields);
            }) {
                error!("{err}");
            }
        }
        Commands::Set {
            motors,
            all,
            mode,
            dir,
            pwm,
            rpm,
            counts_per_rev,
            kp,
            ki,
            kd,
        } => {
            let writes_u32 = [
                mode.map(|val| (MotorRegisterOffset::OperationMode, val as u32)),
                dir.map(|val| (MotorRegisterOffset::Direction, val as u32)),
            ];
            let writes_i32 = [
                pwm.map(|val| (MotorRegisterOffset::PWMDutyCycle, val)),
                rpm.map(|val| (MotorRegisterOffset::RPMDesired, val)),
                counts_per_rev.map(|val| (MotorRegisterOffset::CountsPerRevolution, val)),
            ];
            let writes_f32 = [
                kp.map(|val| (MotorRegisterOffset::PIDKp, val)),
                ki.map(|val| (MotorRegisterOffset::PIDKi, val)),
                kd.map(|val| (MotorRegisterOffset::PIDKd, val)),
            ];
            if !writes_u32.iter().any(|entry| entry.is_some())
                && !writes_i32.iter().any(|entry| entry.is_some())
                && !writes_f32.iter().any(|entry| entry.is_some())
            {
                error!("At least one parameter must be specified.");
                return;
            }
            if !*all && motors.is_empty() {
                error!("No motors specified. Pass --all or a motor list.");
                return;
            }
            for (offset, value) in writes_u32.into_iter().flatten() {
                set_motors_reg_value_u32(&mut dev, motors, *all, offset, value);
            }
            for (offset, value) in writes_i32.into_iter().flatten() {
                set_motors_reg_value(&mut dev, motors, *all, offset, value);
            }
            for (offset, value) in writes_f32.into_iter().flatten() {
                set_motors_reg_value_f32(&mut dev, motors, *all, offset, value);
            }
        }
        Commands::Monitor {
            motors,
            all,
            period_ms,
            params,
            count,
        } => {
            let mut cycles_left = *count;
            loop {
                if let Err(err) = for_each_motor_status(&mut dev, motors, *all, |idx, status| {
                    let fields = format_motor_params(status, params);
                    info!("motor={} {}", idx + 1, fields);
                }) {
                    error!("{err}");
                    return;
                }

                if let Some(left) = cycles_left.as_mut() {
                    if *left == 0 {
                        break;
                    }
                    *left -= 1;
                    if *left == 0 {
                        break;
                    }
                }

                sleep(Duration::from_millis(*period_ms));
            }
        }
    }
}
