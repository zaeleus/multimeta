use std::collections::HashSet;

use lazy_static::lazy_static;
use regex::{Captures, Regex};
use unidecode::unidecode;

lazy_static! {
    static ref MINOR_WORDS: HashSet<&'static str> = {
        vec![
            "a", "an", "as", "at", "but", "by", "for", "from",
            "in", "into", "of", "on", "sans", "than", "the", "to",
            "via", "with",
        ].into_iter().collect()
    };

    static ref ACRONYMS: HashSet<&'static str> = {
        vec!["dj", "ost"].into_iter().collect()
    };
}

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
    lazy_static! {
        static ref FIRST_WORD_RE: Regex = Regex::new(r"^\w+").unwrap();
        static ref REST_OF_WORDS_RE: Regex = Regex::new(r"\b([\w']+)").unwrap();
    }

    let s = s.to_lowercase();

    let s = FIRST_WORD_RE.replace(&s, |caps: &Captures| {
        capitalize(&caps[0])
    });

    let s = REST_OF_WORDS_RE.replace_all(&s, |caps: &Captures| {
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
    lazy_static! {
        static ref RE1: Regex = Regex::new(r"(?i)[^a-z0-9-_]+").unwrap();
        static ref RE2: Regex = Regex::new(r"-{2,}").unwrap();
    }

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
        assert_eq!(titleize("neoui ondo (Remind of You)"), "Neoui Ondo (Remind of You)");
        assert_eq!(titleize("See Saw (Feat. Kim Lip) (Chuu, Go Won)"), "See Saw (Feat. Kim Lip) (Chuu, Go Won)");
        assert_eq!(titleize("Girl's Talk (이브, 츄)"), "Girl's Talk (이브, 츄)");
        assert_eq!(titleize("See Saw (Feat. kimlip) (chuu, gowon)"), "See Saw (Feat. Kimlip) (Chuu, Gowon)");
        assert_eq!(titleize("kkumkkuneun maeumeuro (Chinese Ver.)"), "Kkumkkuneun Maeumeuro (Chinese Ver.)");
        assert_eq!(titleize("himssenyeoja dobongsun OST Part.1"), "Himssenyeoja Dobongsun OST Part.1");
        assert_eq!(titleize("#cheotsarang"), "#Cheotsarang");
        assert_eq!(titleize("Straight Up (Feat. DJ Wegun, kim hyo eun)"), "Straight Up (Feat. DJ Wegun, Kim Hyo Eun)");
    }

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
}
