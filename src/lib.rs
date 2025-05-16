#![forbid(unsafe_code)]

pub mod algorithm;
pub mod common;
pub mod generator;
pub mod promela;
pub mod runner;
pub mod model;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use std::path::Path;
use std::path::PathBuf;
use strum::Display;

use convert_case::{Case, Casing};

use log::info;

use runner::{run_verification, SpinOutcome};

const DEFAULT_OUTPUT_DIR: &str = "results";

/// Algorithm synthesis for two robots gathering.
/// Given a system model, the program generates all viable algorithms for that model
/// and uses model checking to search for those that solve gathering (aka, rendez-vous).
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[allow(non_snake_case)]
pub struct Cli {
    /// Category of algorithms
    #[arg(value_enum)]
    category: ModelKind,

    /// Number of colors allowed in the model
    #[arg()]
    n_colors: u8,

    /// Limits search to class L algorithms
    #[arg(short = 'L')]
    class_L: bool,

    /// Enables sequential execution
    #[arg(short = 'S', long = "sequential")]
    sequential: bool,

    /// Enables weak filtering
    #[arg(short = 'w')]
    weak_filter: bool,

    /// Enables Viglietta's retain rule filtering ("A robot retains its color if and only if it sees the other robot set to a different color.")
    #[arg(short = 'R')]
    retain_filter: bool,

    /// Scheduler of the model
    #[arg(short = 's', long = "sched", value_enum, default_value = "async")]
    scheduler: common::Scheduler,

    /// Rigid moves restriction (otherwise non-rigid)
    #[arg(long = "rigid")]
    rigid: bool,

    /// Quasi self-stabilizing restriction (otherwise self-stabilizing)
    #[arg(short = 'Q', long = "quasi-ss")]
    quasi_ss: bool,

    /// Write output to a file (use default filename made from command line arguments if no name is specified with -o; stdout by default)
    #[arg(short = 'f', long = "file")]
    to_file: bool,

    /// Output file for reporting outcomes (-f is implicit if this option is provided)
    #[arg(short = 'o', long = "out")]
    output_dir: Option<PathBuf>,

    #[arg(short = 'r', long = "ramdisk")]
    ramdisk: Option<String>,
}

#[derive(Default, ValueEnum, Display, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModelKind {
    #[default]
    Full,
    Internal,
    External,
}

impl TryFrom<&str> for ModelKind {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        use ModelKind::*;
        match value {
            "F" => Ok(Full),
            "I" => Ok(Internal),
            "E" => Ok(External),
            _ => Err(anyhow::Error::msg(format!(
                "invalid model kind: {}",
                value
            ))),
        }
    }
}

