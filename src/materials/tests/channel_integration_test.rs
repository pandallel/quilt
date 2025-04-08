#[cfg(test)]
mod integration_tests {
    use std::time::Duration;
    use crate::materials::{
        Material, 
        MaterialStatus,
        MaterialMessage,
        MaterialRepository,
        ChannelPair,
        create_channel,
        create_channel_with_capacity,
        MaterialChannelExt,
    };

    // Simulates the basic workflow of the three-stage pipeline
    #[tokio::test]
    async fn test_pipeline_message_flow() {
        // Create a shared repository
        let repo = MaterialRepository::new();

        // Create channels for each stage
        let mut discovery_cutting = create_channel();
        let mut cutting_labeling = create_channel();

        // Create a test material
        let material = Material::new("test/document.md".to_string());
        let material_id = material.id.clone();

        // Store the material in the repository
        repo.register_material(material.clone()).await.unwrap();

        // Stage 1: Discovery sends a Discovered message
        discovery_cutting.sender.send_message(MaterialMessage::Discovered(material.clone())).await.unwrap();

        // Stage 2: Cutting receives the Discovered message
        let received_message = discovery_cutting.receiver.recv().await.unwrap();
        
        if let MaterialMessage::Discovered(received_material) = received_message {
            assert_eq!(received_material.id, material_id);
            
            // Update material status in repository to Cut
            repo.update_material_status(&material_id, MaterialStatus::Cut, None).await.unwrap();
            
            // Send Cut message to Labeling stage
            cutting_labeling.sender.send_message(MaterialMessage::Cut(material_id.clone())).await.unwrap();
        } else {
            panic!("Expected Discovered message");
        }

        // Stage 3: Labeling receives the Cut message
        let received_message = cutting_labeling.receiver.recv().await.unwrap();
        
        if let MaterialMessage::Cut(id) = received_message {
            assert_eq!(id, material_id);
            
            // Update material status in repository to Swatched
            repo.update_material_status(&material_id, MaterialStatus::Swatched, None).await.unwrap();
        } else {
            panic!("Expected Cut message");
        }

        // Verify final material status
        let final_material = repo.get_material(&material_id).await.unwrap();
        assert_eq!(final_material.status, MaterialStatus::Swatched);
    }

    // Test backpressure behavior when channel is full
    #[tokio::test]
    async fn test_channel_backpressure() {
        // Create a channel with very small capacity
        let ChannelPair { sender, mut receiver } = create_channel_with_capacity(2);
        
        // Create test materials
        let material1 = Material::new("test/doc1.md".to_string());
        let material2 = Material::new("test/doc2.md".to_string());
        let material3 = Material::new("test/doc3.md".to_string());
        
        // Send two messages to fill the channel
        sender.send_message(MaterialMessage::Discovered(material1)).await.unwrap();
        sender.send_message(MaterialMessage::Discovered(material2)).await.unwrap();
        
        // Try to send a third message with a short timeout
        // This should block or timeout because the channel is full
        let result = sender.try_send_message_timeout(
            MaterialMessage::Discovered(material3),
            Duration::from_millis(100)
        ).await;
        
        // We expect this to timeout because the channel is full
        assert!(result.is_err());
        
        // Receive a message to make room
        let _ = receiver.recv().await.unwrap();
        
        // Now we should be able to send a message
        sender.send_message(MaterialMessage::Shutdown).await.unwrap();
    }

    // Test shutdown signal propagation
    #[tokio::test]
    async fn test_shutdown_propagation() {
        // Create channels for each stage
        let mut discovery_cutting = create_channel();
        let mut cutting_labeling = create_channel();
        
        // Send shutdown to first stage
        discovery_cutting.sender.send_shutdown().await.unwrap();
        
        // Receive at cutting stage
        let message = discovery_cutting.receiver.recv().await.unwrap();
        assert!(matches!(message, MaterialMessage::Shutdown));
        
        // Propagate shutdown to next stage
        if matches!(message, MaterialMessage::Shutdown) {
            cutting_labeling.sender.send_shutdown().await.unwrap();
        }
        
        // Receive at labeling stage
        let message = cutting_labeling.receiver.recv().await.unwrap();
        assert!(matches!(message, MaterialMessage::Shutdown));
    }
} 