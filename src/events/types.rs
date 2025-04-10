use crate::materials::types::Material;
use std::fmt;
use time::OffsetDateTime;

/// Strongly typed material identifier for improved type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialId(String);

impl MaterialId {
    /// Create a new MaterialId
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Get the string representation of the ID
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for MaterialId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for MaterialId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Processing stages in the material pipeline
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessingStage {
    /// Material discovery stage
    Discovery,
    /// Material cutting stage
    Cutting,
    /// Material swatching stage
    Swatching,
    /// Custom processing stage
    Custom(String),
}

impl fmt::Display for ProcessingStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Discovery => write!(f, "discovery"),
            Self::Cutting => write!(f, "cutting"),
            Self::Swatching => write!(f, "swatching"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Material event containing material information
#[derive(Debug, Clone)]
pub struct MaterialDiscoveredEvent {
    /// ID of the material
    pub material_id: MaterialId,
    /// Timestamp when the event occurred
    pub timestamp: OffsetDateTime,
    /// File path of the material
    pub file_path: String,
}

/// Error event during material processing
#[derive(Debug, Clone)]
pub struct MaterialProcessingErrorEvent {
    /// ID of the material that had a processing error
    pub material_id: MaterialId,
    /// Timestamp when the error occurred
    pub timestamp: OffsetDateTime,
    /// Processing stage where the error occurred
    pub stage: ProcessingStage,
    /// Error message
    pub message: String,
}

/// Represents an event in the Quilt system
#[derive(Debug, Clone)]
pub enum QuiltEvent {
    /// Material has been discovered and registered
    MaterialDiscovered(MaterialDiscoveredEvent),
    /// System event for shutdown or health check
    System(SystemEvent),
    /// Processing error occurred
    ProcessingError(MaterialProcessingErrorEvent),
}

/// System-wide events
#[derive(Debug, Clone)]
pub enum SystemEvent {
    /// Shutdown request
    Shutdown,
    /// Health check event
    HealthCheck,
}

/// Event type conversion and utility methods
impl QuiltEvent {
    /// Create a MaterialDiscovered event from a Material
    pub fn material_discovered(material: &Material) -> Self {
        Self::MaterialDiscovered(MaterialDiscoveredEvent {
            material_id: MaterialId::new(material.id.clone()),
            timestamp: OffsetDateTime::now_utc(),
            file_path: material.file_path.clone(),
        })
    }

    /// Create a Shutdown event
    pub fn shutdown() -> Self {
        Self::System(SystemEvent::Shutdown)
    }

    /// Create a HealthCheck event
    pub fn health_check() -> Self {
        Self::System(SystemEvent::HealthCheck)
    }

    /// Create a processing error event
    ///
    /// # Arguments
    ///
    /// * `material_id` - ID of the material that had an error
    /// * `stage` - Processing stage where the error occurred
    /// * `message` - Error message
    pub fn create_processing_error_event(
        material_id: &str,
        stage: ProcessingStage,
        message: &str,
    ) -> Self {
        Self::ProcessingError(MaterialProcessingErrorEvent {
            material_id: MaterialId::new(material_id.to_string()),
            timestamp: OffsetDateTime::now_utc(),
            stage,
            message: message.to_string(),
        })
    }

    /// Create a processing error event with a string stage (backward compatibility)
    ///
    /// # Deprecated
    /// Use `create_processing_error_event` with a `ProcessingStage` instead
    pub fn processing_error(material_id: &str, stage: &str, message: &str) -> Self {
        Self::create_processing_error_event(
            material_id,
            match stage {
                "discovery" => ProcessingStage::Discovery,
                "cutting" => ProcessingStage::Cutting,
                "swatching" => ProcessingStage::Swatching,
                _ => ProcessingStage::Custom(stage.to_string()),
            },
            message,
        )
    }
}

impl fmt::Display for QuiltEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MaterialDiscovered(evt) => write!(
                f,
                "MaterialDiscovered {{ material_id: {}, file_path: {} }}",
                evt.material_id.as_str(),
                evt.file_path
            ),
            Self::System(SystemEvent::Shutdown) => write!(f, "System.Shutdown"),
            Self::System(SystemEvent::HealthCheck) => write!(f, "System.HealthCheck"),
            Self::ProcessingError(evt) => write!(
                f,
                "ProcessingError {{ material_id: {}, stage: {}, message: {} }}",
                evt.material_id.as_str(),
                evt.stage,
                evt.message
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::types::Material;

    #[test]
    fn test_event_creation() {
        let material = Material::new("test/file.md".to_string());
        let material_id = material.id.clone();
        let file_path = material.file_path.clone();

        let discovered = QuiltEvent::material_discovered(&material);

        if let QuiltEvent::MaterialDiscovered(evt) = discovered {
            assert_eq!(evt.material_id.as_str(), material_id);
            assert_eq!(evt.file_path, file_path);
        } else {
            panic!("Expected MaterialDiscovered event");
        }
    }

    #[test]
    fn test_system_events() {
        let shutdown = QuiltEvent::shutdown();
        let health_check = QuiltEvent::health_check();

        if let QuiltEvent::System(SystemEvent::Shutdown) = shutdown {
            // Successfully matched
        } else {
            panic!("Expected Shutdown event");
        }

        if let QuiltEvent::System(SystemEvent::HealthCheck) = health_check {
            // Successfully matched
        } else {
            panic!("Expected HealthCheck event");
        }
    }

    #[test]
    fn test_event_display() {
        let material = Material::new("test/file.md".to_string());
        let event = QuiltEvent::material_discovered(&material);

        let display = format!("{}", event);
        assert!(display.contains("MaterialDiscovered"));
        assert!(display.contains(&material.id));
        assert!(display.contains("test/file.md"));
    }

    #[test]
    fn test_processing_error_event() {
        let error_event = QuiltEvent::create_processing_error_event(
            "test-material",
            ProcessingStage::Cutting,
            "Test error message",
        );

        if let QuiltEvent::ProcessingError(evt) = error_event {
            assert_eq!(evt.material_id.as_str(), "test-material");
            assert_eq!(evt.stage, ProcessingStage::Cutting);
            assert_eq!(evt.message, "Test error message");
        } else {
            panic!("Expected ProcessingError event");
        }
    }

    #[test]
    fn test_material_id_from_str() {
        let id = MaterialId::from("test-id");
        assert_eq!(id.as_str(), "test-id");
    }
}
