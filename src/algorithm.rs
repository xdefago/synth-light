use itertools::{self, Itertools};
pub use strum::IntoEnumIterator;
use strum::{Display, EnumString};

use anyhow::{anyhow, bail, Context};

use crate::common::*;

#[derive(Eq, PartialEq, Debug, Clone, Copy, EnumString, Display, PartialOrd, Ord)]
pub enum Guard {
    LExternal(Color),             //< (other's color)
    LInternal(Color),             //< (my color)
    LFull(Color, Color),          //< (my color, other's color)
    External(Color, Distance),    //< (other's color, distance to other)
    Internal(Color, Distance),    //< (my color, distance to other)
    Full(Color, Color, Distance), //< (my color, other's color, distance to other)
}

impl Guard {
    pub fn model_kind(&self) -> crate::ModelKind {
        use Guard::*;
        match self {
            Full(_, _, _) | LFull(_, _) => crate::ModelKind::Full,
            External(_, _) | LExternal(_) => crate::ModelKind::External,
            Internal(_, _) | LInternal(_) => crate::ModelKind::Internal,
        }
    }

    #[allow(non_snake_case)]
    pub fn class_L(&self) -> bool {
        use Guard::*;
        matches!(self, LExternal(_) | LInternal(_) | LFull(_, _))
    }

    pub fn is_gathered(&self) -> bool {
        use Guard::*;
        matches!(
            self,
            External(_, d) | Internal(_, d) | Full(_, _, d) if d == &Distance::Same
        )
    }

    pub fn same_colors(&self) -> bool {
        use Guard::*;
        matches!(
            self,
            LExternal(_) | LInternal(_) | External(_, _) | Internal(_, _)
        ) || matches!(
            self,
            LFull(c1, c2) | Full(c1, c2, _) if c1 == c2
        )
    }

    pub fn my_color(&self) -> Option<Color> {
        use Guard::*;
        match self {
            LExternal(_) | External(_, _) => None,
            LInternal(c) | Internal(c, _) | LFull(c, _) | Full(c, _, _) => Some(*c),
        }
    }

    pub fn distance(&self) -> Option<Distance> {
        use Guard::*;
        match self {
            LExternal(_) | LInternal(_) | LFull(_, _) => None,
            External(_, d) | Internal(_, d) | Full(_, _, d) => Some(*d),
        }
    }

    pub fn other_color(&self) -> Option<Color> {
        use Guard::*;
        match self {
            LInternal(_) | Internal(_, _) => None,
            LExternal(c) | External(c, _) | LFull(_, c) | Full(_, c, _) => Some(*c),
        }
    }

    pub fn as_code(&self) -> String {
        use Guard::*;
        match self {
            LExternal(c) | LInternal(c) => format!("{}", c.0),
            LFull(c1, c2) => format!("{}{}", c1.0, c2.0),
            External(c, Distance::Same) | Internal(c, Distance::Same) => format!("{}s", c.0),
            External(c, _) | Internal(c, _) => format!("{}d", c.0),
            Full(c1, c2, Distance::Same) => format!("{}{}s", c1.0, c2.0),
            Full(c1, c2, _) => format!("{}{}d", c1.0, c2.0),
        }
    }

    pub fn try_parse(model: crate::ModelKind, class_l: bool, code: &str) -> anyhow::Result<Self> {
        use crate::ModelKind::*;
        if code.is_empty() || 3 < code.len() {
            bail!("wrong length for guard code: \"{code}\"");
        }
        match model {
            Full => {
                let c1 = code
                    .get(0..1)
                    .map(Color::try_from)
                    .ok_or_else(|| anyhow!("missing color 1"))??;
                let c2 = code
                    .get(1..2)
                    .map(Color::try_from)
                    .ok_or_else(|| anyhow!("missing color 2"))??;
                if class_l {
                    Ok(Guard::LFull(c1, c2))
                } else {
                    let d = code
                        .get(2..3)
                        .map(Distance::try_parse)
                        .ok_or_else(|| anyhow!("missing distance"))??;
                    Ok(Guard::Full(c1, c2, d))
                }
            }
            External | Internal if class_l => {
                let col = code
                    .get(0..1)
                    .map(Color::try_from)
                    .ok_or_else(|| anyhow!("missing color"))??;
                if model == External {
                    Ok(Guard::LExternal(col))
                } else {
                    Ok(Guard::LInternal(col))
                }
            }
            External | Internal => {
                let col = code
                    .get(0..1)
                    .map(Color::try_from)
                    .ok_or_else(|| anyhow!("missing color"))??;
                let d = code
                    .get(2..3)
                    .map(Distance::try_parse)
                    .ok_or_else(|| anyhow!("missing distance"))??;
                if model == External {
                    Ok(Guard::External(col, d))
                } else {
                    Ok(Guard::Internal(col, d))
                }
            }
        }
    }

