use crate::algorithm::*;
use crate::common::*;
use crate::ModelKind;

/// generates all algorithms for a given model.
///
/// # Arguments
///
/// * `model`    - kind of model considered ([`ModelKind`]).
/// * `n_colors` - number of colors
/// * `class_l`  - flag whether the model is limited to class L algorithms (`true`) or not (`false`)
///
/// # Notes
///
/// * Full:
///     * 2 colors -> 4704 viables
///     * 3 colors -> ...
/// * Full, class L:
///     * 2 colors -> 294 viables
///     * 3 colors -> ...
/// * External:
///     * 3 colors -> 162 viables
//.     * 4 colors -> ...
/// * External, class L:
///     * 4 colors -> 72 viables
///     * 5 colors -> 720 viables
///     * 6 colors -> 7200 viables  (down from ~34 millions)   
///
pub fn generate_algorithms_in_model(
    model: ModelKind,
    n_colors: u8,
    class_l: bool,
) -> impl Iterator<Item = Algorithm> {
    let colors = (0..n_colors).map(Color);
    let dist = [Distance::Same, Distance::Near].into_iter();

    let guards = match model {
        ModelKind::Full if class_l => {
            let my_cols = colors.clone();
            let other_cols = colors;
            itertools::iproduct!(my_cols, other_cols)
                .map(|(c1, c2)| Guard::LFull(c1, c2))
                .collect::<Vec<_>>()
        }
        ModelKind::Full => {
            let my_cols = colors.clone();
            let other_cols = colors;
            itertools::iproduct!(dist, my_cols, other_cols)
                .map(|(d, c1, c2)| Guard::Full(c1, c2, d))
                .collect::<Vec<_>>()
        }
        ModelKind::External if class_l => colors.map(Guard::LExternal).collect::<Vec<_>>(),
        ModelKind::External => {
            let other_cols = colors;
            itertools::iproduct!(dist, other_cols)
                .map(|(d, c)| Guard::External(c, d))
                .collect::<Vec<_>>()
        }
        ModelKind::Internal if class_l => colors.map(Guard::LInternal).collect::<Vec<_>>(),
        ModelKind::Internal => {
            let my_cols = colors;
            itertools::iproduct!(dist, my_cols)
                .map(|(d, c)| Guard::Internal(c, d))
                .collect::<Vec<_>>()
        }
    };

    let n_guards = guards.len();

    let all_actions_iter = (1..n_guards).fold::<Box<dyn Iterator<Item = Vec<_>>>, _>(
        Box::new(
            itertools::iproduct!(Move::iter(), Color::iter_ncols(n_colors))
                .map(|(m, c)| vec![Action(c, m)]),
        ),
        |accum, _| {
            Box::new(
                itertools::iproduct!(
                    accum,
                    itertools::iproduct!(Move::iter(), Color::iter_ncols(n_colors))
                        .map(|(m, c)| Action(c, m))
                )
                .map::<Vec<_>, _>(|(v, a)| {
                    let mut v = v;
                    v.push(a);
                    v
                }),
            )
        },
    );

    all_actions_iter
        .map::<Algorithm, _>(move |actions| Algorithm::new(n_colors, &guards, actions.as_slice()))
}

