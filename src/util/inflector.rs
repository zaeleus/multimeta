use std::collections::HashSet;

use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use unidecode::unidecode;

static MINOR_WORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    vec![
        "a", "an", "as", "at", "but", "by", "for", "from", "in", "into", "of", "on", "sans",
        "than", "the", "to", "via", "with",
    ]
    .into_iter()
    .collect()
});

static ACRONYMS: Lazy<HashSet<&'static str>> =
    Lazy::new(|| vec!["dj", "ost"].into_iter().collect());

pub fn capitalize(s: &str) -> String {
    let mut chs = s.chars();

    match chs.next() {
        Some(ch) => ch.to_uppercase().chain(chs).collect(),
        None => String::new(),
    }
}

fn is_acronym(word: &str) -> bool {
    ACRONYMS.contains(word)
}

fn is_minor_word(word: &str) -> bool {
    MINOR_WORDS.contains(word)
}

pub fn titleize(s: &str) -> String {
    static FIRST_WORD_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\w+").unwrap());
    static REST_OF_WORDS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b([\w']+)").unwrap());

    let s = s.to_lowercase();

    let s = FIRST_WORD_RE.replace(&s, |caps: &Captures<'_>| capitalize(&caps[0]));

    let s = REST_OF_WORDS_RE.replace_all(&s, |caps: &Captures<'_>| {
        let word = &caps[1];

        if is_minor_word(word) {
            word.to_owned()
        } else if is_acronym(word) {
            word.to_uppercase()
        } else {
            capitalize(word)
        }
    });

    s.to_string()
}

// This works similarly to `ActiveSupport::Inflector.parameterize`.
pub fn parameterize(s: &str) -> String {
    static RE1: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)[^a-z0-9-_]+").unwrap());
    static RE2: Lazy<Regex> = Lazy::new(|| Regex::new(r"-{2,}").unwrap());

    let s = unidecode(s);
    let s = RE1.replace_all(&s, "-");
    let s = RE2.replace_all(&s, "-");
    s.trim_matches('-').to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("hello"), "Hello");
        assert_eq!(capitalize("oST"), "OST");
    }

    #[test]
    fn test_is_minor_word() {
        assert!(is_minor_word("a"));
        assert!(is_minor_word("the"));
        assert!(is_minor_word("for"));
        assert!(!is_minor_word("before"));
        assert!(!is_minor_word("around"));
    }

    #[test]
    fn test_is_acronym() {
        assert!(is_acronym("dj"));
        assert!(is_acronym("ost"));
        assert!(!is_acronym("wjsn"));
    }

    #[test]
    fn test_titleize() {
        assert_eq!(titleize("la la la"), "La La La");
        assert_eq!(
            titleize("neoui ondo (Remind of You)"),
            "Neoui Ondo (Remind of You)"
        );
        assert_eq!(
            titleize("See Saw (Feat. Kim Lip) (Chuu, Go Won)"),
            "See Saw (Feat. Kim Lip) (Chuu, Go Won)"
        );
        assert_eq!(titleize("Girl's Talk (이브, 츄)"), "Girl's Talk (이브, 츄)");
        assert_eq!(
            titleize("See Saw (Feat. kimlip) (chuu, gowon)"),
            "See Saw (Feat. Kimlip) (Chuu, Gowon)"
        );
        assert_eq!(
            titleize("kkumkkuneun maeumeuro (Chinese Ver.)"),
            "Kkumkkuneun Maeumeuro (Chinese Ver.)"
        );
        assert_eq!(
            titleize("himssenyeoja dobongsun OST Part.1"),
            "Himssenyeoja Dobongsun OST Part.1"
        );
        assert_eq!(titleize("#cheotsarang"), "#Cheotsarang");
        assert_eq!(
            titleize("Straight Up (Feat. DJ Wegun, kim hyo eun)"),
            "Straight Up (Feat. DJ Wegun, Kim Hyo Eun)"
        );
    }

    #[test]
    fn test_parameterize() {
        assert_eq!(
            parameterize("Kkumkkuneun Maeumeuro"),
            "kkumkkuneun-maeumeuro"
        );
        assert_eq!(
            parameterize("Kkumkkuneun Maeumeuro (Chinese Ver.)"),
            "kkumkkuneun-maeumeuro-chinese-ver"
        );
        assert_eq!(parameterize("꿈꾸는 마음으로"), "ggumgguneun-maeumeuro");
        assert_eq!(
            parameterize("꿈꾸는 마음으로 (Chinese Ver.)"),
            "ggumgguneun-maeumeuro-chinese-ver"
        );
        assert_eq!(parameterize("über"), "uber");
        assert_eq!(parameterize("\"foo\" / ~bar~"), "foo-bar");
        assert_eq!(parameterize("--foo-bar--"), "foo-bar");
    }
}