    pub fn number_for_model(model: crate::ModelKind, num_colors: u8, class_l: bool) -> usize {
        use crate::ModelKind::*;
        let basic_count = match model {
            Full => num_colors as usize * num_colors as usize,
            Internal | External => num_colors as usize,
        };
        if class_l {
            basic_count
        } else {
            2 * basic_count
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Action(pub Color, pub Move); //<  Action(next color, movement)

impl Action {
    pub fn is_stationary(&self) -> bool {
        self.1 == Move::Stay
    }
    pub fn color(&self) -> Color {
        self.0
    }
    pub fn movement(&self) -> Move {
        self.1
    }
    pub fn as_code(&self) -> String {
        format!("{}{}", self.1.as_code(), self.0 .0)
    }

    pub fn try_parse(code: &str) -> anyhow::Result<Self> {
        if code.len() != 2 {
            bail!("wrong length for action: \"{}\"", code);
        }
        let mv = Move::try_from(&code[0..1]).context("parsing move for action")?;
        let col = Color::try_from(&code[1..]).context("parsing color for action")?;
        Ok(Action(col, mv))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rule(Guard, Action);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Algorithm {
    num_colors: u8,
    guards: Vec<Guard>,
    actions: Vec<Action>,
}

impl Algorithm {
    pub fn new(num_colors: u8, guards: &[Guard], actions: &[Action]) -> Self {
        let guards = guards.to_vec();
        let n_guards = guards.len();
        let actions: Vec<_> = Vec::from(actions);
        assert_eq!(actions.len(), n_guards);
        assert!(actions.iter().all(|Action(c, _)| c < &Color(num_colors)));
        Algorithm {
            num_colors,
            guards,
            actions,
        }
    }

    pub fn model_kind(&self) -> crate::ModelKind {
        self.guards[0].model_kind()
    }

    #[allow(non_snake_case)]
    pub fn class_L(&self) -> bool {
        self.guards[0].class_L()
    }

    pub fn try_parse(
        model: crate::ModelKind,
        num_colors: u8,
        class_l: bool,
        code: &str,
    ) -> anyhow::Result<Self> {
        let guards_actions: Vec<_> = code.split("__").collect();
        match guards_actions.as_slice() {
            #![allow(clippy::redundant_closure)]
            [guards_str, actions_str] => {
                let guards = guards_str
                    .split('_')
                    .map(|code| Guard::try_parse(model, class_l, code))
                    .collect::<Result<Vec<_>, _>>()?;
                let actions = actions_str
                    .split('_')
                    .map(|code| Action::try_parse(code))
                    .collect::<Result<Vec<_>, _>>()?;
                if guards.len() != actions.len() {
                    bail!(
                        "guards and actions have different lengths ({} guards, {} actions)",
                        guards.len(),
                        actions.len()
                    );
                }
                if guards.len() != Guard::number_for_model(model, num_colors, class_l) {
                    bail!(
                        "number of guards ({}) does not match model ({})",
                        guards.len(),
                        Guard::number_for_model(model, num_colors, class_l)
                    );
                }
                Ok(Algorithm::new(num_colors, &guards, &actions))
            }
            [_actions] => bail!("guards are missing"),
            _ => bail!("missing separation string (or too many)"),
        }
    }

    pub fn as_code(&self) -> String {
        #![allow(unstable_name_collisions)]
        static SEP: &str = "_";

        let guard_part = self
            .guards
            .iter()
            .map(|g| g.as_code())
            .intersperse(SEP.into())
            .collect::<String>();
        let action_part = self
            .actions
            .iter()
            .map(|a| a.as_code())
            .intersperse(SEP.into())
            .collect::<String>();
        format!("{}__{}", guard_part, action_part)
    }

    pub fn num_colors(&self) -> u8 {
        self.num_colors
    }

    pub fn rules(&self) -> impl Iterator<Item = (&Guard, &Action)> {
        self.guards.iter().zip(self.actions.iter())
    }

    /// checks if all gathered rules are stationary (i.e., [Move::Stay]).
    /// When the robots are already gathered, all moves ([Move::ToOther] and [Move::ToHalf]) are equivalent to [Move::Stay].
    pub fn all_gathered_are_stay(&self) -> bool {
        self.rules()
            .filter(|(g, _)| g.is_gathered())
            .all(|(_, a)| a.is_stationary())
    }

    /// checks if the algorithms contains a non-gathered rule such that the action is stationary (i.e., [Move::Stay]).
    /// An algorithm without such rule cannot achieve gathering under a centralized scheduler.
    pub fn some_non_gathered_is_stay(&self) -> bool {
        self.rules()
            .any(|(g, a)| a.is_stationary() && !g.is_gathered())
    }

    /// checks if the algorithm contains a non-gathered rule such that the action has a [Move::ToOther].
    /// An algorithm without such rule cannot achieve gathering under a centralized scheduler.
    pub fn some_non_gathered_is_to_other(&self) -> bool {
        self.rules()
            .any(|(g, Action(_, m))| m == &Move::ToOther && !g.is_gathered())
    }

    /// checks if the algorithm contains a non-gathered rule such that the action has a [Move::ToHalf].
    /// An algorithm without such rule cannot achieve gathering under an FSYNC scheduler.
    pub fn some_non_gathered_is_to_half(&self) -> bool {
        self.rules()
            .any(|(g, Action(_, m))| m == &Move::ToHalf && !g.is_gathered())
    }

    /// checks if all colors are used in the non-gathered actions.
    /// The rationale is that, if this is not the case, then gathering would be solvable with less colors,
    /// and such an algorithm is to be found in the lesser model already.
    pub fn all_colors_used_in_non_gathered(&self) -> bool {
        Color::iter_ncols(self.num_colors).all(|c| {
            self.rules()
                .any(|(g, Action(c2, _))| c2 == &c && !g.is_gathered())
        })
    }

    /// checks if all colors are used in the actions.
    /// The rationale is that, if this is not the case, then gathering would be solvable with less colors,
    /// and such an algorithm is to be found in the lesser model already.
    pub fn all_colors_used_in_actions(&self) -> bool {
        Color::iter_ncols(self.num_colors)
            .all(|c| self.actions.iter().any(|Action(c2, _)| c2 == &c))
    }

    /// checks whether the algorithm is in a canonical form with respect to its permutation class.
    /// The function is not exact in the sense that it will not return false for every non-canonical algorithm.
    /// On the other hand, it will return true for all canonical algorithms.
    /// The purpose is merely to use it as a best-effort filter to reduce the search space.
    pub fn is_pseudo_canonical(&self) -> bool {
        let non_gathered = self
            .rules()
            .filter(|(g, _)| !g.is_gathered())
            .collect::<Vec<_>>();
        let same_colors_same_sorted = non_gathered
            .iter()
            .filter(|(g, _)| g.same_colors())
            .map(|(_, a)| a.1)
            .fold((true, Move::Stay), |(res, ref_mv), mv| {
                (res && ref_mv <= mv, Move::max(ref_mv, mv))
            })
            .0;
        same_colors_same_sorted
    }

    /// checks whether the algorithm satisfies the following condition expressed by Viglietta (ALGOSENSOR 2013)
    /// "A robot retains its color if and only if it sees the other robot set to a different color."
    pub fn retains_color_iif_other_color_different(&self) -> bool {
        self.rules().all(|(&g, &a)| match g {
            Guard::LFull(my, _) | Guard::Full(my, _, _) =>
            // - a robot always change its color when the other robot has the same color
            {
                (g.same_colors() && a.color() != my)
                    ||
                    // - a robot never changes its color when the other robot has a different color
                    (! g.same_colors() && a.color() == my)
            }
            _ => true,
        })
    }
}

impl std::fmt::Debug for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Algo")
            .field("guards", &self.guards)
            .field("actions", &self.actions)
            .finish()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::generator::tests::*;

    #[test]
    fn test_pseudo_canonical() {
        let num_colors = 2;
        let guards = guards_for_full_lights_2_cols();
        let actions = [
            // gathered
            Action(Color(0), Move::Stay),
            Action(Color(1), Move::Stay),
            Action(Color(0), Move::Stay),
            Action(Color(1), Move::Stay),
            // non-gathered
            Action(Color(0), Move::Stay),
            Action(Color(1), Move::ToHalf),
            Action(Color(0), Move::Stay),
            Action(Color(1), Move::ToOther),
        ];
        let algo = Algorithm::new(num_colors, &guards, &actions);
        assert!(algo.all_colors_used_in_actions());
        assert!(algo.all_colors_used_in_non_gathered());
        assert!(algo.all_gathered_are_stay());
        assert!(algo.some_non_gathered_is_stay());
        assert!(algo.some_non_gathered_is_to_half());
        assert!(algo.some_non_gathered_is_to_other());
        assert!(algo.is_pseudo_canonical());

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
        assert!(algo.all_colors_used_in_actions());
        assert!(algo.all_colors_used_in_non_gathered());
        assert!(algo.all_gathered_are_stay());
        assert!(algo.some_non_gathered_is_stay());
        assert!(algo.some_non_gathered_is_to_half());
        assert!(algo.some_non_gathered_is_to_other());
        assert!(!algo.is_pseudo_canonical());
    }

    #[test]
    fn test_action() {
        let a1 = Action(Color(1), Move::Stay);
        let a2 = Action(Color(2), Move::ToHalf);
        let a3 = Action(Color(3), Move::ToOther);

        assert_eq!(a1.as_code(), "S1");
        assert_eq!(a2.as_code(), "H2");
        assert_eq!(a3.as_code(), "O3");
    }

    #[test]
    fn test_algorithm() {
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

        assert_eq!(
            algo.as_code(),
            "00s_01s_10s_11s_00d_01d_10d_11d__S0_S1_S0_S1_H0_H1_O0_S1"
        );
    }

    #[test]
    fn test_parse() {
        let num_colors = 2;
        let model = crate::ModelKind::Full;
        let guards = guards_for_full_lights_2_cols();
        let class_l = false;
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
        let algo_ref = Algorithm::new(num_colors, &guards, &actions);

        let code = "00s_01s_10s_11s_00d_01d_10d_11d__S0_S1_S0_S1_H0_H1_O0_S1";
        let algo = Algorithm::try_parse(model, num_colors, class_l, code);
        println!("algo: {:?}", algo);

        assert_eq!(algo.unwrap(), algo_ref);
    }
}
