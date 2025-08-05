use std::fmt::Display;

use de::Error as _;
use internal::{Contact, InternalOneBotSegment, InternalOneBotTypedSegment, Location, Poke};
use ser::Error as _;
use serde::{Deserialize, Serialize, de, ser};
use sithra_kit::{
    transport::{self, ValueError},
    types::message::{NIL, Segment},
};

use crate::message::internal::InternalOneBotUnknownSegment;

pub mod internal {
    use serde::{Deserialize, Serialize, de::Error as _};
    use sithra_kit::transport::Value;

    use crate::util::or_in_base64;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "snake_case", tag = "type", content = "data")]
    pub enum InternalOneBotTypedSegment {
        Text {
            text: String,
        },
        Face {
            id: String,
        },
        Image {
            file: String,
        },
        Record {
            file: String,
        },
        Video {
            file: String,
        },
        At {
            #[serde(default)]
            id: String,
            #[serde(default)]
            qq: String,
        },
        Rps,
        Dice,
        Shake,
        Poke(Poke),
        Contact(Contact),
        Location(Location),
        Reply {
            id: String,
        },
    }

    impl InternalOneBotTypedSegment {
        pub async fn or_in_base64(self) -> Self {
            match self {
                Self::Image { file } => Self::Image {
                    file: or_in_base64(&file).await.unwrap_or(file),
                },
                Self::Record { file } => Self::Record {
                    file: or_in_base64(&file).await.unwrap_or(file),
                },
                Self::Video { file } => Self::Video {
                    file: or_in_base64(&file).await.unwrap_or(file),
                },
                _ => self,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum InternalOneBotSegment {
        Typed(InternalOneBotTypedSegment),
        Unknown(InternalOneBotUnknownSegment),
    }

    impl InternalOneBotSegment {
        pub async fn or_in_base64(self) -> Self {
            if let Self::Typed(typed) = self {
                Self::Typed(typed.or_in_base64().await)
            } else {
                self
            }
        }
    }

    impl<'de> Deserialize<'de> for InternalOneBotSegment {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = serde_json::Value::deserialize(deserializer)?;
            let typed = InternalOneBotTypedSegment::deserialize(value.clone());
            match typed {
                Ok(typed) => Ok(Self::Typed(typed)),
                Err(_err) => {
                    let unknown = InternalOneBotUnknownSegment::deserialize(value)
                        .map_err(D::Error::custom)?;
                    Ok(Self::Unknown(unknown))
                }
            }
        }
    }

    impl Serialize for InternalOneBotSegment {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                Self::Typed(typed) => typed.serialize(serializer),
                Self::Unknown(unknown) => unknown.serialize(serializer),
            }
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct InternalOneBotUnknownSegment {
        #[serde(default, rename = "type")]
        pub ty:   String,
        pub data: Value,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Poke {
        #[serde(rename = "type")]
        pub ty: String,
        pub id: String,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Contact {
        #[serde(rename = "type")]
        pub ty: String,
        pub id: String,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Location {
        pub lat: f64,
        pub lon: f64,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct InternalOneBotUnknown {
        #[serde(rename = "type")]
        pub ty:   String,
        pub data: Value,
    }
}

#[derive(Debug, Clone)]
pub struct OneBotSegment(pub InternalOneBotSegment);

impl OneBotSegment {
    pub fn text<T: Display>(content: T) -> Self {
        Self(InternalOneBotSegment::Typed(
            InternalOneBotTypedSegment::Text {
                text: content.to_string(),
            },
        ))
    }

    pub fn image<T: Display>(url: T) -> Self {
        Self(InternalOneBotSegment::Typed(
            InternalOneBotTypedSegment::Image {
                file: url.to_string(),
            },
        ))
    }

    pub fn img<T: Display>(url: T) -> Self {
        Self::image(url)
    }

    pub fn at<T: Display>(target: T) -> Self {
        Self(InternalOneBotSegment::Typed(
            InternalOneBotTypedSegment::At {
                id: target.to_string(),
                qq: target.to_string(),
            },
        ))
    }

    pub fn reply<T: Display>(target: T) -> Self {
        Self(InternalOneBotSegment::Typed(
            InternalOneBotTypedSegment::Reply {
                id: target.to_string(),
            },
        ))
    }

    #[must_use]
    pub const fn location((lat, lon): (f64, f64)) -> Self {
        Self(InternalOneBotSegment::Typed(
            InternalOneBotTypedSegment::Location(Location { lat, lon }),
        ))
    }

    pub fn face<T: Display>(id: T) -> Self {
        Self(InternalOneBotSegment::Typed(
            InternalOneBotTypedSegment::Face { id: id.to_string() },
        ))
    }

    pub fn video<T: Display>(url: T) -> Self {
        Self(InternalOneBotSegment::Typed(
            InternalOneBotTypedSegment::Video {
                file: url.to_string(),
            },
        ))
    }

    pub fn record<T: Display>(url: T) -> Self {
        Self(InternalOneBotSegment::Typed(
            InternalOneBotTypedSegment::Record {
                file: url.to_string(),
            },
        ))
    }

    #[must_use]
    pub const fn rps() -> Self {
        Self(InternalOneBotSegment::Typed(
            InternalOneBotTypedSegment::Rps,
        ))
    }

    #[must_use]
    pub const fn dice() -> Self {
        Self(InternalOneBotSegment::Typed(
            InternalOneBotTypedSegment::Dice,
        ))
    }

    #[must_use]
    pub const fn shake() -> Self {
        Self(InternalOneBotSegment::Typed(
            InternalOneBotTypedSegment::Shake,
        ))
    }

    pub fn poke<T1: Display, T2: Display>((ty, id): (T1, T2)) -> Self {
        Self(InternalOneBotSegment::Typed(
            InternalOneBotTypedSegment::Poke(Poke {
                ty: ty.to_string(),
                id: id.to_string(),
            }),
        ))
    }

    pub fn contact<T1: Display, T2: Display>((ty, id): (T1, T2)) -> Self {
        Self(InternalOneBotSegment::Typed(
            InternalOneBotTypedSegment::Contact(Contact {
                ty: ty.to_string(),
                id: id.to_string(),
            }),
        ))
    }
}

impl TryFrom<Segment> for OneBotSegment {
    type Error = ValueError;

    fn try_from(value: Segment) -> Result<Self, Self::Error> {
        let Segment { ty, data } = value;
        match ty.as_str() {
            "text" => Ok(Self(InternalOneBotSegment::Typed(
                InternalOneBotTypedSegment::Text {
                    text: transport::from_value(data)?,
                },
            ))),
            "face" => Ok(Self(InternalOneBotSegment::Typed(
                InternalOneBotTypedSegment::Face {
                    id: transport::from_value(data)?,
                },
            ))),
            "image" => Ok(Self(InternalOneBotSegment::Typed(
                InternalOneBotTypedSegment::Image {
                    file: transport::from_value(data)?,
                },
            ))),
            "record" => Ok(Self(InternalOneBotSegment::Typed(
                InternalOneBotTypedSegment::Record {
                    file: transport::from_value(data)?,
                },
            ))),
            "video" => Ok(Self(InternalOneBotSegment::Typed(
                InternalOneBotTypedSegment::Video {
                    file: transport::from_value(data)?,
                },
            ))),
            "at" => {
                let id: String = transport::from_value(data)?;
                Ok(Self(InternalOneBotSegment::Typed(
                    InternalOneBotTypedSegment::At {
                        id: id.clone(),
                        qq: id,
                    },
                )))
            }
            "rps" => Ok(Self(InternalOneBotSegment::Typed(
                InternalOneBotTypedSegment::Rps,
            ))),
            "dice" => Ok(Self(InternalOneBotSegment::Typed(
                InternalOneBotTypedSegment::Dice,
            ))),
            "shake" => Ok(Self(InternalOneBotSegment::Typed(
                InternalOneBotTypedSegment::Shake,
            ))),
            "poke" => Ok(Self(InternalOneBotSegment::Typed(
                InternalOneBotTypedSegment::Poke(transport::from_value(data)?),
            ))),
            "contact" => Ok(Self(InternalOneBotSegment::Typed(
                InternalOneBotTypedSegment::Contact(transport::from_value(data)?),
            ))),
            "location" => Ok(Self(InternalOneBotSegment::Typed(
                InternalOneBotTypedSegment::Location(transport::from_value(data)?),
            ))),
            "reply" => Ok(Self(InternalOneBotSegment::Typed(
                InternalOneBotTypedSegment::Reply {
                    id: transport::from_value(data)?,
                },
            ))),
            _ => Ok(Self(InternalOneBotSegment::Unknown(
                InternalOneBotUnknownSegment { ty, data },
            ))),
        }
    }
}

impl TryFrom<OneBotSegment> for Segment {
    type Error = ValueError;

    fn try_from(value: OneBotSegment) -> Result<Self, Self::Error> {
        match value {
            OneBotSegment(InternalOneBotSegment::Typed(typed)) => match typed {
                InternalOneBotTypedSegment::Text { text } => Ok(Self::text(&text)),
                InternalOneBotTypedSegment::Face { id } => Self::custom("face", id),
                InternalOneBotTypedSegment::Image { file } => Ok(Self::image(file)),
                InternalOneBotTypedSegment::Record { file } => Self::custom("file", file),
                InternalOneBotTypedSegment::Video { file } => Self::custom("video", file),
                InternalOneBotTypedSegment::At { id, qq } => {
                    if qq.is_empty() {
                        Ok(Self::at(&id))
                    } else {
                        Ok(Self::at(&qq))
                    }
                }
                InternalOneBotTypedSegment::Rps => Self::custom("rps", NIL),
                InternalOneBotTypedSegment::Dice => Self::custom("dice", NIL),
                InternalOneBotTypedSegment::Shake => Self::custom("shake", NIL),
                InternalOneBotTypedSegment::Reply { id } => Self::custom("reply", id),
                InternalOneBotTypedSegment::Poke(poke) => Self::custom("poke", poke),
                InternalOneBotTypedSegment::Contact(contact) => Self::custom("contact", contact),
                InternalOneBotTypedSegment::Location(location) => {
                    Self::custom("location", location)
                }
            },
            OneBotSegment(InternalOneBotSegment::Unknown(unknown)) => Ok(Self {
                ty:   unknown.ty,
                data: unknown.data,
            }),
        }
    }
}

impl<'de> Deserialize<'de> for OneBotSegment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
        D::Error: de::Error,
    {
        let raw = Segment::deserialize(deserializer)?;
        raw.try_into().map_err(|e| D::Error::custom(format!("Invalid segment: {e}")))
    }
}

impl Serialize for OneBotSegment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        S::Error: ser::Error,
    {
        let segment: Segment = self
            .clone()
            .try_into()
            .map_err(|e| S::Error::custom(format!("Failed to serialize segment: {e}")))?;
        segment.serialize(serializer)
    }
}
