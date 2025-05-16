use clap::Parser;
use num_format::{Locale, ToFormattedString};

use synth_lights::{self, generator, ModelKind};

use indicatif::ProgressIterator;

///
/// Generates all algorithms for a given model and counts them at each stage of filtering.
///
#[derive(Debug, Parser)]
#[clap(author, version, about="Generates all algorithms for a given model and counts them at each stage of filtering.", long_about = None)]
#[allow(non_snake_case)]
pub struct Cli {
    /// Category of algorithms
    #[clap(value_enum)]
    category: ModelKind,

    /// Number of colors allowed in the model
    #[clap()]
    n_colors: u8,

    #[clap(long = "latex")]
    as_latex: bool,

    /// class L algorithms
    #[clap(short = 'L')]
    class_L: bool,

    /// Enables weak filtering
    #[clap(short = 'w')]
    weak_filter: bool,

    /// Enables Viglietta's retain rule filtering ("A robot retains its color if and only if it sees the other robot set to a different color.")
    #[clap(short = 'R')]
    retain_filter: bool,
}

fn main() {
    let cli = Cli::parse();

    // using an array to circumvent the limitations of the inept borrow checker
    // isn't worth the trouble, therefore copy-paste will do instead of array.
    let mut count_0: usize = 0;
    let mut count_1: usize = 0;
    let mut count_2: usize = 0;
    let mut count_3: usize = 0;
    let mut count_4: usize = 0;
    let mut count_5: usize = 0;
    let mut count_6: usize = 0;
    let mut count_7: usize = 0;
    let mut count_8: usize = 0;

    let weak_filter = cli.weak_filter;
    let retain_filter = cli.retain_filter;
    let total_algos = generator::count_algorithms_in_model(cli.category, cli.n_colors, cli.class_L);

    let all_algos =
        generator::generate_algorithms_in_model(cli.category, cli.n_colors, cli.class_L);
    let all_viable_algos = all_algos
        .progress_count(total_algos)
        .inspect(|_| count_0 += 1)
        .filter(|a| a.all_gathered_are_stay())
        .inspect(|_| count_1 += 1)
        .filter(|a| a.all_colors_used_in_actions())
        .inspect(|_| count_2 += 1)
        .filter(|a| a.all_colors_used_in_non_gathered())
        .inspect(|_| count_3 += 1)
        .filter(|a| a.is_pseudo_canonical())
        .inspect(|_| count_4 += 1)
        .filter(|a| weak_filter || a.some_non_gathered_is_stay())
        .inspect(|_| count_5 += 1)
        .filter(|a| weak_filter || a.some_non_gathered_is_to_half())
        .inspect(|_| count_6 += 1)
        .filter(|a| weak_filter || a.some_non_gathered_is_to_other())
        .inspect(|_| count_7 += 1)
        .filter(|a| !retain_filter || a.retains_color_iif_other_color_different())
        .inspect(|_| count_8 += 1);
    let _ = all_viable_algos.collect::<Vec<_>>();

    if cli.as_latex {
        let class_l = if cli.class_L { "$\\mathcal{L}$" } else { "" };
        let kind = cli.category.to_string().to_lowercase();
        let n_colors = cli.n_colors;
        let model_name = format!("{kind} {n_colors} {class_l}");

        println!(" & {} \\\\ \\hline", model_name);
        println!("ALL                               & {:>7} \\\\", count_0);
        println!("all gathered are stay             & {:>7} \\\\", count_1);
        println!("all colors used in actions        & {:>7} \\\\", count_2);
        println!("all colors used in non-gathered   & {:>7} \\\\", count_3);
        println!("is pseudo-canonical               & {:>7} \\\\", count_4);
        if !weak_filter {
            println!("$\\exists$ non-gathered is stay    & {:>7} \\\\", count_5);
            println!("$\\exists$ non-gathered is to-half & {:>7} \\\\", count_6);
            println!("$\\exists$ non-gathered is to-other& {:>7} \\\\", count_7);
        }
        if retain_filter {
            println!("retains color iif other is different & {:>7} \\\\", count_8);
        }
    } else {
        println!(
            "Model: {} {}-colors {}",
            cli.category,
            cli.n_colors,
            if cli.class_L { "class L" } else { "" }
        );
        println!();
        println!(
            "TOTAL:                          {:>11}",
            count_0.to_formatted_string(&Locale::en)
        );
        println!(
            "all_gathered_are_stay():        {:>11}",
            count_1.to_formatted_string(&Locale::en)
        );
        println!(
            "all_colors_used_in_actions:     {:>11}",
            count_2.to_formatted_string(&Locale::en)
        );
        println!(
            "all_colors_used_in_non_gathered:{:>11}",
            count_3.to_formatted_string(&Locale::en)
        );
        println!(
            "is_pseudo_canonical:            {:>11}",
            count_4.to_formatted_string(&Locale::en)
        );
        if !weak_filter {
            println!(
                "some_non_gathered_is_stay:      {:>11}",
                count_5.to_formatted_string(&Locale::en)
            );
            println!(
                "some_non_gathered_is_to_half:   {:>11}",
                count_6.to_formatted_string(&Locale::en)
            );
            println!(
                "some_non_gathered_is_to_other:  {:>11}",
                count_7.to_formatted_string(&Locale::en)
            );
        }
        if retain_filter {
            println!(
                "retains_color_iif_other_color_different:{:>11}",
                count_8.to_formatted_string(&Locale::en)
            )
        }
    }
}
