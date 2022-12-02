use std::net::TcpStream;

use bevy::prelude::*;
use bevy_egui::EguiContext;
use egui::{Color32, InnerResponse, Key, RichText, TextEdit, Ui};
use iyes_loopless::{
    prelude::{AppLooplessStateExt, IntoConditionalSystem},
    state::NextState,
};
use tungstenite::{handshake::client::Response, ClientHandshake, HandshakeError, WebSocket};

use crate::{
    multiplayer::MultiplayerState,
    registry::GameRegistry,
    server::{ActiveGame, ClientMessage, ClientSocket, GameMarker, MessageSocket, ServerMessage},
    SingleplayerState,
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MenuState {
    Loading,
    MainMenu,
    ServerSelect,
    GameSelect,
    InGame,
}

type ClientResult =
    Result<(WebSocket<TcpStream>, Response), HandshakeError<ClientHandshake<TcpStream>>>;

#[derive(Resource, Default)]
struct MenuFields {
    remote_addr: String,
    username: String,
    remote_select_err: &'static str,
    trying_connection: Option<ClientResult>,
}

pub fn standard_window<F, R>(
    ctx: &mut EguiContext,
    add_contents: F,
) -> Option<InnerResponse<Option<R>>>
where
    F: FnOnce(&mut Ui) -> R,
{
    egui::Window::new("")
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .show(ctx.ctx_mut(), add_contents)
}

fn run_main_menu(mut commands: Commands, mut ctx: ResMut<EguiContext>) {
    standard_window(&mut ctx, |ui| {
        ui.vertical_centered(|ui| {
            let initial_height = ui.available_height();
            ui.label(
                RichText::new("Quicksweeper")
                    .size(32.0)
                    .color(Color32::GOLD),
            );
            if ui.button("Singleplayer mode").clicked() {
                commands.insert_resource(NextState(SingleplayerState::PreGame));
                commands.insert_resource(NextState(MenuState::InGame));
            }
            if ui.button("Multiplayer mode").clicked() {
                commands.insert_resource(NextState(MultiplayerState::PreGame));
                commands.insert_resource(NextState(MenuState::InGame));
            }
            if ui.button("Connect to server").clicked() {
                commands.insert_resource(NextState(MenuState::ServerSelect))
            }
            let height = initial_height - ui.available_height();
            ui.set_max_height(height)
        });
    });
}

fn server_select_menu(
    mut commands: Commands,
    mut ctx: ResMut<EguiContext>,
    mut fields: Local<MenuFields>,
) {
    standard_window(&mut ctx, |ui| {
        let (focus_lost, focus_gained) = ui
            .vertical_centered(|ui| {
                ui.colored_label(Color32::RED, fields.remote_select_err);
                let r1 = ui
                    .horizontal(|ui| {
                        ui.label("Server address:");
                        ui.add_enabled(
                            fields.trying_connection.is_none(),
                            TextEdit::singleline(&mut fields.remote_addr),
                        )
                    })
                    .inner;

                let r2 = ui
                    .horizontal(|ui| {
                        ui.label("Username:");
                        ui.add_enabled(
                            fields.trying_connection.is_none(),
                            TextEdit::singleline(&mut fields.username),
                        )
                    })
                    .inner;

                (
                    r1.lost_focus() || r2.lost_focus(),
                    r1.gained_focus() || r2.gained_focus(),
                )
            })
            .inner;

        if fields.trying_connection.is_some() {
            let maybe_handshake = std::mem::replace(&mut fields.trying_connection, None).unwrap();
            match maybe_handshake {
                Ok((socket, _)) => {
                    let mut socket = ClientSocket(socket);
                    socket
                        .send_message(ClientMessage::Greet {
                            username: fields.username.clone(),
                        })
                        .map_err(|_| {
                            fields.remote_select_err = "Failure in initializing connection";
                        })?;

                    socket.send_message(ClientMessage::Games).map_err(|_| {
                        fields.remote_select_err = "Could not retrieve games on the server"
                    })?;

                    commands.spawn((socket,));
                    commands.insert_resource(NextState(MenuState::GameSelect));
                }
                Err(HandshakeError::Interrupted(handshake)) => {
                    fields.trying_connection = Some(handshake.handshake())
                }
                Err(e) => {
                    eprintln!("{e}");
                    fields.remote_select_err = "Failed to perform handshake with server";
                }
            }
        }
        // execute requests to connect to server
        else if focus_lost && ui.input().key_pressed(Key::Enter) {
            let stream = TcpStream::connect(&fields.remote_addr).map_err(|_| {
                fields.remote_select_err = "Could not find that address";
            })?;

            stream.set_nonblocking(true).map_err(|_| {
                fields.remote_select_err = "Unable to set nonblocking connection mode";
            })?;

            let addr = format!("ws://{}/", fields.remote_addr);
            fields.trying_connection = Some(tungstenite::client(addr, stream));
        } else if focus_gained {
            fields.remote_select_err = "";
        }

        Result::<_, ()>::Ok(())
    });
}

/// Event issued when a multiplayer game has been selected. The corresponding game's client
/// implementation should then pick up this event and
struct ToGame(GameMarker);

/// Same thing as [run_menu], but only safe to run with socket access
fn game_select_menu(
    mut commands: Commands,
    mut ctx: ResMut<EguiContext>,
    mut games: Local<Vec<ActiveGame>>,
    mut selected_gamemode: Local<(Option<String>, Option<GameMarker>)>,
    mut socket: Query<&mut ClientSocket>,
    mut start_game: EventWriter<ToGame>,
    registry: Res<GameRegistry>,
) {
    let mut socket = socket.single_mut();

    struct GameSelectResponse {
        go_back: egui::Response,
        reload: egui::Response,
        create: Option<GameMarker>,
        join_game: Option<(Entity, GameMarker)>,
    }

    if let Some(Ok(ServerMessage::ActiveGames(v))) = socket.recv_message() {
        *games = v;
    }

    let response = standard_window(&mut ctx, |ui| {
        egui::Grid::new("window").show(ui, |ui| {
            // header
            let go_back = ui.button("⏴back");
            ui.label("Game select");
            let reload = ui.button("reload🔁");
            ui.end_row();

            let mut join_game = None;
            for game in games.iter() {
                if let Some(descriptor) = registry.get(&game.marker) {
                    ui.vertical(|ui| {
                        ui.label(&descriptor.name);
                        ui.label(&descriptor.description);
                    });

                    ui.label(""); // empty

                    if ui.button("Join").clicked() {
                        join_game = Some((game.id, game.marker))
                    }

                    ui.end_row();
                } else {
                    ui.label("Unsuppored game type");
                }
            }

            egui::ComboBox::from_label("gamemode")
                .selected_text(
                    selected_gamemode
                        .0
                        .as_deref()
                        .unwrap_or("<Choose gamemode>"),
                )
                .show_ui(ui, |ui| {
                    for (&marker, descriptor) in registry.iter() {
                        ui.selectable_value(
                            &mut *selected_gamemode,
                            (Some(descriptor.name.clone()), Some(marker)),
                            &descriptor.name,
                        );
                    }
                });
            let create = ui.button("+create").clicked();
            ui.end_row();

            GameSelectResponse {
                go_back,
                reload,
                create: create.then_some(selected_gamemode.1).flatten(),
                join_game,
            }
        })
    })
    .unwrap()
    .inner
    .unwrap()
    .inner;

    if response.go_back.clicked() {
        commands.insert_resource(NextState(MenuState::MainMenu));
    } else if response.reload.clicked() {
        let _ = socket.send_message(ClientMessage::Games); // TODO: Report error to user
    } else if let Some(mode) = response.create {
        let _ = socket.send_message(ClientMessage::Create {
            game: mode,
            data: Vec::new(),
        });
    } else if let Some((game, marker)) = response.join_game {
        let _ = socket.send_message(ClientMessage::Join { game });
        // TODO: Enter ingame
        start_game.send(ToGame(marker));
        commands.insert_resource(NextState(MenuState::InGame));
    }
}

fn destroy_socket(mut commands: Commands, q: Query<Entity, With<ClientSocket>>) {
    q.for_each(|ent| commands.entity(ent).despawn())
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(MenuState::Loading)
            .add_event::<ToGame>()
            .init_resource::<MenuFields>()
            .add_system(run_main_menu.run_in_state(MenuState::MainMenu))
            .add_system(server_select_menu.run_in_state(MenuState::ServerSelect))
            .add_system(game_select_menu.run_in_state(MenuState::GameSelect))
            .add_enter_system(MenuState::MainMenu, destroy_socket);
    }
}
