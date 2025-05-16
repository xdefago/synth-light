use clap::Parser;

use synth_lights::{self, algorithm::Algorithm, ModelKind};

#[derive(Debug, Parser)]
#[clap(author, version, about="Generates the Promela code of an algorithm given its code string (e.g., 0_1_2__S2_H0_O1)", long_about = None)]
#[allow(non_snake_case)]
pub struct Cli {
    /// Category of algorithms
    #[clap(value_enum)]
    category: ModelKind,

    /// Number of colors allowed in the model
    #[clap()]
    n_colors: u8,

    /// Algorithm code string (e.g., 0_1_2__S2_H0_O1)
    #[clap()]
    algorithm: String,

    /// Class L algorithms
    #[clap(short = 'L')]
    class_L: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let algorithm = Algorithm::try_parse(cli.category, cli.n_colors, cli.class_L, &cli.algorithm)?;
    let promela = synth_lights::promela::generate_promela(&algorithm);

    println!("# Algorithm: {}", algorithm.as_code());

    println!();

    println!("{}", promela);
    Ok(())
}
