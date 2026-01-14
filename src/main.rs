use clap::{ArgAction, Parser, Subcommand, ValueEnum};

use crate::device::Device;

mod device;
mod spi_proto;


#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliMotorDirection {
    /// Forward
    Fw,
    /// Backward
    Bw,
    /// Stop
    Stop,
}

impl From<CliMotorDirection> for device::MotorDirection {
    fn from(direction: CliMotorDirection) -> Self {
        match direction {
            CliMotorDirection::Fw => device::MotorDirection::Forward,
            CliMotorDirection::Bw => device::MotorDirection::Backward,
            CliMotorDirection::Stop => device::MotorDirection::Stop,
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

/// Helper function to write a PID parameter to all motors
fn write_pid_param_to_all_motors(
    dev: &mut Device,
    value: u32,
    kp_regs: [device::RegisterID; 4],
) {
    let param_bytes = value.to_le_bytes();
    for reg in kp_regs {
        dev.req_reg_write(reg, &param_bytes);
    }
}

/// Helper function to write a PID parameter to specific motors
fn write_pid_param_to_motors(
    dev: &mut Device,
    motors: &[u8],
    value: u32,
    offset: device::MotorRegisterOffset,
) {
    let param_bytes = value.to_le_bytes();
    for motor in motors {
        let idx = (*motor - 1) as usize;
        let motor_id = device::MotorID::try_from(idx as u8).unwrap();
        let reg_id = device::RegisterID::from_motor_id(&motor_id, offset);
        dev.req_reg_write(reg_id, &param_bytes);
    }
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
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(1..=4))]
        motors: Vec<u8>,
        /// Query all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
    },
    /// Get motor direction status
    GetMotDir {
        /// Motors to query: 1..=4, or use --all
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(1..=4))]
        motors: Vec<u8>,
        /// Query all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
    },
    /// Set motor direction
    SetMotDir {
        /// Motors to set: 1..=4, or use --all
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(1..=4))]
        motors: Vec<u8>,
        /// Set all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
        /// Direction: Fw -> Forward, Bw -> Backward, Stop -> Stop
        #[arg(short = 'd', long = "dir")]
        direction: CliMotorDirection,
    },
    /// Get motor pwm value
    GetMotPwm {
        /// Motors to query: 1..=4, or use --all
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(1..=4))]
        motors: Vec<u8>,
        /// Query all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
    },
    /// Set motor pwm value
    /// Pwm value range: 0 - 100
    SetMotPwm {
        /// Motors to set: 1..=4, or use --all
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(1..=4))]
        motors: Vec<u8>,
        /// Set all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
        #[arg(short = 'p', long = "pwm", value_parser = clap::value_parser!(u8).range(0..=100))]
        pwm_value: u8,
    },
    /// Get PID parameters
    GetPidParams {
        /// Motors to query: 1..=4, or use --all
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(1..=4))]
        motors: Vec<u8>,
        /// Query all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
    },
    /// Set PID parameters
    SetPidParams {
        /// Motors to set: 1..=4, or use --all
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(1..=4))]
        motors: Vec<u8>,
        /// Set all motors
        #[arg(long, action = ArgAction::SetTrue)]
        all: bool,
        /// PID Kp parameter (optional)
        #[arg(long = "kp")]
        kp: Option<u32>,
        /// PID Ki parameter (optional)
        #[arg(long = "ki")]
        ki: Option<u32>,
        /// PID Kd parameter (optional)
        #[arg(long = "kd")]
        kd: Option<u32>,
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
           if *all == true {
                let dir_reg = (device::MotorDirection::from(*direction) as u32).to_le_bytes();
                dev.req_reg_write(device::RegisterID::Motor1Direction, &dir_reg);
                dev.req_reg_write(device::RegisterID::Motor2Direction, &dir_reg);
                dev.req_reg_write(device::RegisterID::Motor3Direction, &dir_reg);
                dev.req_reg_write(device::RegisterID::Motor4Direction, &dir_reg);
           } else {
                if motors.is_empty() {
                    eprintln!("No motors specified. Pass --all or a motor list.");
                    return;
                }
                for motor in motors {
                    let idx = (*motor - 1) as usize;
                    let motor_id = device::MotorID::try_from(idx as u8).unwrap();
                    let reg_id = device::RegisterID::from_motor_id(&motor_id, device::MotorRegisterOffset::Direction);
                    let dir_reg = (device::MotorDirection::from(*direction) as u32).to_le_bytes();
                    dev.req_reg_write(reg_id, &dir_reg);
                }
           }
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
           if *all == true {
                let pwm_reg = (*pwm_value as u32).to_le_bytes();
                dev.req_reg_write(device::RegisterID::Motor1PWMDutyCycle, &pwm_reg);
                dev.req_reg_write(device::RegisterID::Motor2PWMDutyCycle, &pwm_reg);
                dev.req_reg_write(device::RegisterID::Motor3PWMDutyCycle, &pwm_reg);
                dev.req_reg_write(device::RegisterID::Motor4PWMDutyCycle, &pwm_reg);
           } else {
                if motors.is_empty() {
                    eprintln!("No motors specified. Pass --all or a motor list.");
                    return;
                }
                for motor in motors {
                    let idx = (*motor - 1) as usize;
                    let motor_id = device::MotorID::try_from(idx as u8).unwrap();
                    let reg_id = device::RegisterID::from_motor_id(&motor_id, device::MotorRegisterOffset::PWMDutyCycle);
                    let pwm_reg = (*pwm_value as u32).to_le_bytes();
                    dev.req_reg_write(reg_id, &pwm_reg);
                }
           } 
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

            if *all {
                if let Some(kp_val) = kp {
                    write_pid_param_to_all_motors(
                        &mut dev,
                        *kp_val,
                        [
                            device::RegisterID::Motor1PIDKp,
                            device::RegisterID::Motor2PIDKp,
                            device::RegisterID::Motor3PIDKp,
                            device::RegisterID::Motor4PIDKp,
                        ],
                    );
                }
                
                if let Some(ki_val) = ki {
                    write_pid_param_to_all_motors(
                        &mut dev,
                        *ki_val,
                        [
                            device::RegisterID::Motor1PIDKi,
                            device::RegisterID::Motor2PIDKi,
                            device::RegisterID::Motor3PIDKi,
                            device::RegisterID::Motor4PIDKi,
                        ],
                    );
                }
                
                if let Some(kd_val) = kd {
                    write_pid_param_to_all_motors(
                        &mut dev,
                        *kd_val,
                        [
                            device::RegisterID::Motor1PIDKd,
                            device::RegisterID::Motor2PIDKd,
                            device::RegisterID::Motor3PIDKd,
                            device::RegisterID::Motor4PIDKd,
                        ],
                    );
                }
            } else {
                if motors.is_empty() {
                    eprintln!("No motors specified. Pass --all or a motor list.");
                    return;
                }
                
                if let Some(kp_val) = kp {
                    write_pid_param_to_motors(&mut dev, motors, *kp_val, device::MotorRegisterOffset::PIDKp);
                }
                
                if let Some(ki_val) = ki {
                    write_pid_param_to_motors(&mut dev, motors, *ki_val, device::MotorRegisterOffset::PIDKi);
                }
                
                if let Some(kd_val) = kd {
                    write_pid_param_to_motors(&mut dev, motors, *kd_val, device::MotorRegisterOffset::PIDKd);
                }
            }
        }
    }
}
