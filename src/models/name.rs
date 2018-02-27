#[derive(Debug, Eq, PartialEq)]
pub struct Name {
    pub name: String,
    pub locale: String,
    pub is_original: bool,
    pub is_default: bool,
}

impl Name {
    pub fn new(name: &str, locale: &str, is_original: bool, is_default: bool) -> Name {
        Name {
            name: name.to_owned(),
            locale: locale.to_owned(),
            is_original: is_original,
            is_default: is_default,
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
