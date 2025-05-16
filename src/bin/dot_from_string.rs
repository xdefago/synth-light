use clap::Parser;

use dot_writer::{Attributes, Color, DotWriter, Style};
use synth_lights::{
    self,
    algorithm::Algorithm,
    common::{Color as AlgoColor, Move},
    ModelKind,
};

#[derive(Debug, Parser)]
#[clap(author, version, about="Generates the dot code of an algorithm given its code string (e.g., 0_1_2__S2_H0_O1)", long_about = None)]
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

fn movement(mv: Move) -> String {
    match mv {
        Move::Stay => "Stay",
        Move::ToHalf => "Half",
        Move::ToOther => "Other",
    }
    .to_string()
}

fn algo_to_dot(algorithm: &Algorithm) -> String {
    let mut output_bytes = Vec::new();
    {
        let mut writer = DotWriter::from(&mut output_bytes);
        writer.set_pretty_print(true);

        let mut digraph = writer.digraph();
        digraph
            .node_attributes()
            .set_style(Style::Filled)
            .set_color(Color::LightGrey);
        digraph
            .graph_attributes()
            .set_label(&format!(
                "{} {} {}\n{}",
                algorithm.model_kind(),
                algorithm.num_colors(),
                if algorithm.class_L() { "L" } else { "" },
                algorithm.as_code()
            ))
            .set_font("monospace");

        for (guard, action) in algorithm.rules() {
            let current_states = if let Some(c) = guard.my_color() {
                vec![c]
            } else {
                AlgoColor::iter_ncols(algorithm.num_colors()).collect()
            };
            let move_action = movement(action.movement());
            let color_to = action.color();
            let label = match (guard.other_color(), guard.is_gathered()) {
                (Some(c), true) if !algorithm.class_L() => format!("({}G):{}", c, move_action),
                (Some(c), _) => format!("({}):{}", c, move_action),
                (None, true) if !algorithm.class_L() => format!("G:{}", move_action),
                (None, _) => format!("{}", move_action),
            };

            for color_from in current_states {
                digraph
                    .edge(color_from.to_string(), color_to.to_string())
                    .attributes()
                    .set_label(&label);
            }
        }
    }
    String::from_utf8(output_bytes).unwrap()
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let algorithm = Algorithm::try_parse(cli.category, cli.n_colors, cli.class_L, &cli.algorithm)?;
    let dot_code = algo_to_dot(&algorithm);

    println!("# Algorithm: {}", algorithm.as_code());

    println!();

    println!("{}", dot_code);
    Ok(())
}
