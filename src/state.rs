
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum AppState {
    Menu,
    Loading,
    PreGame,
    Game,
    GameFailed,
}