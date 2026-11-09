use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use crate::{Message, ProtocolError};

/// The last payload broadcast for each message type.
///
/// The server keeps one of these and answers `init` and `requestInit` from it.
/// Payloads are stored as JSON values so each frame is serialised once and can
/// be cloned cheaply.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SectionCache {
    sections: BTreeMap<String, Value>,
}

impl SectionCache {
    /// Creates an empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Serialises `payload` and stores it under `message_type`.
    ///
    /// Returns an error only if the DTO cannot be represented as JSON.
    pub fn set<T: Serialize>(
        &mut self,
        message_type: &str,
        payload: &T,
    ) -> Result<(), ProtocolError> {
        let value = serde_json::to_value(payload).map_err(|source| ProtocolError::Serialize {
            message_type: message_type.to_owned(),
            source,
        })?;
        self.sections.insert(message_type.to_owned(), value);
        Ok(())
    }

    /// Stores an already-serialised payload.
    pub fn set_value(&mut self, message_type: &str, payload: Value) {
        self.sections.insert(message_type.to_owned(), payload);
    }

    /// Returns the cached payload for `message_type`.
    #[must_use]
    pub fn get(&self, message_type: &str) -> Option<&Value> {
        self.sections.get(message_type)
    }

    /// Iterates every cached section in message-type order.
    ///
    /// The server uses this to resend all known sections after a port or
    /// asset-mode change.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.sections
            .iter()
            .map(|(message_type, payload)| (message_type.as_str(), payload))
    }

    /// Removes a cached payload.
    pub fn remove(&mut self, message_type: &str) -> Option<Value> {
        self.sections.remove(message_type)
    }

    /// Returns true when `message_type` has been broadcast at least once.
    #[must_use]
    pub fn contains(&self, message_type: &str) -> bool {
        self.sections.contains_key(message_type)
    }

    /// Number of cached sections.
    #[must_use]
    pub fn len(&self) -> usize {
        self.sections.len()
    }

    /// Returns true when nothing has been broadcast yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }

    /// Removes every cached section, for example when a match is unloaded.
    pub fn clear(&mut self) {
        self.sections.clear();
    }

    /// Builds the keyed `init` message from the current cache.
    #[must_use]
    pub fn init(&self) -> Message<BTreeMap<String, Value>> {
        Message::new("init", self.sections.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter_returns_sections_in_message_type_order() {
        let mut cache = SectionCache::new();
        cache.set_value("zeta", serde_json::json!({ "value": 2 }));
        cache.set_value("alpha", serde_json::json!({ "value": 1 }));

        let kinds: Vec<&str> = cache.iter().map(|(kind, _)| kind).collect();
        assert_eq!(kinds, ["alpha", "zeta"]);
    }
}
