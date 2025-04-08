use super::types::Material;

/// Messages that are passed between worker actors in the processing pipeline
///
/// These message types form the core communication protocol for the actor model
/// architecture in Quilt. They represent the different states and events in the
/// material processing lifecycle.
#[derive(Debug, Clone, PartialEq)]
pub enum MaterialMessage {
    /// Indicates a new material has been discovered and should be processed
    ///
    /// Contains the full Material struct with metadata. This is sent from the
    /// Discovery Worker to the Cutting Worker.
    Discovered(Material),

    /// Indicates a material has been cut into swatches (contains material ID)
    ///
    /// Contains only the material ID to minimize message size. This is sent
    /// from the Cutting Worker to the Labeling Worker.
    Cut(String),

    /// Indicates a material has been swatched/embedded (contains material ID)
    ///
    /// Contains only the material ID to minimize message size. This message
    /// signals the completion of the processing pipeline for a material.
    Swatched(String),

    /// Indicates an error occurred during processing (material ID, error message)
    ///
    /// Contains the material ID and an error message describing what went wrong.
    /// This can be sent by any worker when processing fails.
    Error(String, String),

    /// Signal to shutdown the worker
    ///
    /// Used to initiate a graceful shutdown of the processing pipeline.
    /// This should be propagated through all stages.
    Shutdown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::types::Material;

    #[test]
    fn test_message_creation() {
        // Create a test material
        let material = Material::new("test/doc.md".to_string());

        // Test Discovered message
        let msg = MaterialMessage::Discovered(material.clone());
        assert!(matches!(msg, MaterialMessage::Discovered(_)));

        // Test Cut message
        let msg = MaterialMessage::Cut(material.id.clone());
        assert!(matches!(msg, MaterialMessage::Cut(_)));

        // Test Swatched message
        let msg = MaterialMessage::Swatched(material.id.clone());
        assert!(matches!(msg, MaterialMessage::Swatched(_)));

        // Test Error message
        let error_msg = "Processing failed".to_string();
        let msg = MaterialMessage::Error(material.id.clone(), error_msg.clone());
        if let MaterialMessage::Error(id, err) = msg {
            assert_eq!(id, material.id);
            assert_eq!(err, error_msg);
        } else {
            panic!("Expected Error message");
        }

        // Test Shutdown message
        let msg = MaterialMessage::Shutdown;
        assert!(matches!(msg, MaterialMessage::Shutdown));
    }
}
