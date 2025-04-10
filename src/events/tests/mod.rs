use std::sync::Arc;

use crate::events::{EventBus, QuiltEvent};
use crate::materials::{Material, MaterialRegistry, MaterialRepository};

/// Setup function to create a registry with event bus for testing
async fn setup_registry() -> (MaterialRegistry, Arc<EventBus>) {
    let repository = MaterialRepository::new();
    let event_bus = Arc::new(EventBus::new());
    let registry = MaterialRegistry::new(repository, event_bus.clone());
    (registry, event_bus)
}

#[tokio::test]
async fn test_registry_event_integration() {
    // Setup registry with event bus
    let (registry, event_bus) = setup_registry().await;

    // Subscribe to events
    let mut rx = event_bus.subscribe();

    // Create and register material
    let material = Material::new("test/integration.md".to_string());
    let material_id = material.id.clone();

    registry.register_material(material).await.unwrap();

    // Check for MaterialDiscovered event
    let event = rx.recv().await.unwrap();
    match event {
        QuiltEvent::MaterialDiscovered(evt) => {
            assert_eq!(evt.material_id.as_str(), material_id);
            assert_eq!(evt.file_path, "test/integration.md");
        }
        _ => panic!("Expected MaterialDiscovered event"),
    }
}

#[tokio::test]
async fn test_multiple_materials_with_events() {
    // Setup registry with event bus
    let (registry, event_bus) = setup_registry().await;

    // Subscribe to events
    let mut rx = event_bus.subscribe();

    // Create and register multiple materials
    let material1 = Material::new("test/file1.md".to_string());
    let material2 = Material::new("test/file2.md".to_string());
    let material3 = Material::new("test/file3.md".to_string());

    let id1 = material1.id.clone();
    let id2 = material2.id.clone();
    let id3 = material3.id.clone();

    // Register all materials
    registry.register_material(material1).await.unwrap();
    registry.register_material(material2).await.unwrap();
    registry.register_material(material3).await.unwrap();

    // We should receive three MaterialDiscovered events
    for _ in 0..3 {
        match rx.recv().await.unwrap() {
            QuiltEvent::MaterialDiscovered(evt) => {
                assert!(
                    evt.material_id.as_str() == id1
                        || evt.material_id.as_str() == id2
                        || evt.material_id.as_str() == id3
                );
            }
            _ => panic!("Expected MaterialDiscovered event"),
        }
    }
}

#[tokio::test]
async fn test_event_processing_with_multiple_subscribers() {
    // Setup registry with event bus
    let (registry, event_bus) = setup_registry().await;

    // Create multiple subscribers
    let mut rx1 = event_bus.subscribe();
    let mut rx2 = event_bus.subscribe();
    let mut rx3 = event_bus.subscribe();

    // Create a material
    let material = Material::new("test/multi-subscriber.md".to_string());
    let material_id = material.id.clone();

    // Register the material
    registry.register_material(material).await.unwrap();

    // Each subscriber should receive the event
    for rx in [&mut rx1, &mut rx2, &mut rx3].iter_mut() {
        match rx.recv().await.unwrap() {
            QuiltEvent::MaterialDiscovered(evt) => {
                assert_eq!(evt.material_id.as_str(), material_id);
                assert_eq!(evt.file_path, "test/multi-subscriber.md");
            }
            _ => panic!("Expected MaterialDiscovered event"),
        }
    }
}
