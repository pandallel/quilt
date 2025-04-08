pub mod types;
// registry module removed as it needs significant refactoring
pub mod channels;
pub mod messages;
pub mod repository;
pub mod scanner;

#[cfg(test)]
mod tests;

pub use types::{Material, MaterialFileType, MaterialStatus};
// Material registry will be reimplemented for actor model
pub use channels::{
    create_channel, create_channel_with_capacity, ChannelError, ChannelPair, CuttingChannel,
    LabelingChannel, MaterialChannelExt, MaterialReceiverExt, DEFAULT_CHANNEL_CAPACITY,
};
pub use messages::MaterialMessage;
pub use repository::{MaterialRepository, RepositoryError};
pub use scanner::{DirectoryScanner, ScanError, ScanResult, ScanResults};
