use super::*;
use lazy_regex::regex_captures;


#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_snake_case)]
pub struct Model {
    pub category: ModelKind,
    pub n_colors: u8,
    pub class_L: bool,
}

impl From<(ModelKind, u8, bool)> for Model {
    #[allow(non_snake_case)]
    fn from((category, n_colors, class_L): (ModelKind, u8, bool)) -> Self {
        Self {
            category,
            n_colors,
            class_L,
        }
    }
}

impl TryFrom<&str> for Model {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        if let Some( (_, kind, n_cols, class_l) ) = regex_captures!(
            r"^(?P<kind>F|E|I)(?P<n_cols>\d+)(?P<class_L>L)?$",
            value
        ) {
            let kind = ModelKind::try_from(kind)?;
            let color = common::Color::try_from(n_cols)?;
            let class_l = class_l == "L";
            let model = Model::from((kind, color.0, class_l));
            Ok(model)
        } else {
            Err(anyhow::anyhow!("Invalid model string: {}", value))
        }
    }
}

impl TryFrom<String> for Model {
    type Error = anyhow::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_from_str() {
        for (model, expected) in &[
            ("F3", Model::from((ModelKind::Full, 3, false))),
            ("E3", Model::from((ModelKind::External, 3, false))),
            ("I3", Model::from((ModelKind::Internal, 3, false))),
            ("F3L", Model::from((ModelKind::Full, 3, true))),
            ("E3L", Model::from((ModelKind::External, 3, true))),
            ("I3L", Model::from((ModelKind::Internal, 3, true))),
            ("F10", Model::from((ModelKind::Full, 10, false))),
            ("E10", Model::from((ModelKind::External, 10, false))),
            ("I10", Model::from((ModelKind::Internal, 10, false))),
            ("F10L", Model::from((ModelKind::Full, 10, true))),
            ("E10L", Model::from((ModelKind::External, 10, true))),
            ("I10L", Model::from((ModelKind::Internal, 10, true))),
        ] {
            assert_eq!(Model::try_from(*model).unwrap(), *expected);
        }
    }
}
