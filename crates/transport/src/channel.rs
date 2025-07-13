#![doc = include_str!("./channel.md")]

use std::fmt::Display;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare]
#[derive(Clone, Debug, Deserialize, Serialize)]
/// Represents a communication channel with a unique ID, type, name, and
/// optional parent ID.
///
/// # Fields
/// - `id`: Unique identifier for the channel. If the platform cannot provide a
///   user ID, the event ID is used.
/// - `ty`: Type of the channel (`group`, `direct`, or `private`).
/// - `name`: Display name of the channel. If the platform cannot provide a
///   nickname, the `id` is used.
/// - `parent_id`: Optional parent channel ID, used for nested channels (e.g.,
///   subgroups).
pub struct Channel {
    pub id:        String,
    #[serde(rename = "type")]
    pub ty:        ChannelType,
    pub name:      String,
    pub parent_id: Option<String>,
    pub self_id:   Option<String>,
}

impl Channel {
    /// Creates a new private channel.
    ///
    /// # Arguments
    /// - `id`: Unique identifier for the channel.
    /// - `name`: Display name of the channel.
    ///
    /// # Example
    /// ```
    /// let channel = Channel::Private("user123".to_string(), "Alice".to_string());
    /// ```
    #[allow(non_snake_case)]
    #[must_use]
    pub const fn Private(id: String, name: String) -> Self {
        Self {
            id,
            ty: ChannelType::Private,
            name,
            parent_id: None,
            self_id: None,
        }
    }

    /// Creates a new group channel.
    ///
    /// # Arguments
    /// - `id`: Unique identifier for the group.
    /// - `name`: Display name of the group.
    ///
    /// # Example
    /// ```
    /// let channel =
    ///     Channel::Group("group123".to_string(), "Developers".to_string());
    /// ```
    #[allow(non_snake_case)]
    #[must_use]
    pub const fn Group(id: String, name: String) -> Self {
        Self {
            id,
            ty: ChannelType::Group,
            name,
            parent_id: None,
            self_id: None,
        }
    }

    /// Creates a new direct message channel.
    ///
    /// # Arguments
    /// - `id`: Unique identifier for the direct message.
    /// - `name`: Display name of the recipient.
    ///
    /// # Example
    /// ```
    /// let channel = Channel::Direct("user456".to_string(), "Bob".to_string());
    /// ```
    #[allow(non_snake_case)]
    #[must_use]
    pub const fn Direct(id: String, name: String) -> Self {
        Self {
            id,
            ty: ChannelType::Direct,
            name,
            parent_id: None,
            self_id: None,
        }
    }

    /// Creates a new direct message channel associated with a group.
    ///
    /// # Arguments
    /// - `group_id`: Unique identifier for the parent group.
    /// - `id`: Unique identifier for the direct message.
    /// - `name`: Display name of the recipient.
    ///
    /// # Example
    /// ```
    /// let channel = Channel::DirectFromGroup(
    ///     "group123".to_string(),
    ///     "user789".to_string(),
    ///     "Charlie".to_string(),
    /// );
    /// ```
    #[allow(non_snake_case)]
    #[must_use]
    pub const fn DirectFromGroup(group_id: String, id: String, name: String) -> Self {
        Self {
            id,
            ty: ChannelType::Direct,
            name,
            parent_id: Some(group_id),
            self_id: None,
        }
    }

    #[must_use]
    pub fn set_self_id<T: Display>(mut self, id: T) -> Self {
        self.self_id = Some(id.to_string());
        self
    }
}

/// Represents the type of a communication channel.
#[typeshare]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    /// A group channel, typically used for multi-user conversations.
    Group,
    /// A direct message channel, used for one-on-one conversations.
    Direct,
    /// A private channel, used for restricted or hidden conversations.
    Private,
}
