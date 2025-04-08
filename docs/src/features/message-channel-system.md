# Message Channel System

## Overview

The Message Channel System implements the communication layer between worker stages in Quilt's processing pipeline, following the actor model architecture. It provides a set of message types and channel utilities for reliable, asynchronous communication with backpressure handling.

## Message Types

The system uses a typed message enum `MaterialMessage` with five variants:

- **Discovered(Material)**: Signals that a new material has been discovered and should be processed.
- **Cut(String)**: Indicates a material (identified by ID) has been cut into swatches.
- **Swatched(String)**: Indicates a material has been fully processed and embedded.
- **Error(String, String)**: Reports an error during processing (material ID, error message).
- **Shutdown**: Special signal to gracefully terminate workers.

## Channel Architecture

The channel system employs the following components:

- **ChannelPair**: A structure containing both sender and receiver for a channel.
- **Default Capacity**: All channels default to a capacity of 100 messages, providing sufficient buffering while still allowing backpressure.
- **Type Aliases**: `CuttingChannel` and `LabelingChannel` provide semantic clarity for each stage.

```rust
pub struct ChannelPair {
    pub sender: mpsc::Sender<MaterialMessage>,
    pub receiver: mpsc::Receiver<MaterialMessage>,
}

pub type CuttingChannel = mpsc::Sender<MaterialMessage>;
pub type LabelingChannel = mpsc::Sender<MaterialMessage>;
```

## Channel Management

Several utilities simplify channel creation and usage:

- **create_channel()**: Creates a channel with default capacity.
- **create_channel_with_capacity()**: Creates a channel with custom capacity.
- **MaterialChannelExt**: Extension trait providing helper methods for senders:
  - `send_message()`: Send a message with error handling
  - `try_send_message_timeout()`: Attempt to send with a timeout
  - `send_shutdown()`: Send a shutdown signal
- **MaterialReceiverExt**: Extension trait for receivers with timeout capabilities.

## Error Handling

The `ChannelError` enum provides structured error handling for:

- Send failures
- Timeout errors
- Channel closure

```rust
pub enum ChannelError {
    SendError(String),
    ReceiveTimeout(Duration),
    ChannelClosed,
}
```

## Usage Example

```rust
// Create channels for the pipeline stages
let discovery_cutting = create_channel();
let cutting_labeling = create_channel();

// Send a material discovery message
let material = Material::new("path/to/document.md".to_string());
discovery_cutting.sender.send_message(
    MaterialMessage::Discovered(material)
).await?;

// Receive and process messages
if let Some(message) = cutting_channel.receiver.recv().await {
    match message {
        MaterialMessage::Discovered(material) => {
            // Process material...

            // Update repository
            repo.update_material_status(&material.id, MaterialStatus::Cut, None).await?;

            // Send to next stage
            labeling_channel.sender.send_message(
                MaterialMessage::Cut(material.id)
            ).await?;
        },
        MaterialMessage::Shutdown => {
            // Propagate shutdown signal
            labeling_channel.sender.send_shutdown().await?;
            break;
        },
        // Handle other message types...
    }
}
```

## Integration with Worker Stages

The Message Channel System connects the three main stages of material processing:

1. **Discovery Stage**: Sends `Discovered` messages when new materials are found
2. **Cutting Stage**: Receives `Discovered` messages, processes materials, and sends `Cut` messages
3. **Labeling Stage**: Receives `Cut` messages and processes them into final embeddings

Each stage uses the repository as a shared data store for tracking material state, while channels handle the passing of processing events between stages.

## Design Decisions

1. **Message Size Optimization**: Cut and Swatched messages contain only material IDs rather than full Material objects to minimize message size.

2. **Backpressure Handling**: Fixed-capacity channels (100 messages) provide natural backpressure when downstream stages can't keep up.

3. **Error Propagation**: Simple error handling follows the "fail fast" principle, with structured error types for easy diagnosis.

4. **Graceful Shutdown**: Dedicated Shutdown message type enables clean termination of the processing pipeline.

5. **Extension Traits**: Helper methods encapsulate common channel operations, improving code clarity and error handling.

## Limitations and Future Enhancements

1. **Monitoring**: Channel depth monitoring will be added in Milestone 4.2.

2. **Dynamic Capacity**: Adaptive channel sizing based on workload will be implemented in Milestone 4.3.

3. **Retry Mechanisms**: Advanced error recovery with retry logic is planned for Milestone 4.3.

4. **Persistence**: Message persistence for crash recovery will be added in Milestone 3.3.
