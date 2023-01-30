use async_trait::async_trait;
use tokio::task::JoinHandle;

use super::{app::GameConnector, double_channel::DoubleChannel};

pub struct GameComponents {
    /// The channel with which the game communicates as a host. This object has no explicit
    /// priveliges, but it is guaranteed by the server that this channel will be passed to the
    /// player that initialized the game.
    host_channel: DoubleChannel<Vec<u8>>,
    connector: GameConnector,
    main_task: JoinHandle<()>,
}

/// Trait which provides the initialization behavior (and thus the general behavior) of a particular
/// gamemode. The sole function is meant to spawn a task which provides the main behavior of the
/// game, and return the join handle for that task as well as some objects for manipulation of the
/// game.
#[async_trait]
pub trait QuicksweeperGame {
    async fn create(params: Vec<u8>) -> (DoubleChannel<Vec<u8>>, GameConnector, JoinHandle<()>);
}
