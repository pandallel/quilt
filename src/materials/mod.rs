pub mod types;
// registry module removed as it needs significant refactoring
pub mod scanner;
pub mod repository;
pub mod messages;
pub mod channels;

#[cfg(test)]
mod tests;

pub use types::{Material, MaterialStatus, MaterialFileType};
// Material registry will be reimplemented for actor model
pub use scanner::{DirectoryScanner, ScanError, ScanResult, ScanResults};
pub use repository::{MaterialRepository, RepositoryError};
pub use messages::MaterialMessage;
pub use channels::{
    ChannelPair, 
    ChannelError, 
    CuttingChannel, 
    LabelingChannel,
    create_channel, 
    create_channel_with_capacity,
    MaterialChannelExt,
    MaterialReceiverExt,
    DEFAULT_CHANNEL_CAPACITY,
}; 