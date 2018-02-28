use handlebars::Handlebars;
use serde_json::Value;

use models::{Album, Song};

lazy_static! {
    static ref HBS: Handlebars = {
        let mut hbs = Handlebars::new();

        hbs.set_strict_mode(true);

        hbs.register_template_string("album", include_str!("templates/album.toml.hbs")).unwrap();
        hbs.register_template_string("song", include_str!("templates/song.toml.hbs")).unwrap();
        hbs.register_template_string("tracklist", include_str!("templates/tracklist.toml.hbs")).unwrap();

        hbs.register_helper("default-name", Box::new(helpers::default_name));
        hbs.register_helper("parameterize", Box::new(helpers::parameterize));
        hbs.register_helper("format-duration", Box::new(helpers::format_duration));

        hbs
    };
}

pub struct Renderer;

impl Renderer {
    pub fn new() -> Renderer {
        Renderer
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

        let mut result = HBS.render("tracklist", &data).expect("failed to render tracklist");
        // Remove the consecutive trailing new line.
        result.pop();

        result
    }
}

fn default_name(values: &[Value]) -> Option<String> {
    values.iter()
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

mod helpers {
    use handlebars::{Handlebars, Helper, RenderContext, RenderError};

    use super::default_name as _default_name;
    use util::{
        parameterize as _parameterize,
        format_duration as _format_duration,
    };

    pub fn default_name(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
        let values = h.param(0)
            .and_then(|v| v.value().as_array())
            .ok_or(RenderError::new("default-name: first argument must be an array"))?;

        let name = _default_name(&values)
            .ok_or(RenderError::new("default-name: no default name found"))?;

        rc.writer.write(name.as_bytes())?;

        Ok(())
    }

    pub fn format_duration(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
        let t = h.param(0)
            .and_then(|v| v.value().as_i64())
            .map(|v| v as i32)
            .ok_or(RenderError::new("format-duration: first argument must be a number"))?;

        let duration = _format_duration(t);

        rc.writer.write(duration.to_string().as_bytes())?;

        Ok(())
    }

    pub fn parameterize(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
        let text = h.param(0)
            .and_then(|v| v.value().as_str())
            .ok_or(RenderError::new("parameterize: first argument must be a string"))?;

        let transformed = _parameterize(text);

        rc.writer.write(transformed.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::default_name;

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
}
