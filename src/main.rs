
use clap::{Parser, Subcommand, ValueEnum};

use crate::device::Device;

mod device;
mod spi_proto;
#[derive(Debug, Clone, ValueEnum)]
#[derive(Subcommand)]
enum MotorDirection {
    /// Forward
    Fw,
    /// Backward
    Bw,
    /// Stop
    Stop,
}

#[derive(Subcommand)]
enum Commands {
   /// Get firmware version
    FwVers,
   /// Get device ID
    DeviceId, 
    /// Get motor direction status
    /// 0 -> all motors, 1 -> motor 1, 2 -> motor 2, etc.
    GetMotDir {
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(0..=4))]
        motors: Vec<u8>,
    },
    /// Set motor direction
    /// 0 -> motor 1, 1 -> motor 2, etc.
    SetMotDir {
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(0..=4))]
        motors: Vec<u8>,
        /// Direction: Fw -> Forward, Bw -> Backward, Stop -> Stop
        #[arg(short='d', long="dir")]
        direction: MotorDirection,
    },
    /// Get motor pwm value
    /// 0 -> all motors, 1 -> motor 1, 2 -> motor 2, etc.
    GetMotPwm {
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(0..=4))]
        motors: Vec<u8>,
    },
    /// Set motor pwm value
    /// 0 -> all motors, 1 -> motor 1, 2 -> motor 2, etc.
    /// Pwm value range: 0 - 100
    SetMotPwm {
        #[arg(value_delimiter = ' ', value_parser = clap::value_parser!(u8).range(0..=4))]
        motors: Vec<u8>,
        #[arg(short='p', long="pwm", value_parser = clap::value_parser!(u8).range(0..=100))]
        pwm_values: u8,
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

    match cli.spi_if.as_str() {
        "/dev/spidev0.0" | "/dev/spidev0.1" => {},
        _ => {
            eprintln!("Unsupported device path: {}", cli.spi_if);
            return;
        }
    }

    match &cli.command {
        Commands::FwVers => {
            println!("Firmware Version command selected");
        },
        Commands::DeviceId => {
            println!("Device ID command selected");
        },
        Commands::GetMotDir { motors } => {
            println!("Get Motor Direction command selected for motors: {:?}", motors);
        },
        Commands::SetMotDir { motors, direction } => {
            println!("Set Motor Direction command selected for motors: {:?} to direction: {:?}", motors, direction);
        },
        Commands::GetMotPwm { motors } => {
            println!("Get Motor PWM command selected for motors: {:?}", motors);
        },
        Commands::SetMotPwm { motors, pwm_values } => {
            println!("Set Motor PWM command selected for motors: {:?} to PWM value: {}", motors, pwm_values);
        },
    }
    let mut device = Device::new(&cli.spi_if);
    device.print_register_dump();
}