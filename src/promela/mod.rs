use anyhow::Result;
use itertools::{self, Itertools};
use std::fs;
use std::include_str;
use std::path::{Path, PathBuf};

use crate::algorithm::{Action, Algorithm, Guard};
use crate::common::*;

const ALGORITHM_FILE: &str = "Algorithms.pml";

const MAIN_PML: &str = include_str!("MainGathering.pml");
const ROBOTS_PML: &str = include_str!("Robots.pml");
const SCHEDULERS_PML: &str = include_str!("Schedulers.pml");
const TYPES_PML: &str = include_str!("Types.pml");

pub const PML_FILES: [(&str, &str); 4] = [
    ("MainGathering.pml", MAIN_PML),
    ("Robots.pml", ROBOTS_PML),
    ("Schedulers.pml", SCHEDULERS_PML),
    ("Types.pml", TYPES_PML),
];

#[derive(Clone, Copy, Debug)]
pub struct ModelRunOptions {
    pub scheduler: Scheduler,
    pub rigid: bool,
    pub quasi_ss: bool,
}

impl IntoIterator for ModelRunOptions {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut args = Vec::with_capacity(3);
        args.push(format!("-DSCHEDULER={}", self.scheduler.as_promela()));
        if self.rigid {
            args.push("-DMOVEMENT=RIGID".to_string());
        }
        if self.quasi_ss {
            args.push("-DQUASISS".to_string());
        }
        args.into_iter()
    }
}

pub fn prepare_promela_code(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow::Error::msg(format!(
            "Location not found: {:?}",
            path
        )));
    }
    if !path.is_dir() {
        return Err(anyhow::Error::msg(format!(
            "Location is not a directory: {:?}",
            path
        )));
    }
    for (name, content) in PML_FILES {
        let mut file_path = PathBuf::new();
        file_path.push(path);
        file_path.push(name);
        fs::write(file_path, content)?;
    }
    Ok(())
}

pub fn install_algorithm(path: &Path, algo: &Algorithm) -> Result<()> {
    let promela = generate_promela(algo);
    install_algorithm_from_code(path, &promela)
}

pub fn install_algorithm_from_code(path: &Path, promela: &str) -> Result<()> {
    let mut file_path = path.to_path_buf();
    file_path.push(ALGORITHM_FILE);
    let file_path = file_path.as_path();

    std::fs::write(file_path, promela)?;
    Ok(())
}

fn promela_rule(rule: (&Guard, &Action)) -> String {
    match rule {
        (Guard::Full(s,o,Distance::Same), Action(c,m)) =>
            format!("    :: (obs.color.me == {s}) && (obs.color.other == {o}) && (obs.same_position) -> command.move = {m}; command.new_color = {c};"),
        (Guard::Full(s,o,_), Action(c,m)) =>
            format!("    :: (obs.color.me == {s}) && (obs.color.other == {o}) && ! (obs.same_position) -> command.move = {m}; command.new_color = {c};"),
        //
        (Guard::Internal(s,Distance::Same), Action(c,m)) =>
            format!("    :: (obs.color.me == {s}) && (obs.same_position) -> command.move = {m}; command.new_color = {c};"),
        (Guard::Internal(s,_), Action(c,m)) =>
            format!("    :: (obs.color.me == {s}) && ! (obs.same_position) -> command.move = {m}; command.new_color = {c};"),
        //
        (Guard::External(o,Distance::Same), Action(c,m)) =>
            format!("    :: (obs.color.other == {o}) && (obs.same_position) -> command.move = {m}; command.new_color = {c};"),
        (Guard::External(o,_), Action(c,m)) =>
            format!("    :: (obs.color.other == {o}) && ! (obs.same_position) -> command.move = {m}; command.new_color = {c};"),
        //
        (Guard::LFull(s,o), Action(c,m)) =>
            format!("    :: (obs.color.me == {s}) && (obs.color.other == {o}) -> command.move = {m}; command.new_color = {c};"),
        //
        (Guard::LInternal(s), Action(c,m)) =>
            format!("    :: (obs.color.me == {s}) -> command.move = {m}; command.new_color = {c};"),
        //
        (Guard::LExternal(o), Action(c,m)) =>
            format!("    :: (obs.color.other == {o}) -> command.move = {m}; command.new_color = {c};"),

    }
}
pub fn generate_promela(algo: &Algorithm) -> String {
    #![allow(unstable_name_collisions)]
    let rules: String = algo
        .rules()
        .map(promela_rule)
        .intersperse("\n".into())
        .collect();
    let body: String = ["    if", &rules, "    fi;"]
        .into_iter()
        .intersperse("\n")
        .collect();
    let num_colors = algo.num_colors();
    let code = algo.as_code();
    format!(
        r##"
#ifndef __ALGORITHMS_PML__
#define __ALGORITHMS_PML__
#  define ALGO_NAME      "ALGO_SYNTH_{code}"
#  define Algorithm(o,c) Alg_Synth(o,c)
#  define MAX_COLOR      ({num_colors})
#  define NUM_COLORS     ({num_colors})
inline Alg_Synth(obs, command)
{{
    command.move      = STAY;
    command.new_color = obs.color.me;
{body}
}}
#endif
"##
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::{Action, Algorithm};
    use crate::generator::tests::*;

    #[test]
    fn test_promela_files() {
        let num_colors = 2;
        let guards = guards_for_full_lights_2_cols();
        let actions = [
            // gathered
            Action(Color(0), Move::Stay),
            Action(Color(1), Move::Stay),
            Action(Color(0), Move::Stay),
            Action(Color(1), Move::Stay),
            // non-gathered
            Action(Color(0), Move::ToHalf),
            Action(Color(1), Move::ToHalf),
            Action(Color(0), Move::ToOther),
            Action(Color(1), Move::Stay),
        ];
        let algo = Algorithm::new(num_colors, &guards, &actions);
        println!("{}", generate_promela(&algo));
    }

    #[test]
    fn test_promela_gen() {
        let num_colors = 2;
        let guards = guards_for_full_lights_2_cols();

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
                Action(Color(0), Move::Stay),
                Action(Color(0), Move::Stay),
            ],
        );

        let fail_code = generate_promela(&fail_algo);
        println!("Fail Algo: {}", fail_algo.as_code());
        println!("{}", fail_code);

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

        let pass_code = generate_promela(&pass_algo);
        println!("Pass Algo: {}", pass_algo.as_code());
        println!("{}", pass_code);

        let num_colors = 3;
        let guards = &guards_for_external_3_cols();

        let external_algo = Algorithm::new(
            num_colors,
            &guards,
            &[
                Action(Color(0), Move::Stay),
                Action(Color(0), Move::Stay),
                Action(Color(0), Move::Stay),
                Action(Color(0), Move::Stay),
                Action(Color(0), Move::ToHalf),
                Action(Color(0), Move::ToHalf),
            ],
        );

        let external_code = generate_promela(&external_algo);
        println!("External Algo: {}", external_algo.as_code());
        println!("{}", external_code);
    }
}
