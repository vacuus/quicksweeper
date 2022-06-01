
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum AppState {
    Menu,
    PreGame,
    Game,
    GameFailed,
}