use handlebars::{no_escape, Handlebars};
use once_cell::sync::Lazy;
use serde_json::{json, Value};

use crate::models::{Album, Song};

static HBS: Lazy<Handlebars> = Lazy::new(|| {
    let mut hbs = Handlebars::new();

    hbs.set_strict_mode(true);
    hbs.register_escape_fn(no_escape);

    hbs.register_template_string("album", include_str!("templates/album.toml.hbs"))
        .unwrap();
    hbs.register_template_string("song", include_str!("templates/song.toml.hbs"))
        .unwrap();
    hbs.register_template_string("tracklist", include_str!("templates/tracklist.toml.hbs"))
        .unwrap();

    hbs.register_helper("default-name", Box::new(helpers::default_name));
    hbs.register_helper("format-duration", Box::new(helpers::format_duration));
    hbs.register_helper("escape-quotes", Box::new(helpers::escape_quotes));

    hbs
});

#[derive(Default)]
pub struct Renderer;

impl Renderer {
    pub fn new() -> Renderer {
        Renderer::default()
    }

    pub fn render_album(&self, artist_id: &str, album: &Album) -> String {
        let data = json!({ "artist_id": artist_id, "album": album });
        HBS.render("album", &data).expect("failed to render album")
    }

    pub fn render_song(&self, song: &Song) -> String {
        let data = json!({ "song": song });
        HBS.render("song", &data).expect("failed to render song")
    }

    pub fn render_tracklist(&self, artist_id: &str, album: &Album) -> String {
        let data = json!({ "artist_id": artist_id, "album": album });
        HBS.render("tracklist", &data)
            .expect("failed to render tracklist")
    }
}

fn default_name(values: &[Value]) -> Option<String> {
    values
        .iter()
        .filter_map(|value| {
            value.as_object().map(|o| {
                let name = o["name"].as_str().unwrap_or("");
                let is_default = o["is_default"].as_bool().unwrap_or(false);
                (name, is_default)
            })
        })
        .find(|&(_, is_default)| is_default)
        .map(|(name, _)| name.to_owned())
}

fn escape_quotes(s: &str) -> String {
    s.replace('"', r#"\""#)
}

mod helpers {
    use handlebars::{
        Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError,
    };

    use super::{default_name as _default_name, escape_quotes as _escape_quotes};
    use crate::util::format_duration as _format_duration;

    pub fn default_name(
        h: &Helper<'_, '_>,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext<'_, '_>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let values = h
            .param(0)
            .and_then(|v| v.value().as_array())
            .ok_or_else(|| RenderError::new("default-name: first argument must be an array"))?;

        let name = _default_name(&values)
            .ok_or_else(|| RenderError::new("default-name: no default name found"))?;

        out.write(&name)?;

        Ok(())
    }

    pub fn format_duration(
        h: &Helper<'_, '_>,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext<'_, '_>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let t = h
            .param(0)
            .and_then(|v| v.value().as_i64())
            .map(|v| v as i32)
            .ok_or_else(|| RenderError::new("format-duration: first argument must be a number"))?;

        let duration = _format_duration(t);

        out.write(&duration)?;

        Ok(())
    }

    pub fn escape_quotes(
        h: &Helper<'_, '_>,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext<'_, '_>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let text = h
            .param(0)
            .and_then(|v| v.value().as_str())
            .ok_or_else(|| RenderError::new("escape-quotes: first argument must be a string"))?;

        let transformed = _escape_quotes(text);

        out.write(&transformed)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::models::{album, song, Name};

    use super::*;

    fn build_album() -> Album {
        let song_a = song::Builder::new()
            .set_position(3)
            .set_duration(266)
            .add_name(Name::new("잠 못 드는 밤 비는 내리고", "ko", true, false))
            .add_name(Name::new(
                "Jam Mot Deuneun Bam Bineun Naerigo",
                "ko-Latn",
                false,
                true,
            ))
            .add_name(Name::new("Sleepless Rainy Night", "en", false, false))
            .build();

        let song_b = song::Builder::new()
            .set_position(4)
            .set_duration(233)
            .add_name(Name::new("어젯밤 이야기", "ko", true, false))
            .add_name(Name::new("Eojetbam Iyagi", "ko-Latn", false, true))
            .add_name(Name::new("Last Night Story", "en", false, false))
            .build();

        album::Builder::new()
            .set_id("kkotgalpi-dul")
            .set_kind(album::Kind::Single)
            .set_country("KR")
            .set_released_on("2017-09-22")
            .set_artwork_url("https://lp.dev/assets/artwork.jpg")
            .set_url("https://lp.dev/albums/1")
            .add_name(Name::new("꽃갈피 둘", "ko", true, false))
            .add_name(Name::new("Kkotgalpi Dul", "ko-Latn", false, true))
            .add_name(Name::new("A Flower Bookmark 2", "en", false, false))
            .add_name(Name::new("I & U", "und", false, false))
            .add_song(song_a)
            .add_song(song_b)
            .build()
    }

    #[test]
    fn test_render_album() {
        let album = build_album();
        let renderer = Renderer::new();
        let result = renderer.render_album("iu", &album);
        let expected = fs::read_to_string("tests/snapshots/album.toml").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_render_song() {
        let album = build_album();
        let renderer = Renderer::new();
        let result = renderer.render_song(&album.songs[0]);
        let expected = fs::read_to_string("tests/snapshots/song.toml").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_render_tracklist() {
        let album = build_album();
        let renderer = Renderer::new();
        let result = renderer.render_tracklist("iu", &album);
        let expected = fs::read_to_string("tests/snapshots/tracklist.toml").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_default_name() {
        let data = json!([
            { "name": "우주소녀", "locale": "ko", "is_original": true, "is_default": false },
            { "name": "WJSN", "locale": "en", "is_original": false, "is_default": true },
            { "name": "Cosmic Girls", "locale": "en", "is_original": false, "is_default": false },
        ]);

        let values = data.as_array().unwrap();
        assert_eq!(default_name(&values), Some(String::from("WJSN")));

        let data = json!([]);
        let values = data.as_array().unwrap();
        assert!(default_name(&values).is_none());
    }

    #[test]
    fn test_escape_quotes() {
        assert_eq!(escape_quotes("As You Wish"), "As You Wish");
        assert_eq!(
            escape_quotes(r#"The year of "YES""#),
            r#"The year of \"YES\""#
        );
        assert_eq!(
            escape_quotes(r#"It's a "New Day""#),
            r#"It's a \"New Day\""#
        );
    }
}
