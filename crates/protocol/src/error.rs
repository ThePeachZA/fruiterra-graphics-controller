/// Errors raised while building wire messages.
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    /// A DTO could not be serialised to a JSON value.
    #[error("failed to serialise the {message_type} payload: {source}")]
    Serialize {
        /// Message type whose payload failed.
        message_type: String,
        /// Underlying serde error.
        #[source]
        source: serde_json::Error,
    },
}