pub fn count_algorithms_in_model(model: ModelKind, n_colors: u8, class_l: bool) -> u64 {
    let n_moves = 3;
    match model {
        ModelKind::Full => {
            let num_guards = n_colors as u32 * n_colors as u32;
            let in_class_l = u64::pow(n_colors as u64, num_guards) * u64::pow(n_moves, num_guards);
            if class_l {
                in_class_l
            } else {
                in_class_l * in_class_l
            }
        }
        ModelKind::Internal | ModelKind::External => {
            let num_guards = n_colors as u32;
            let in_class_l = u64::pow(n_colors as u64, num_guards) * u64::pow(n_moves, num_guards);
            if class_l {
                in_class_l
            } else {
                in_class_l * in_class_l
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn guards_for_full_lights_2_cols() -> Vec<Guard> {
        vec![
            Guard::Full(Color(0), Color(0), Distance::Same),
            Guard::Full(Color(0), Color(1), Distance::Same),
            Guard::Full(Color(1), Color(0), Distance::Same),
            Guard::Full(Color(1), Color(1), Distance::Same),
            //
            Guard::Full(Color(0), Color(0), Distance::Near),
            Guard::Full(Color(0), Color(1), Distance::Near),
            Guard::Full(Color(1), Color(0), Distance::Near),
            Guard::Full(Color(1), Color(1), Distance::Near),
        ]
    }

    pub fn guards_for_external_3_cols() -> Vec<Guard> {
        vec![
            Guard::External(Color(0), Distance::Same),
            Guard::External(Color(1), Distance::Same),
            Guard::External(Color(2), Distance::Same),
            //
            Guard::External(Color(0), Distance::Near),
            Guard::External(Color(1), Distance::Near),
            Guard::External(Color(2), Distance::Near),
        ]
    }

    #[test]
    fn test_action_iter() {
        const FIRST_FIVE: [&str; 5] = [
            "00s_01s_10s_11s_00d_01d_10d_11d__S0_S0_S0_S0_S0_S0_H0_O1",
            "00s_01s_10s_11s_00d_01d_10d_11d__S0_S0_S0_S0_S0_S0_H1_O0",
            "00s_01s_10s_11s_00d_01d_10d_11d__S0_S0_S0_S0_S0_S0_H1_O1",
            "00s_01s_10s_11s_00d_01d_10d_11d__S0_S0_S0_S0_S0_S0_O0_H1",
            "00s_01s_10s_11s_00d_01d_10d_11d__S0_S0_S0_S0_S0_S0_O1_H0",
        ];

        let mut count_0: usize = 0;
        let mut count_1: usize = 0;
        let mut count_2: usize = 0;
        let mut count_3: usize = 0;
        let mut count_4: usize = 0;
        let mut count_5: usize = 0;
        let mut count_6: usize = 0;
        let mut count_7: usize = 0;

        let algo_vec = generate_algorithms_in_model(ModelKind::Full, 2, false)
            .inspect(|_| count_0 += 1)
            .filter(|a| a.all_gathered_are_stay())
            .inspect(|_| count_1 += 1)
            .filter(|a| a.all_colors_used_in_actions())
            .inspect(|_| count_2 += 1)
            .filter(|a| a.all_colors_used_in_non_gathered())
            .inspect(|_| count_3 += 1)
            .filter(|a| a.some_non_gathered_is_stay())
            .inspect(|_| count_4 += 1)
            .filter(|a| a.some_non_gathered_is_to_half())
            .inspect(|_| count_5 += 1)
            .filter(|a| a.some_non_gathered_is_to_other())
            .inspect(|_| count_6 += 1)
            .filter(|a| a.is_pseudo_canonical())
            .inspect(|_| count_7 += 1)
            .collect::<Vec<_>>();

        for (i, algo) in algo_vec.iter().take(5).enumerate() {
            assert_eq!(algo.as_code(), FIRST_FIVE[i]);
        }

        //		eprintln!("Filtering algos:\n- stage 0: {count_0}\n- stage 1: {count_1}\n- stage 2: {count_2}\n- stage 3: {count_3}\n- stage 4: {count_4}\n- stage 5: {count_5}\n- stage 6: {count_6}\n- stage 7: {count_7}");
        assert_eq!(count_0, 1679616);
        assert_eq!(count_1, 20736);
        assert_eq!(count_2, 20574);
        assert_eq!(count_3, 18144);
        assert_eq!(count_4, 14560);
        assert_eq!(count_5, 11200);
        assert_eq!(count_6, 8064);
        assert_eq!(count_7, 4704);
    }

    #[test]
    fn test_count_algorithms() {
        let test_cases = [
            ((ModelKind::Full, 2, false), 1_679_616),
            ((ModelKind::Full, 2, true), 1_296),
            ((ModelKind::Full, 3, true), 387_420_489),
            ((ModelKind::External, 4, true), 20_736),
            ((ModelKind::External, 7, true), 1_801_088_541),
            ((ModelKind::External, 4, false), 429_981_696),
        ];

        for ((model, n_colors, class_l), expected) in test_cases {
            assert_eq!(
                count_algorithms_in_model(model, n_colors, class_l),
                expected
            );
        }
    }
}
