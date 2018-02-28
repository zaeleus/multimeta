use regex::Regex;
use unidecode::unidecode;

// This works similarly to `ActiveSupport::Inflector.parameterize`.
pub fn parameterize(s: &str) -> String {
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"(?i)[^a-z0-9-_]+").unwrap();
        static ref RE2: Regex = Regex::new(r"-{2,}").unwrap();
    }

    let s = unidecode(s);
    let s = RE1.replace_all(&s, "-");
    let s = RE2.replace_all(&s, "-");
    s.trim_matches('-').to_lowercase()
}

pub fn format_duration(t: i32) -> String {
    let minutes = t / 60;
    let seconds = t % 60;
    format!("{}:{:02}", minutes, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameterize() {
        assert_eq!(parameterize("Kkumkkuneun Maeumeuro"), "kkumkkuneun-maeumeuro");
        assert_eq!(parameterize("Kkumkkuneun Maeumeuro (Chinese Ver.)"), "kkumkkuneun-maeumeuro-chinese-ver");
        assert_eq!(parameterize("꿈꾸는 마음으로"), "ggumgguneun-maeumeuro");
        assert_eq!(parameterize("꿈꾸는 마음으로 (Chinese Ver.)"), "ggumgguneun-maeumeuro-chinese-ver");
        assert_eq!(parameterize("über"), "uber");
        assert_eq!(parameterize("\"foo\" / ~bar~"), "foo-bar");
        assert_eq!(parameterize("--foo-bar--"), "foo-bar");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(8), "0:08");
        assert_eq!(format_duration(32), "0:32");
        assert_eq!(format_duration(63), "1:03");
        assert_eq!(format_duration(207), "3:27");
        assert_eq!(format_duration(671), "11:11");
    }
}
