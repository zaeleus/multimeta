use crate::editor::NameInput;

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct Name {
    pub name: String,
    pub locale: String,
    pub is_original: bool,
    pub is_default: bool,
}

impl Name {
    pub fn new<S, T>(name: S, locale: T, is_original: bool, is_default: bool) -> Name
    where
        S: Into<String>,
        T: Into<String>,
    {
        Name {
            name: name.into(),
            locale: locale.into(),
            is_original,
            is_default,
        }
    }
}

impl From<NameInput> for Name {
    fn from(input: NameInput) -> Name {
        Name {
            name: input.name,
            locale: input.locale,
            is_original: input.is_original,
            is_default: input.is_default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Name;

    #[test]
    fn test_new() {
        let name = Name::new("이달의 소녀", "ko", true, false);
        assert_eq!(name.name, "이달의 소녀");
        assert_eq!(name.locale, "ko");
        assert_eq!(name.is_original, true);
        assert_eq!(name.is_default, false);
    }
}
