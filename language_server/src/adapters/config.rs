use std::collections::HashMap;

use hex_color::HexColor;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct UserColors {
    #[serde(default = "UserColors::default_background")]
    pub background: HexColor,
}

impl Default for UserColors {
    fn default() -> Self {
        Self {
            background: UserColors::default_background(),
        }
    }
}

impl UserColors {
    fn default_background() -> HexColor {
        HexColor::rgba(134, 134, 134, 255)
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "Config::desierialize_highlights")]
    pub highlights: HashMap<String, UserColors>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            highlights: Self::default_highlights(),
        }
    }
}

impl Config {
    fn default_highlights() -> HashMap<String, UserColors> {
        HashMap::from_iter([("TODO".to_owned(), UserColors::default())])
    }

    pub(crate) fn desierialize_highlights<'de, D>(
        deserializer: D,
    ) -> Result<HashMap<String, UserColors>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = HashMap::<String, UserColors>::deserialize(deserializer)?;

        if !v.is_empty() {
            return Ok(v);
        }

        Ok(Self::default_highlights())
    }

    pub fn parse_json(v: Value) -> Self {
        serde_json::from_value(v).unwrap_or_default()
    }
}

// grcov-excl-start
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn parse_colors_config_works() {
        let raw_json = json!({"background": "#818181"});
        let colors = serde_json::from_value::<UserColors>(raw_json).unwrap();

        assert_eq!(colors.background.split_rgb(), (129, 129, 129));
    }

    #[test]
    fn parse_null_config_works() {
        let raw_json = json!(null);
        let config = Config::parse_json(raw_json);

        assert_eq!(config.highlights.len(), 1);
    }

    #[test]
    fn parse_non_existing_config_works() {
        let raw_json = json!({});
        let config = Config::parse_json(raw_json);

        assert_eq!(config.highlights.len(), 1);
    }

    #[test]
    fn parse_empty_config_works() {
        let raw_json = json!({
            "highlights": {}
        });
        let config = Config::parse_json(raw_json);

        assert_eq!(config.highlights.len(), 1);
    }

    #[test]
    fn parse_keyword_empty_config_works() {
        let raw_json = json!({
            "highlights": {
                "TODO": {}
            }
        });
        let config = Config::parse_json(raw_json);
        let highlights = config.highlights;

        assert_eq!(
            highlights["TODO"].background,
            UserColors::default_background()
        );
    }

    #[test]
    fn parse_whole_config_works() {
        let raw_json = json!({
            "highlights": {
                "TODO": {
                    "background": "#ffffff",
                },
                "NOTE": {
                    "background": "#808080",
                },
                "COMMENT": {
                    "background": "#808080",
                },
                "FIXME": {
                    "background": "#b8b80e",
                },
                "BUG": {
                    "background": "#FF0000",
                },
            }
        });
        let config = Config::parse_json(raw_json);
        let highlights = config.highlights;

        assert_eq!(highlights["TODO"].background.split_rgb(), (255, 255, 255));
        assert_eq!(highlights["NOTE"].background.split_rgb(), (128, 128, 128));
        assert_eq!(
            highlights["COMMENT"].background.split_rgb(),
            (128, 128, 128)
        );
        assert_eq!(highlights["FIXME"].background.split_rgb(), (184, 184, 14));
        assert_eq!(highlights["BUG"].background.split_rgb(), (255, 0, 0));
    }
}
// grcov-excl-stop
