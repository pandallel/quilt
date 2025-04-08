use actix::prelude::*;
use log::{debug, info};
use crate::actors::{Ping, Shutdown};

/// Messages specific to the DiscoveryActor
pub mod messages {
    use actix::prelude::*;
    
    /// Command to start discovery in a directory
    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct StartDiscovery {
        pub directory: String,
    }
}

/// Actor responsible for discovering materials in directories
pub struct DiscoveryActor {
    name: String,
}

impl DiscoveryActor {
    /// Create a new DiscoveryActor with the given name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Actor for DiscoveryActor {
    type Context = Context<Self>;
    
    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("DiscoveryActor '{}' started", self.name);
    }
    
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("DiscoveryActor '{}' stopped", self.name);
    }
}

/// Handler for Ping messages
impl Handler<Ping> for DiscoveryActor {
    type Result = bool;
    
    fn handle(&mut self, _msg: Ping, _ctx: &mut Self::Context) -> Self::Result {
        debug!("DiscoveryActor '{}' received ping", self.name);
        true
    }
}

/// Handler for Shutdown messages
impl Handler<Shutdown> for DiscoveryActor {
    type Result = ();
    
    fn handle(&mut self, _msg: Shutdown, ctx: &mut Self::Context) -> Self::Result {
        info!("DiscoveryActor '{}' shutting down", self.name);
        ctx.stop();
    }
}

/// Handler for StartDiscovery messages
impl Handler<messages::StartDiscovery> for DiscoveryActor {
    type Result = ();
    
    fn handle(&mut self, msg: messages::StartDiscovery, _ctx: &mut Self::Context) -> Self::Result {
        info!("DiscoveryActor '{}' starting discovery in '{}'", self.name, msg.directory);
        println!("Discovery started in directory: {}", msg.directory);
    }
} 