# Material Registry

The Material Registry is Quilt's central store for managing materials. It maintains the collection of all discovered materials, handles their registration, and emits events when their status changes.

## Overview

The Material Registry is responsible for:

- Storing all discovered materials
- Ensuring uniqueness of material paths
- Managing material status changes
- Emitting events for material lifecycle changes
- Providing lookup capabilities by ID and path

## Basic Usage

The most basic way to use the Material Registry is to create one and register materials:

```rust
let mut registry = MaterialRegistry::new();

// Register a new material
let material = Material::new("docs/example.md".to_string());
if let Some(registered) = registry.upsert(material) {
    println!("Material registered: {}", registered.id);
}
```

## Core Features

### Material Storage

The registry stores materials indexed by their unique ID, while also maintaining path uniqueness:

```rust
// Get by ID
if let Some(material) = registry.get("material_id") {
    println!("Found material: {}", material.file_path);
}

// Get by path
if let Some(material) = registry.get_by_path("docs/example.md") {
    println!("Found material with ID: {}", material.id);
}

// List all materials
let all_materials = registry.list_all();
```

### Event Emission

The registry emits events when materials change status, allowing other components to react to these changes:

```rust
// Listen for material events
registry.on(|event| match event {
    MaterialEvent::StatusChanged { material, old_status, error } => {
        println!("Material {} status changed from {:?} to {}",
            material.id,
            old_status,
            material.status);
    }
});
```

## Common Scenarios

### Registering New Materials

```gherkin
Scenario: New Material Registration
  Given a new material with a unique path
  When the material is registered with the registry
  Then it is stored successfully
  And a StatusChanged event is emitted
  And the event has no previous status

Scenario: Duplicate Path Registration
  Given an existing material in the registry
  When a new material with the same path is registered
  Then the registration fails
  And no event is emitted
  And the original material remains unchanged
```

### Updating Materials

```gherkin
Scenario: Material Status Update
  Given an existing material in the registry
  When the material's status is changed
  Then the material is updated in the registry
  And a StatusChanged event is emitted
  And the event includes the previous status

Scenario: Material Validation
  Given a discovered material
  When the material is validated
  Then its status is updated to Valid or Invalid
  And a StatusChanged event is emitted
  And any validation errors are included in the event
```

### Material Lookup

```gherkin
Scenario: Lookup By ID
  Given materials in the registry
  When a lookup by ID is performed
  Then the exact matching material is returned

Scenario: Lookup By Path
  Given materials in the registry
  When a lookup by file path is performed
  Then the material with matching path is returned
```

## Current Status

- ✅ Material storage and indexing
- ✅ Path uniqueness enforcement
- ✅ Material status tracking
- ✅ Event emission for status changes
- ✅ Lookup by ID and path
- ✅ List all materials

## Next Steps

Future improvements being considered:

1. Batch operations for registering multiple materials
2. Material metadata storage and indexing
3. Material relationship tracking
4. Query capabilities for finding related materials
5. Material versioning and history tracking
