use crate::materials::types::Material;
use std::fmt;
use time::OffsetDateTime;

/// Material event containing material information
#[derive(Debug, Clone)]
pub struct MaterialDiscoveredEvent {
    /// ID of the material
    pub material_id: String,
    /// Timestamp when the event occurred
    pub timestamp: OffsetDateTime,
    /// File path of the material
    pub file_path: String,
}

/// Represents an event in the Quilt system
#[derive(Debug, Clone)]
pub enum QuiltEvent {
    /// Material has been discovered and registered
    MaterialDiscovered(MaterialDiscoveredEvent),
    /// System event for shutdown or health check
    System(SystemEvent),
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
            material_id: material.id.clone(),
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
}

impl fmt::Display for QuiltEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MaterialDiscovered(evt) => write!(
                f,
                "MaterialDiscovered {{ material_id: {}, file_path: {} }}",
                evt.material_id, evt.file_path
            ),
            Self::System(SystemEvent::Shutdown) => write!(f, "System.Shutdown"),
            Self::System(SystemEvent::HealthCheck) => write!(f, "System.HealthCheck"),
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
            assert_eq!(evt.material_id, material_id);
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
}
