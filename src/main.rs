use clap::{ArgAction, Parser, Subcommand, ValueEnum};

use crate::device::Device;

mod device;
mod spi_proto;

const SUPPORTED_SPI_IFS: [&str; 2] = ["/dev/spidev0.0", "/dev/spidev0.1"];

#[derive(Debug, Clone, ValueEnum)]
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

enum MotorSelection {
    All,
    Indices(Vec<usize>),
}

fn parse_motor_selection(motors: &[u8], all: &bool) -> Result<MotorSelection, String> {
    if *all {
        if !motors.is_empty() {
            return Err("Use --all or provide motor numbers, not both.".to_string());
        }
        return Ok(MotorSelection::All);
    }

    if motors.is_empty() {
        return Err("No motors specified. Pass --all or a motor list.".to_string());
    }

    let max = device::DEVICE_SUPPORTED_MOTORS as u8;
    let mut indices = Vec::with_capacity(motors.len());
    for &motor in motors {
        if motor == 0 || motor > max {
            return Err(format!("Invalid motor {} (valid: 1..={}).", motor, max));
        }
        indices.push((motor - 1) as usize);
    }
    Ok(MotorSelection::Indices(indices))
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
            let selection = match parse_motor_selection(&motors, all) {
                Ok(selection) => selection,
                Err(err) => {
                    eprintln!("{err}");
                    return;
                }
            };
            let direction: device::MotorDirection = (*direction).clone().into();
            match selection {
                MotorSelection::All => {
                    eprintln!(
                        "SetMotDir is not implemented yet. Would set all motors to {:?}.",
                        direction
                    );
                }
                MotorSelection::Indices(indices) => {
                    let motors_list: Vec<usize> = indices.iter().map(|idx| idx + 1).collect();
                    eprintln!(
                        "SetMotDir is not implemented yet. Would set motors {:?} to {:?}.",
                        motors_list, direction
                    );
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
            let selection = match parse_motor_selection(&motors, all) {
                Ok(selection) => selection,
                Err(err) => {
                    eprintln!("{err}");
                    return;
                }
            };
            match selection {
                MotorSelection::All => {
                    eprintln!(
                        "SetMotPwm is not implemented yet. Would set all motors to PWM {}.",
                        pwm_value
                    );
                }
                MotorSelection::Indices(indices) => {
                    let motors_list: Vec<usize> = indices.iter().map(|idx| idx + 1).collect();
                    eprintln!(
                        "SetMotPwm is not implemented yet. Would set motors {:?} to PWM {}.",
                        motors_list, pwm_value
                    );
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
    }
}
