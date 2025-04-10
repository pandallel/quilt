use crate::actors::Ping;
use crate::cutting::CuttingActor;
use crate::events::EventBus;
use crate::events::QuiltEvent;
use crate::materials::types::Material;
use crate::materials::MaterialRegistry;
use crate::materials::MaterialRepository;
use actix::prelude::*;
use std::sync::Arc;
use std::time::Duration;

#[actix::test]
async fn test_cutting_actor_integration() {
    // Initialize event bus and registry
    let event_bus = Arc::new(EventBus::new());
    let repository = MaterialRepository::new();
    let registry = MaterialRegistry::new(repository, event_bus.clone());
    
    // Start the cutting actor
    let cutting_actor = CuttingActor::new("IntegrationCuttingActor", registry.clone()).start();
    
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