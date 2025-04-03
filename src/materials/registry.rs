use std::collections::HashMap;
use event_emitter::{EventEmitter, EventListener};
use crate::materials::types::{Material, MaterialEvent, MaterialStatus};

/// A registry for storing and managing materials with event emission
///
/// Note: Current implementation is not thread-safe. If concurrent access is needed,
/// consider using tokio::sync::RwLock or implementing proper error handling for std::sync::RwLock
#[derive(Default)]
pub struct MaterialRegistry {
    /// Storage for materials, indexed by their ID
    materials: HashMap<String, Material>,
    /// Event emitter for material events
    events: EventEmitter<MaterialEvent>,
}

impl MaterialRegistry {
    /// Create a new empty MaterialRegistry
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
            events: EventEmitter::new(),
        }
    }

    /// Register a new material in the registry
    /// Returns None if the material was already registered (based on file path)
    pub fn register(&mut self, material: Material) -> Option<Material> {
        // Check if material with same path exists
        if self.materials.values().any(|m| m.file_path == material.file_path) {
            return None;
        }

        let material_clone = material.clone();
        self.materials.insert(material.id.clone(), material);
        
        // Emit status changed event with no old status (initial discovery)
        self.events.emit(MaterialEvent::StatusChanged {
            material: material_clone.clone(),
            old_status: None,
            error: None,
        });
        
        Some(material_clone)
    }

    /// Get a material by its ID
    pub fn get(&self, id: &str) -> Option<Material> {
        self.materials.get(id).cloned()
    }

    /// Get a material by its file path
    pub fn get_by_path(&self, file_path: &str) -> Option<Material> {
        self.materials
            .values()
            .find(|m| m.file_path == file_path)
            .cloned()
    }

    /// Update an existing material
    /// Returns None if the material wasn't found
    pub fn update(&mut self, material: Material) -> Option<Material> {
        if let Some(old_material) = self.materials.get(&material.id) {
            let old_status = old_material.status.clone();
            
            // Only emit event if status changed
            if material.status != old_status {
                self.events.emit(MaterialEvent::StatusChanged {
                    material: material.clone(),
                    old_status: Some(old_status),
                    error: material.error.clone(),
                });
            }
            
            self.materials.insert(material.id.clone(), material.clone());
            Some(material)
        } else {
            None
        }
    }

    /// Remove a material from the registry by its ID
    pub fn remove(&mut self, id: &str) -> Option<Material> {
        self.materials.remove(id)
    }

    /// Subscribe to status change events
    pub fn on<F>(&self, callback: F) -> EventListener
    where
        F: Fn(&MaterialEvent) + Send + Sync + 'static,
    {
        self.events.on(callback)
    }

    /// Get all materials in the registry
    pub fn list_all(&self) -> Vec<Material> {
        self.materials
            .values()
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    // Test helpers
    fn setup() -> MaterialRegistry {
        MaterialRegistry::new()
    }

    fn create_test_material(path: &str) -> Material {
        Material::new(path.to_string())
    }

    fn collect_events(registry: &MaterialRegistry) -> (RefCell<Vec<MaterialEvent>>, EventListener) {
        let events = RefCell::new(Vec::new());
        let listener = registry.on(move |event| {
            if let MaterialEvent::StatusChanged { material, old_status, error } = event {
                events.borrow_mut().push(MaterialEvent::StatusChanged {
                    material: material.clone(),
                    old_status: old_status.clone(),
                    error: error.clone(),
                });
            }
        });
        (events, listener)
    }

    #[test]
    fn register_new_material_succeeds() {
        let mut registry = setup();
        let material = create_test_material("test/doc.md");
        let material_id = material.id.clone();

        let registered = registry.register(material)
            .expect("Should successfully register new material");
        assert_eq!(registered.id, material_id);
    }

    #[test]
    fn register_duplicate_path_fails() {
        let mut registry = setup();
        let material = create_test_material("test/doc.md");
        registry.register(material).expect("First registration should succeed");

        let duplicate = create_test_material("test/doc.md");
        assert!(registry.register(duplicate).is_none(), "Should reject duplicate path");
    }

    #[test]
    fn get_material_by_id_succeeds() {
        let mut registry = setup();
        let material = create_test_material("test/doc.md");
        let material_id = material.id.clone();

        registry.register(material).expect("Registration should succeed");

        let retrieved = registry.get(&material_id)
            .expect("Should find material by ID");
        assert_eq!(retrieved.id, material_id);
    }

    #[test]
    fn get_material_by_path_succeeds() {
        let mut registry = setup();
        let material = create_test_material("test/doc.md");
        let material_id = material.id.clone();

        registry.register(material).expect("Registration should succeed");

        let by_path = registry.get_by_path("test/doc.md")
            .expect("Should find material by path");
        assert_eq!(by_path.id, material_id);
    }

    #[test]
    fn initial_discovery_emits_correct_event() {
        let mut registry = setup();
        let (events, _listener) = collect_events(&registry);

        let material = create_test_material("test/doc.md");
        registry.register(material).expect("Registration should succeed");

        let events = events.borrow();
        assert_eq!(events.len(), 1, "Should emit exactly one event");
        
        if let MaterialEvent::StatusChanged { material, old_status, error } = &events[0] {
            assert_eq!(material.status, MaterialStatus::Discovered, 
                "New material should be in Discovered state");
            assert!(old_status.is_none(), 
                "Initial discovery should have no previous status");
            assert!(error.is_none(), 
                "Initial discovery should have no error");
        }
    }

    #[test]
    fn successful_validation_emits_correct_event() {
        let mut registry = setup();
        let (events, _listener) = collect_events(&registry);

        // Register and validate
        let mut material = create_test_material("test/doc.md");
        registry.register(material.clone()).expect("Registration should succeed");
        
        material.status = MaterialStatus::Valid;
        registry.update(material).expect("Update should succeed");

        let events = events.borrow();
        let validation_event = &events[1];  // Skip discovery event

        if let MaterialEvent::StatusChanged { material, old_status, error } = validation_event {
            assert_eq!(material.status, MaterialStatus::Valid, 
                "Material should be marked as Valid");
            assert_eq!(old_status.as_ref().expect("Should have previous status"), 
                &MaterialStatus::Discovered,
                "Previous status should be Discovered");
            assert!(error.is_none(), 
                "Successful validation should have no error");
        }
    }

    #[test]
    fn failed_validation_emits_correct_event() {
        let mut registry = setup();
        let (events, _listener) = collect_events(&registry);

        // Register and fail validation
        let mut material = create_test_material("test/doc.md");
        registry.register(material.clone()).expect("Registration should succeed");
        
        material.status = MaterialStatus::Invalid;
        material.error = Some("Invalid format".to_string());
        registry.update(material).expect("Update should succeed");

        let events = events.borrow();
        let failure_event = &events[1];  // Skip discovery event

        if let MaterialEvent::StatusChanged { material, old_status, error } = failure_event {
            assert_eq!(material.status, MaterialStatus::Invalid, 
                "Material should be marked as Invalid");
            assert_eq!(old_status.as_ref().expect("Should have previous status"),
                &MaterialStatus::Discovered,
                "Previous status should be Discovered");
            assert_eq!(error.as_ref().expect("Should have error message"),
                "Invalid format",
                "Should have correct error message");
        }
    }

    #[test]
    fn list_all_returns_all_materials() {
        let mut registry = setup();
        
        registry.register(create_test_material("test/doc1.md"))
            .expect("First registration should succeed");
        registry.register(create_test_material("test/doc2.md"))
            .expect("Second registration should succeed");
        
        let all_materials = registry.list_all();
        assert_eq!(all_materials.len(), 2, "Should return all registered materials");
    }
} 