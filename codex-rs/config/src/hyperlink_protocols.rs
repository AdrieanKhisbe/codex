//! User-selected additions to the terminal UI's built-in HTTP(S) hyperlink protocols.

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::de::Error as _;

/// Additional URI schemes, without a colon, matching `[A-Za-z][A-Za-z0-9+.-]*`.
/// Names are case-insensitive; duplicates are ignored. HTTP and HTTPS remain enabled.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct AdditionalHyperlinkProtocols(#[schemars(schema_with = "protocols_schema")] Vec<String>);

fn protocols_schema(generator: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
    let mut item = String::json_schema(generator).into_object();
    item.string().pattern = Some("^[A-Za-z][A-Za-z0-9+.-]*$".into());
    let mut schema = Vec::<String>::json_schema(generator).into_object();
    schema.array().items = Some(schemars::schema::SingleOrVec::Single(Box::new(item.into())));
    schema.into()
}

impl AdditionalHyperlinkProtocols {
    pub fn contains(&self, scheme: &str) -> bool {
        self.0
            .iter()
            .any(|enabled| enabled.eq_ignore_ascii_case(scheme))
    }
}

impl<'de> Deserialize<'de> for AdditionalHyperlinkProtocols {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut schemes = Vec::<String>::deserialize(deserializer)?;
        for scheme in &mut schemes {
            let mut bytes = scheme.bytes();
            if !bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
                || !bytes.all(|byte| byte.is_ascii_alphanumeric() || b"+.-".contains(&byte))
            {
                return Err(D::Error::custom(format!(
                    "invalid hyperlink protocol {scheme:?}: expected a URI scheme without a colon matching [A-Za-z][A-Za-z0-9+.-]*"
                )));
            }
            scheme.make_ascii_lowercase();
        }
        schemes.sort();
        schemes.dedup();
        Ok(Self(schemes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConfigLayerEntry;
    use crate::ConfigLayerSource;
    use crate::ConfigLayerStack;
    use codex_utils_absolute_path::AbsolutePathBuf;

    #[test]
    fn hyperlink_protocols_normalize_names_and_reject_invalid_schemes() {
        let protocols: AdditionalHyperlinkProtocols =
            serde_json::from_str(r#"["VSCODE", "vscode", "My+App.2-0"]"#).unwrap();
        assert!(protocols.contains("VsCode"));
        assert_eq!(
            serde_json::to_value(protocols).unwrap(),
            serde_json::json!(["my+app.2-0", "vscode"])
        );
        for invalid in [
            "",
            "1app",
            "vscode:",
            "file://",
            "with space",
            "é",
            " app",
            "a\n",
        ] {
            assert!(
                serde_json::from_value::<AdditionalHyperlinkProtocols>(serde_json::json!([
                    invalid
                ]))
                .is_err(),
                "{invalid:?}"
            );
        }
    }

    #[test]
    fn hyperlink_protocols_only_user_profile_and_cli_layers_can_replace_the_array() {
        let tmp = tempfile::tempdir().unwrap();
        let path = |name| AbsolutePathBuf::resolve_path_against_base(name, tmp.path());
        let layer = |name, protocols: &str| {
            ConfigLayerEntry::new(
                name,
                toml::from_str(&format!(
                    "[tui]\nadditional_hyperlink_protocols={protocols}"
                ))
                .unwrap(),
            )
        };
        let base = layer(
            ConfigLayerSource::User {
                file: path("config.toml"),
                profile: None,
            },
            "['vscode']",
        );
        let profile = layer(
            ConfigLayerSource::User {
                file: path("work.config.toml"),
                profile: Some("work".into()),
            },
            "['file']",
        );
        let project = layer(
            ConfigLayerSource::Project {
                dot_codex_folder: path(".codex"),
            },
            "['attacker']",
        );
        let stack =
            |layers| ConfigLayerStack::new(layers, Default::default(), Default::default()).unwrap();
        assert!(
            stack(vec![base.clone(), project.clone()])
                .additional_hyperlink_protocols()
                .unwrap()
                .contains("vscode")
        );
        let protocols = stack(vec![base.clone(), profile.clone(), project.clone()])
            .additional_hyperlink_protocols()
            .unwrap();
        assert!(protocols.contains("file"));
        assert!(!protocols.contains("vscode"));
        assert!(!protocols.contains("attacker"));
        let cli = layer(ConfigLayerSource::SessionFlags, "[]");
        assert_eq!(
            stack(vec![base, profile, project, cli])
                .additional_hyperlink_protocols()
                .unwrap(),
            AdditionalHyperlinkProtocols::default()
        );
    }
}
