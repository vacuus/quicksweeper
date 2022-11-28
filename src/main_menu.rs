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
    server::{ActiveGame, ClientData, ClientSocket, MessageSocket},
    SingleplayerState,
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MenuState {
    Loading,
    Menu,
    InGame,
}

enum MenuType {
    MainMenu,
    ServerSelect,
    GameSelect,
}
impl Default for MenuType {
    fn default() -> Self {
        Self::MainMenu
    }
}

type ClientResult =
    Result<(WebSocket<TcpStream>, Response), HandshakeError<ClientHandshake<TcpStream>>>;

#[derive(Resource, Default)]
struct MenuFields {
    menu_type: MenuType,
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

fn run_menu(
    mut commands: Commands,
    mut ctx: ResMut<EguiContext>,
    mut fields: ResMut<MenuFields>,
    mut games: Local<Vec<ActiveGame>>,
) {
    standard_window(&mut ctx, |ui| match fields.menu_type {
        MenuType::MainMenu => {
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
                    fields.menu_type = MenuType::ServerSelect;
                }
                let height = initial_height - ui.available_height();
                ui.set_max_height(height)
            });
        }
        MenuType::ServerSelect => {
            ui.vertical_centered(|ui| {
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

                if fields.trying_connection.is_some() {
                    let maybe_handshake =
                        std::mem::replace(&mut fields.trying_connection, None).unwrap();
                    match maybe_handshake {
                        Ok((socket, _)) => {
                            // TODO: Collect username somehow

                            let mut socket = ClientSocket(socket);
                            socket
                                .write_data(ClientData::Greet {
                                    username: fields.username.clone(),
                                })
                                .map_err(|_| {
                                    fields.remote_select_err = "Failure in initializing connection";
                                })?;

                            commands.insert_resource(socket);
                            fields.menu_type = MenuType::GameSelect;
                        }
                        Err(HandshakeError::Interrupted(handshake)) => {
                            fields.trying_connection = Some(handshake.handshake())
                        }
                        Err(_) => {
                            fields.remote_select_err = "Failed to perform handshake with server"
                        }
                    }
                }
                // execute requests to connect to server
                else if (r1.lost_focus() || r2.lost_focus()) && ui.input().key_pressed(Key::Enter)
                {
                    let stream = TcpStream::connect(&fields.remote_addr).map_err(|_| {
                        fields.remote_select_err = "Could not find that address";
                    })?;

                    stream.set_nonblocking(true).map_err(|_| {
                        fields.remote_select_err = "Unable to set nonblocking connection mode";
                    })?;

                    let addr = format!("ws://{}/", fields.remote_addr);
                    fields.trying_connection = Some(tungstenite::client(addr, stream));
                } else if r1.gained_focus() || r2.gained_focus() {
                    fields.remote_select_err = "";
                }

                Result::<_, ()>::Ok(())
            });
        }
        MenuType::GameSelect => {
            ui.label("Running games");
        }
    });
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(MenuState::Loading)
            .init_resource::<MenuFields>()
            .add_system(run_menu.run_in_state(MenuState::Menu))
            .add_enter_system(MenuState::Menu, |mut t: ResMut<MenuFields>| {
                t.menu_type = MenuType::MainMenu
            });
    }
}
