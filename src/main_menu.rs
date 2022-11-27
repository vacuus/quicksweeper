use bevy::prelude::*;
use bevy_egui::EguiContext;
use egui::{Color32, InnerResponse, RichText, Ui, Key};
use iyes_loopless::{
    prelude::{AppLooplessStateExt, IntoConditionalSystem},
    state::NextState,
};

use crate::{multiplayer::MultiplayerState, SingleplayerState};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MenuState {
    Loading,
    Menu,
    InGame,
}

#[derive(Resource)]
enum MenuType {
    MainMenu,
    ServerSelect,
    GameSelect,
}

#[derive(Resource, Default)]
struct MenuFields {
    remote_addr: String,
}

impl Default for MenuType {
    fn default() -> Self {
        Self::MainMenu
    }
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
    mut state: ResMut<MenuType>,
    mut fields: ResMut<MenuFields>,
) {
    standard_window(&mut ctx, |ui| match *state {
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
                    *state = MenuType::ServerSelect;
                }
                let height = initial_height - ui.available_height();
                ui.set_max_height(height)
            });
        }
        MenuType::ServerSelect => {
            ui.horizontal(|ui| {
                ui.label("Server address:");
                let response = ui.text_edit_singleline(&mut fields.remote_addr);
                if response.lost_focus() && ui.input().key_pressed(Key::Enter) {
                    println!("Received address: {}", fields.remote_addr);
                }
            });
        }
        MenuType::GameSelect => todo!(),
    });
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(MenuState::Loading)
            .init_resource::<MenuType>()
            .init_resource::<MenuFields>()
            .add_system(run_menu.run_in_state(MenuState::Menu))
            .add_enter_system(MenuState::Menu, |mut t: ResMut<MenuType>| {
                *t = MenuType::MainMenu
            });
    }
}
