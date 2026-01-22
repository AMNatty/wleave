use miette::miette;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::error::Error;
use std::fmt::Display;
use std::num::NonZeroU32;
use std::str::FromStr;

#[derive(Clone, Copy, Debug)]
pub enum AspectRatio {
    Float(f32),
    Ratio(u32, u32),
}

impl Default for AspectRatio {
    fn default() -> Self {
        AspectRatio::Float(1.0)
    }
}

impl<'de> Deserialize<'de> for AspectRatio {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Value::deserialize(deserializer)?;
        if let Some(f) = v.as_f64() {
            Ok(AspectRatio::Float(f as f32))
        } else if let Some(s) = v.as_str() {
            FromStr::from_str(s).map_err(serde::de::Error::custom)
        } else {
            Err(serde::de::Error::custom(
                "Aspect ratio neither a positive float nor a ratio (1/1, 2/3, ...)",
            ))
        }
    }
}

impl FromStr for AspectRatio {
    type Err = Box<dyn Error + Send + Sync + 'static>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(float) = s.parse::<f32>() {
            if float < 0.0 {
                return Err("Aspect ratio cannot be negative".into());
            }
            return Ok(AspectRatio::Float(float));
        }

        if let Some((n, d)) = s.split_once('/')
            && let (Ok(n), Ok(d)) = (n.parse::<NonZeroU32>(), d.parse::<NonZeroU32>())
        {
            return Ok(AspectRatio::Ratio(n.into(), d.into()));
        }

        Err("Aspect ratio neither a float nor a ratio".into())
    }
}

impl Display for AspectRatio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Float(r) => write!(f, "{r}"),
            Self::Ratio(n, d) => write!(f, "{n}/{d}"),
        }
    }
}

impl AspectRatio {
    pub fn as_float(self) -> f32 {
        match self {
            Self::Float(f) => f,
            Self::Ratio(n, d) => (n as f32) / (d as f32),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LengthValue {
    Px(f32),
    Percentage(f32),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum LengthSerialized {
    Number(f32),
    String(String),
}

#[derive(Debug, Copy, Clone)]
pub enum LengthDimension {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone)]
pub struct LengthArgs {
    pub viewport: (f32, f32),
    pub dimension: LengthDimension,
}

impl<'de> Deserialize<'de> for LengthValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match LengthSerialized::deserialize(deserializer)? {
            LengthSerialized::Number(num) => Ok(LengthValue::Px(num)),
            LengthSerialized::String(val) => val.parse().map_err(serde::de::Error::custom),
        }
    }
}

impl LengthValue {
    pub fn for_args(&self, args: &LengthArgs) -> f32 {
        match self {
            LengthValue::Percentage(p) => match args.dimension {
                LengthDimension::Horizontal => *p * args.viewport.0,
                LengthDimension::Vertical => *p * args.viewport.1,
            },
            LengthValue::Px(val) => *val,
        }
    }

    fn parse(val: &'_ str) -> Result<Self, cssparser::BasicParseError<'_>> {
        let mut input = cssparser::ParserInput::new(val);
        let mut parser = cssparser::Parser::new(&mut input);

        let token = parser.next()?;

        let value = match token {
            cssparser::Token::Number { value, .. } => LengthValue::Px(*value),
            cssparser::Token::Percentage { unit_value, .. } => LengthValue::Percentage(*unit_value),
            cssparser::Token::Dimension { value, unit, .. } => match unit.as_ref() {
                "px" => LengthValue::Px(*value),
                _ => {
                    let tok = token.clone();
                    return Err(parser
                        .current_source_location()
                        .new_basic_unexpected_token_error(tok));
                }
            },
            _ => {
                let tok = token.clone();
                return Err(parser
                    .current_source_location()
                    .new_basic_unexpected_token_error(tok));
            }
        };

        parser.expect_exhausted()?;

        Ok(value)
    }
}

impl FromStr for LengthValue {
    type Err = miette::Report;

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        LengthValue::parse(val).map_err(|e| miette!("Length parse error: {:?}", e))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct Margin(pub LengthValue);

impl FromStr for Margin {
    type Err = miette::Report;

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        LengthValue::from_str(val)
            .map(Margin)
            .map_err(|e| miette!("Margin parse error: {:?}", e))
    }
}
