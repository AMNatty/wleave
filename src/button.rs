use serde::de::{IntoDeserializer, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;
use std::str::FromStr;
use tracing::warn;

#[derive(Debug, Clone, Deserialize)]
pub enum WButtonActionHandler {
    #[serde(rename = "shell")]
    Shell(String),
    #[serde(rename = "executable")]
    Executable(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct WButtonAction {
    #[serde(flatten)]
    pub handler: WButtonActionHandler,
    #[serde(flatten)]
    pub conditions: BTreeMap<String, String>,
}

impl WButtonAction {
    pub fn is_applicable(&self) -> bool {
        if let WButtonActionHandler::Executable(exe) = &self.handler
            && let Err(e) = which::which(exe)
        {
            warn!("Executable {} not available, skipping: {}", exe, e);
            return false;
        }

        for (key, value) in self.conditions.iter() {
            if let Some(env_var) = key.strip_prefix("$") {
                let Ok(var) = std::env::var(env_var) else {
                    return false;
                };

                if var != *value {
                    return false;
                }
            }
        }

        true
    }
}

impl FromStr for WButtonAction {
    type Err = ();

    fn from_str(action: &str) -> Result<Self, Self::Err> {
        Ok(WButtonAction {
            handler: WButtonActionHandler::Shell(action.to_string()),
            conditions: Default::default(),
        })
    }
}

struct WButtonActionVisitor;

impl<'de> Visitor<'de> for WButtonActionVisitor {
    type Value = WButtonAction;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("string or map")
    }

    fn visit_str<E>(self, value: &str) -> Result<WButtonAction, E>
    where
        E: serde::de::Error,
    {
        Ok(FromStr::from_str(value).expect("shell script always valid"))
    }

    fn visit_string<E>(self, val: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&val)
    }

    fn visit_map<M>(self, map: M) -> Result<WButtonAction, M::Error>
    where
        M: serde::de::MapAccess<'de>,
    {
        Deserialize::deserialize(serde::de::value::MapAccessDeserializer::new(map))
    }
}

struct WButtonActionWrapper(WButtonAction);

impl<'de> Deserialize<'de> for WButtonActionWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(WButtonActionWrapper(
            deserializer.deserialize_any(WButtonActionVisitor)?,
        ))
    }
}

#[derive(Debug, Clone)]
pub enum WButtonActionList {
    Single(WButtonAction),
    Multiple(Vec<WButtonAction>),
}

struct WButtonActionListVisitor;

impl<'de> Visitor<'de> for WButtonActionListVisitor {
    type Value = WButtonActionList;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("one or multiple actions")
    }

    fn visit_str<E>(self, value: &str) -> Result<WButtonActionList, E>
    where
        E: serde::de::Error,
    {
        Ok(WButtonActionList::Single(
            WButtonActionWrapper::deserialize(value.into_deserializer())?.0,
        ))
    }

    fn visit_seq<M>(self, mut seq: M) -> Result<Self::Value, M::Error>
    where
        M: SeqAccess<'de>,
    {
        let mut actions = Vec::new();

        while let Some(WButtonActionWrapper(value)) = seq.next_element::<WButtonActionWrapper>()? {
            actions.push(value);
        }

        Ok(WButtonActionList::Multiple(actions))
    }

    fn visit_map<M>(self, map: M) -> Result<WButtonActionList, M::Error>
    where
        M: serde::de::MapAccess<'de>,
    {
        Ok(WButtonActionList::Single(WButtonAction::deserialize(
            serde::de::value::MapAccessDeserializer::new(map),
        )?))
    }
}

impl<'de> Deserialize<'de> for WButtonActionList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(WButtonActionListVisitor)
    }
}

impl WButtonActionList {
    pub fn enumerate(&self) -> impl Iterator<Item = &WButtonAction> {
        match self {
            WButtonActionList::Single(action) => std::slice::from_ref(action).iter(),
            WButtonActionList::Multiple(actions) => actions.iter(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct WButton {
    pub label: String,
    pub action: WButtonActionList,
    pub text: String,
    pub keybind: String,
    #[serde(default = "default_justify")]
    pub justify: String,
    pub width: Option<f32>,
    pub height: Option<f32>,
    #[serde(default = "default_circular")]
    pub circular: bool,
    #[serde(default)]
    pub icon: Option<String>,
}

fn default_justify() -> String {
    String::from("center")
}

fn default_circular() -> bool {
    false
}