impl TryFrom<String> for ModelKind {
    type Error = anyhow::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

fn suggested_name(cli: &Cli) -> String {
    let prefix = if cli.sequential { "output" } else { "parout" };
    let class_l = if cli.class_L { "_L" } else { "" };
    let kind = cli.category.to_string().to_lowercase();
    let n_colors = cli.n_colors;
    let scheduler = cli.scheduler.to_string().to_case(Case::Kebab);
    let rigid = if cli.rigid { "_rigid" } else { "" };
    let quasi_ss = if cli.quasi_ss { "_qss" } else { "" };
    format!("{prefix}{class_l}_{kind}_{n_colors}_{scheduler}{rigid}{quasi_ss}.txt")
}

pub fn run(cli: &Cli) -> Result<()> {
    use indicatif::ParallelProgressIterator;
    use rayon::prelude::*;
    use std::cell::RefCell;
    use std::fs::File;
    use std::io::Write;
    use std::time::{Duration, Instant};

    thread_local! {
        static ENCLOSURE: RefCell<Option<PathBuf>> = RefCell::new(None);
    }

    fn with_enclosure_do<F>(work_dir: &Path, action: F) -> Result<(usize, String, SpinOutcome)>
    where
        F: Fn(&Path) -> Result<(usize, String, SpinOutcome)>,
    {
        ENCLOSURE.with(|cell| {
            let mut enclosure = cell.borrow_mut();
            if enclosure.is_none() {
                let path = runner::create_enclosure(work_dir)?;
                *enclosure = Some(path);
            }
            let thread_enclosure = enclosure
                .as_deref()
                .ok_or_else(|| anyhow::Error::msg("Could not obtain enclosure"))?;
            action(thread_enclosure)
        })
    }

    let output_file_name = match cli.output_dir {
        Some(ref path) => Some(path.to_owned()),
        None if cli.to_file => {
            let path: PathBuf = [DEFAULT_OUTPUT_DIR, &suggested_name(cli)].iter().collect();
            Some(path)
        }
        _ => None,
    };

    if let Some(ref path) = output_file_name {
        info!(
            "Output to file: {}",
            path.to_str().ok_or_else(|| anyhow::Error::msg(format!(
                "cannot represent filename: {:?}",
                path.as_os_str()
            )))?
        );
    }

    let mut output: Box<dyn Write> = match output_file_name {
        Some(ref path) => Box::new(Tee::new(
            File::options()
                .write(true)
                .create_new(true)
                .open(path)
                .context("failed to open output file (name provided)")?,
            std::io::stdout(),
        )),
        None => Box::new(std::io::stdout()),
    };

    writeln!(output, "Run options: {:?}", cli)?;

    info!("Preparing environment");

    let model_run_options = promela::ModelRunOptions {
        scheduler: cli.scheduler,
        rigid: cli.rigid,
        quasi_ss: cli.quasi_ss,
    };
    let t_start = Instant::now();
    let workdir = runner::create_root_workdir(cli.ramdisk.clone())?;
    let weak_filter = cli.weak_filter;
    let retain_filter = cli.retain_filter;
    let category = cli.category;
    let n_colors = cli.n_colors;
    #[allow(non_snake_case)]
    let class_L = cli.class_L;

    let t_prepare = Instant::now() - t_start;
    let all_algos = generator::generate_algorithms_in_model(category, n_colors, class_L);
    let all_viable_algos = all_algos
        .filter(|a| a.all_gathered_are_stay())
        .filter(|a| a.all_colors_used_in_actions())
        .filter(|a| a.all_colors_used_in_non_gathered())
        .filter(|a| a.is_pseudo_canonical())
        .filter(|a| weak_filter || a.some_non_gathered_is_stay())
        .filter(|a| weak_filter || a.some_non_gathered_is_to_half())
        .filter(|a| weak_filter || a.some_non_gathered_is_to_other())
        .filter(|a| !retain_filter || a.retains_color_iif_other_color_different())
        .enumerate();

    let mut n_algos: usize = 0;
    let mut n_errors: usize = 0;
    let mut n_pass: usize = 0;
    let mut n_fail: usize = 0;
    let mut n_incomplete: usize = 0;

    let t_gen: Duration;
    let t_verif: Duration;
    let t_cleanup: Duration;

    let cleanup_outcome: Result<_>; // used later

    if cli.sequential {
        //
        // Sequential verification
        //
        let enclosure = runner::create_enclosure(workdir.path())?;

        info!("Starting verification");
        t_gen = Instant::now() - t_start;
        for (i, algo) in all_viable_algos {
            let outcome = run_verification(&enclosure, &algo, model_run_options)?;

            n_algos += 1;
            match outcome {
                SpinOutcome::Fail => n_fail += 1,
                SpinOutcome::Pass => n_pass += 1,
                SpinOutcome::SearchIncomplete => n_incomplete += 1,
            }
            if !outcome.is_fail() {
                writeln!(output)?;
                writeln!(output, "{:4} : {} {}", i, outcome, &algo.as_code())?;
            } else if (i + 1) % 100 == 0 {
                write!(output, "\n.")?;
            } else if (i + 1) % 10 == 0 {
                write!(output, ". ")?;
            } else {
                write!(output, ".")?;
            }
            output.flush()?;
        }
        t_verif = Instant::now() - t_start;
        t_cleanup = t_verif;
        cleanup_outcome = Ok(());
        // report and cleanup already done
    } else {
        //
        // Parallel verification
        //
        let all_viable_algos = all_viable_algos.collect::<Vec<_>>();

        let num_algos = all_viable_algos.len() as u64;

        t_gen = Instant::now() - t_start;

        // execute verification in parallel
        info!("Starting verification (parallel)");
        let outcomes = all_viable_algos
            .into_par_iter()
            .map(|(i, algo)| {
                with_enclosure_do(workdir.path(), {
                    |thread_enclosure| {
                        run_verification(thread_enclosure, &algo, model_run_options)
                            .map(|outcome| (i, algo.as_code(), outcome))
                    }
                })
            })
            .progress_count(num_algos)
            .collect::<Vec<_>>();

        info!("Cleaning up");
        // eject ramdisk (if any)
        t_verif = Instant::now() - t_start;
        cleanup_outcome = runner::close_workdir(workdir);

        // report PASS results / incomplete search / errors
        t_cleanup = Instant::now() - t_start;
        for res in outcomes.iter() {
            match res {
                Ok((i, algo_code, SpinOutcome::Pass)) => {
                    writeln!(output, "{:4} : PASS {}", i, algo_code)?;
                    output.flush()?;
                }
                Ok((i, algo_code, SpinOutcome::SearchIncomplete)) => {
                    writeln!(
                        output,
                        "INCOMPLETE > {:4} : SearchIncomplete {}",
                        i, algo_code
                    )?;
                    output.flush()?;
                }
                Ok(_) => { /* skip */ }
                Err(e) => {
                    writeln!(output, "ERROR : {:?}", e)?;
                }
            }
        }

        // count for reporting
        n_algos = num_algos as usize;
        n_errors = outcomes.iter().filter(|res| res.is_err()).count();
        n_pass = outcomes
            .iter()
            .filter_map(|res| res.as_ref().ok())
            .filter(|(_, _, o)| *o == SpinOutcome::Pass)
            .count();
        n_fail = outcomes
            .iter()
            .filter_map(|res| res.as_ref().ok())
            .filter(|(_, _, o)| *o == SpinOutcome::Fail)
            .count();
        n_incomplete = outcomes
            .iter()
            .filter_map(|res| res.as_ref().ok())
            .filter(|(_, _, o)| *o == SpinOutcome::SearchIncomplete)
            .count();
    }

    let t_report = Instant::now() - t_start;

    info!("Generating reports");
    // output verification summary
    writeln!(output, "Verification Finished with {n_pass} pass, {n_fail} fail, {n_incomplete} incomplete, {n_errors} errors ({n_algos} algorithms)")?;

    // output time report:
    // express all durations in millis
    let t_prepare = t_prepare.as_millis();
    let t_gen = t_gen.as_millis();
    let t_verif = t_verif.as_millis();
    let t_cleanup = t_cleanup.as_millis();
    let t_report = t_report.as_millis();
    // compute intervals
    let delta_prepare = t_prepare;
    let delta_gen = t_gen - t_prepare;
    let delta_verif = t_verif - t_gen;
    let delta_cleanup = t_cleanup - t_verif;
    let delta_report = t_report - t_cleanup;
    writeln!(output, "\nTiming report (Total: {} ms):", t_report)?;
    writeln!(
        output,
        "| unit: ms       | prepare | generate | verify | cleanup | report |"
    )?;
    writeln!(
        output,
        "| -------------- | ------- | -------- | ------ | ------- | ------ |"
    )?;
    writeln!(
        output,
        "| **cumulative** | {} | {} | {} | {} | {} |",
        t_prepare, t_gen, t_verif, t_cleanup, t_report
    )?;
    writeln!(
        output,
        "| **additive** | {} | {} | {} | {} | {} |",
        delta_prepare, delta_gen, delta_verif, delta_cleanup, delta_report
    )?;
    writeln!(output)?;
    writeln!(output, "Uname: {}", system_info())?;
    writeln!(output, "Num cpus: {}", num_cpus::get())?;
    writeln!(
        output,
        "OS/Arch: {} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    )?;
    output.flush()?;

    drop(output); // just to make sure that the file is closed before unwinding due to other failures.

    // now, the reporting file is closing:
    // delayed reporting of the cleanup error
    // this is to ensure that the reporting is saved before unrolling everything
    cleanup_outcome
}

fn system_info() -> String {
    duct::cmd!("uname", "-a")
        .read()
        .unwrap_or("<undetermined>".to_string())
}

/// Provides "tee" functionality (as the `tee` command in shell)
/// for any type implementing [std::io::Write].
struct Tee<A, B>
where
    A: std::io::Write,
    B: std::io::Write,
{
    writer_a: A,
    writer_b: B,
}

impl<A, B> Tee<A, B>
where
    A: std::io::Write,
    B: std::io::Write,
{
    pub fn new(writer_a: A, writer_b: B) -> Self {
        Self { writer_a, writer_b }
    }
}

impl<A, B> std::io::Write for Tee<A, B>
where
    A: std::io::Write,
    B: std::io::Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        use std::io::{Error, ErrorKind};
        let len_a = self.writer_a.write(buf)?;
        let len_b = self.writer_b.write(buf)?;
        if len_a == len_b {
            Ok(len_a)
        } else {
            Err(Error::new(
                ErrorKind::Other,
                format!("different length: {len_a} vs. {len_b}"),
            ))
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer_a.flush()?;
        self.writer_b.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::tests::*;
    use algorithm::*;
    use common::*;
    use runner::SpinOutcome;

    #[test]
    fn test_try_outcomes() {
        const TEST_VOLUME: &str = "TestRamDisk_try_outcomes";

        let num_colors = 2;
        let guards = guards_for_full_lights_2_cols();

        let workdir = runner::create_root_workdir(Some(TEST_VOLUME.into())).unwrap();
        let enclosure = runner::create_enclosure(workdir.path()).unwrap();
        let spin_options = promela::ModelRunOptions {
            scheduler: Scheduler::Centralized,
            rigid: false,
            quasi_ss: false,
        };

        let fail_algo = Algorithm::new(
            num_colors,
            &guards,
            &[
                Action(Color(0), Move::Stay),
                Action(Color(0), Move::Stay),
                Action(Color(0), Move::Stay),
                Action(Color(0), Move::Stay),
                Action(Color(0), Move::ToHalf),
                Action(Color(0), Move::ToHalf),
                Action(Color(0), Move::ToHalf),
                Action(Color(0), Move::ToHalf),
            ],
        );
        let pass_algo = Algorithm::new(
            num_colors,
            &guards,
            &[
                Action(Color(0), Move::Stay),
                Action(Color(0), Move::Stay),
                Action(Color(0), Move::Stay),
                Action(Color(0), Move::Stay),
                Action(Color(0), Move::ToOther),
                Action(Color(0), Move::ToOther),
                Action(Color(0), Move::ToOther),
                Action(Color(0), Move::ToOther),
            ],
        );

        let fail_outcome = run_verification(&enclosure, &fail_algo, spin_options).unwrap();
        println!("{:4} : {} {}", 0, fail_outcome, &fail_algo.as_code());

        let pass_outcome = run_verification(&enclosure, &pass_algo, spin_options).unwrap();
        println!("{:4} : {} {}", 1, pass_outcome, &pass_algo.as_code());

        runner::close_workdir(workdir).unwrap();

        assert_eq!(pass_outcome, SpinOutcome::Pass);
        assert_eq!(fail_outcome, SpinOutcome::Fail);
    }

    #[test]
    fn test_external() {
        use runner::*;

        const TEST_VOLUME: &str = "TestRamDisk_external";

        let num_colors = 3;
        let guards = guards_for_external_3_cols();

        let fail_algo = Algorithm::new(
            num_colors,
            &guards,
            &[
                Action(Color(0), Move::Stay),
                Action(Color(0), Move::Stay),
                Action(Color(0), Move::Stay),
                Action(Color(0), Move::ToOther),
                Action(Color(0), Move::ToHalf),
                Action(Color(0), Move::ToHalf),
            ],
        );

        println!("External(3):\n{}", promela::generate_promela(&fail_algo));

        let workdir = runner::create_root_workdir(Some(TEST_VOLUME.into())).unwrap();
        let enclosure = runner::create_enclosure(workdir.path()).unwrap();
        let spin_options = promela::ModelRunOptions {
            scheduler: Scheduler::ASYNC,
            rigid: false,
            quasi_ss: false,
        };

        let res = run_verification(&enclosure, &fail_algo, spin_options);

        runner::close_workdir(workdir).unwrap();

        if let Err(e) = &res {
            println!("{:?}", e);
        }
        assert!(res.is_ok());
    }

    #[test]
    fn test_full_lights() {
        use runner::*;

        const TEST_VOLUME: &str = "TestRamDisk_full_lights";

        let num_colors = 2;
        let guards = guards_for_full_lights_2_cols();

        // PASS S0_S0_S1_S1_S1_S0_O1_H0
        let pass_algo = Algorithm::new(
            num_colors,
            &guards,
            &[
                Action(Color(0), Move::Stay),
                Action(Color(0), Move::Stay),
                Action(Color(1), Move::Stay),
                Action(Color(1), Move::Stay),
                Action(Color(1), Move::Stay),
                Action(Color(0), Move::Stay),
                Action(Color(1), Move::ToOther),
                Action(Color(0), Move::ToHalf),
            ],
        );

        println!("FullLights(2):\n{}", promela::generate_promela(&pass_algo));

        let workdir = runner::create_root_workdir(Some(TEST_VOLUME.into())).unwrap();
        let enclosure = runner::create_enclosure(workdir.path()).unwrap();
        let spin_options = promela::ModelRunOptions {
            scheduler: Scheduler::ASYNC,
            rigid: false,
            quasi_ss: false,
        };

        let res = run_verification(&enclosure, &pass_algo, spin_options);

        runner::close_workdir(workdir).unwrap();
        match &res {
            Ok(outcome) => assert_eq!(outcome, &SpinOutcome::Pass),
            Err(e) => {
                println!("{:?}", e);
            }
        }
        assert!(res.is_ok());
    }

    #[test]
    fn test_rigid_quasi_ss() {
        use runner::*;

        const TEST_VOLUME: &str = "TestRamDisk_rigid_qss";

        let num_colors = 4;
        let guards = (0..num_colors)
            .map(Color)
            .map(Guard::LExternal)
            .collect::<Vec<_>>();

        // Algo H1_S2_O3_S0
        // Oku4ColsX
        let pass_algo = Algorithm::new(
            num_colors,
            &guards,
            &[
                Action(Color(1), Move::ToHalf),
                Action(Color(2), Move::Stay),
                Action(Color(3), Move::ToOther),
                Action(Color(0), Move::Stay),
            ],
        );

        println!("LExternal(4):\n{}", promela::generate_promela(&pass_algo));

        let workdir = runner::create_root_workdir(Some(TEST_VOLUME.into())).unwrap();
        let enclosure = runner::create_enclosure(workdir.path()).unwrap();
        let mut spin_options = promela::ModelRunOptions {
            scheduler: Scheduler::SSYNC,
            rigid: true,
            quasi_ss: true,
        };

        let res_rigid_qss = run_verification(&enclosure, &pass_algo, spin_options);

        spin_options.quasi_ss = false;
        let res_rigid_ss = run_verification(&enclosure, &pass_algo, spin_options);

        spin_options.rigid = false;
        let res_nrigid_ss = run_verification(&enclosure, &pass_algo, spin_options);

        runner::close_workdir(workdir).unwrap();
        match &res_rigid_qss {
            Ok(outcome) => assert_eq!(outcome, &SpinOutcome::Pass),
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }

        match &res_rigid_ss {
            Ok(outcome) => assert_eq!(outcome, &SpinOutcome::Fail),
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }

        match &res_nrigid_ss {
            Ok(outcome) => assert_eq!(outcome, &SpinOutcome::Fail),
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            }
        }
    }

    fn make_test_cli(
        category: ModelKind,
        n_colors: u8,
        class_L: bool,
        sequential: bool,
        scheduler: common::Scheduler,
        rigid: bool,
        quasi_ss: bool,
    ) -> Cli {
        #![allow(non_snake_case)]
        Cli {
            category,
            n_colors,
            class_L,
            sequential,
            scheduler,
            to_file: false,
            output_dir: None,
            ramdisk: None,
            weak_filter: false,
            retain_filter: false,
            rigid,
            quasi_ss,
        }
    }

    #[test]
    fn test_suggested_name() {
        let cli = make_test_cli(
            ModelKind::Full,
            2,
            true,
            false,
            Scheduler::ASYNC_LC_Atomic,
            false,
            false,
        );
        assert_eq!(suggested_name(&cli), "parout_L_full_2_async-lc-atomic.txt");

        let cli = make_test_cli(
            ModelKind::External,
            3,
            false,
            true,
            Scheduler::ASYNC_Move_Regular,
            false,
            false,
        );
        assert_eq!(
            suggested_name(&cli),
            "output_external_3_async-move-regular.txt"
        );

        let cli = make_test_cli(
            ModelKind::Full,
            2,
            true,
            false,
            Scheduler::ASYNC_LC_Atomic,
            true,
            false,
        );
        assert_eq!(
            suggested_name(&cli),
            "parout_L_full_2_async-lc-atomic_rigid.txt"
        );

        let cli = make_test_cli(
            ModelKind::Full,
            2,
            true,
            false,
            Scheduler::ASYNC_LC_Atomic,
            false,
            true,
        );
        assert_eq!(
            suggested_name(&cli),
            "parout_L_full_2_async-lc-atomic_qss.txt"
        );

        let cli = make_test_cli(
            ModelKind::Full,
            2,
            true,
            false,
            Scheduler::ASYNC_LC_Atomic,
            true,
            true,
        );
        assert_eq!(
            suggested_name(&cli),
            "parout_L_full_2_async-lc-atomic_rigid_qss.txt"
        );
    }
}
