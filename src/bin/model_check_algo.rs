use std::path::{Path,PathBuf};
use std::io;
use std::fs;
use clap::Parser;

use synth_lights::common;
use synth_lights::promela;
use synth_lights::runner;
use synth_lights::runner::SpinOutcome;

#[derive(Debug, Parser)]
#[clap(author, version, about="Given the VALID promela code for an algorithm, check that algorithm in the model checker", long_about = None)]
#[allow(non_snake_case)]
pub struct Cli {
    /// Scheduler of the model
    #[arg(short = 's', long = "sched", value_enum, default_value = "async")]
    scheduler: common::Scheduler,

    /// Rigid moves restriction (otherwise non-rigid)
    #[arg(long = "rigid")]
    rigid: bool,

    /// Quasi self-stabilizing restriction (otherwise self-stabilizing)
    #[arg(short = 'Q', long = "quasi-ss")]
    quasi_ss: bool,
    
    /// Algorithm code string (e.g., 0_1_2__S2_H0_O1)
    #[clap(short = 'a', long="algo")]
    algorithm: Option<PathBuf>,

    #[arg(short = 'r', long = "ramdisk")]
    ramdisk: Option<String>,
}

fn run_verification(enclosure: &Path, promela: &str, model_run_options: promela::ModelRunOptions) -> anyhow::Result<(SpinOutcome, Option<String>)> {
    log::info!("Running verification");

    let outcome = runner::run_verification_from_code(&enclosure, promela, model_run_options)?;
    let trail = runner::read_trail_file(&enclosure)?;
    Ok((outcome, trail))
}


fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    log::debug!("Run options: {:?}", cli);

    log::info!("Preparing environment");

    let model_run_options = promela::ModelRunOptions {
        scheduler: cli.scheduler,
        rigid: cli.rigid,
        quasi_ss: cli.quasi_ss,
    };

    let promela = 
        match &cli.algorithm {
            Some(path) => fs::read_to_string(path)?,
            None => io::read_to_string(io::stdin())?,
        };

    let workdir = runner::create_root_workdir(cli.ramdisk.clone())?;
    let enclosure = runner::create_enclosure(workdir.path())?;

    let result = run_verification(&enclosure, &promela, model_run_options);

    // let trail = runner::read_trail_file(&enclosure);
    // println!("{}", trail.unwrap());
    runner::close_workdir(workdir)?;

    let (outcome, trail) = result?;

    println!("{}", promela);
    println!();
    println!("{}", outcome);
    if let Some(trail) = trail {
        println!("{}", trail);
    }

    Ok(())
}
