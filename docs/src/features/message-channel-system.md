# Message Channel System

The message channel system in Quilt uses direct actor-to-actor communication through Tokio channels. Each actor has its own specific message types for clear communication boundaries.

## Message Types

Each stage of the pipeline has dedicated message types:

```rust
// Discovery to Cutting
pub struct MaterialDiscovered {
    pub material: Material,  // Full material for initial registration
}

// Cutting to Swatching
pub struct CutMaterial {
    pub material_id: String,  // Reference to original material
    pub cut_ids: Vec<String>, // IDs of the generated cuts
}

// Swatching completion
pub struct MaterialSwatched {
    pub material_id: String,  // Reference to original material
    pub cut_id: String,      // Reference to the cut that was processed
    pub swatch_id: String,   // ID of the generated swatch
}

// Error handling
pub struct ProcessingError {
    pub material_id: String,
    pub stage: ProcessingStage,
    pub error: String,
}

pub enum ProcessingStage {
    Discovery,
    Cutting,
    Swatching,
}
```

## Channel Types

Each stage uses dedicated channel types for message passing:

```rust
// Channel types for each stage
pub type DiscoverySender = mpsc::Sender<MaterialDiscovered>;
pub type CuttingSender = mpsc::Sender<CutMaterial>;
pub type SwatchingSender = mpsc::Sender<MaterialSwatched>;
```

## Message Flow

1. Discovery Actor:

   - When a new material is found, creates a MaterialDiscovered message
   - Sends the message to the Cutting Actor via DiscoverySender

2. Cutting Actor:

   - Receives MaterialDiscovered messages
   - Processes the material into cuts
   - Sends CutMaterial messages to the Swatching Actor

3. Swatching Actor:
   - Receives CutMaterial messages
   - Processes cuts into swatches
   - Updates the repository with completed swatches

## Error Handling

- Each stage can generate ProcessingError messages
- Errors are logged and the material's state is updated in the repository
- The pipeline continues processing other materials while handling errors

## Backpressure

- Channels use a fixed capacity (100 messages) to prevent memory exhaustion
- When a channel is full, senders will wait until space is available
- This creates natural backpressure through the pipeline
