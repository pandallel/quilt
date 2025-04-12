use crate::actors::Ping;
use crate::cutting::{CuttingActor, InMemoryCutsRepository};
use crate::events::EventBus;
use crate::events::QuiltEvent;
use crate::materials::types::Material;
use crate::materials::{MaterialRegistry, MaterialRepository};
use actix::prelude::*;
use std::sync::Arc;
use std::time::Duration;

#[actix::test]
async fn test_cutting_actor_integration() {
    // Initialize event bus and registry
    let event_bus = Arc::new(EventBus::new());
    let repository = MaterialRepository::new();
    let registry = MaterialRegistry::new(repository, event_bus.clone());
    let cuts_repository = Arc::new(InMemoryCutsRepository::new());

    // Start the cutting actor
    let cutting_actor =
        CuttingActor::new("IntegrationCuttingActor", registry.clone(), cuts_repository).start();

    // Test ping works
    let ping_result = cutting_actor.send(Ping).await;
    assert!(ping_result.is_ok());
    assert!(ping_result.unwrap());

    // Register a new material
    let material = Material::new("test/integration_material.md".to_string());
    let material_id = material.id.clone();
    registry.register_material(material).await.unwrap();

    // Retrieve the material to ensure it's registered
    let material = registry.get_material(&material_id).await.unwrap();

    // Publish a MaterialDiscovered event
    let event = QuiltEvent::material_discovered(&material);
    event_bus.publish(event).unwrap();

    // Wait for event processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Actor should still be alive
    let ping_result = cutting_actor.send(Ping).await;
    assert!(ping_result.is_ok());
    assert!(ping_result.unwrap());
}

#[actix::test]
async fn test_cutting_actor_handles_missing_material() {
    // Initialize event bus
    let event_bus = Arc::new(EventBus::new());

    // Create repository and registry
    let repository = MaterialRepository::new();
    let registry = MaterialRegistry::new(repository, event_bus.clone());
    let cuts_repository = Arc::new(InMemoryCutsRepository::new());

    // Start the cutting actor
    let cutting_actor =
        CuttingActor::new("ErrorTestCuttingActor", registry.clone(), cuts_repository).start();

    // Give the actor time to set up
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Create a material ID that doesn't exist in the repository
    let non_existent_material_id = "non-existent-material-id".to_string();

    // Create a fake material for event creation
    let fake_material = Material::new("test/non_existent_file.md".to_string());
    // Override the ID to use our non-existent ID
    let mut fake_material = fake_material;
    fake_material.id = non_existent_material_id.clone();

    // Create and publish the event using the constructor
    let event = QuiltEvent::material_discovered(&fake_material);

    // Publish the event
    event_bus.publish(event).unwrap();

    // Wait for event processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // The actor will try to process the material, but it won't find it
    // Since it returns an error immediately and doesn't try to update status,
    // we simply check if the actor is still alive

    // Verify the actor is still alive
    let ping_result = cutting_actor.send(Ping).await;
    assert!(ping_result.is_ok());
    assert!(ping_result.unwrap());
}
