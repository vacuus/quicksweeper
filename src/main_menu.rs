use bevy::prelude::*;
use bevy_egui::EguiContext;
use egui::{Color32, InnerResponse, RichText, Ui};
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

fn create_menu(mut commands: Commands, mut ctx: ResMut<EguiContext>, mut state: ResMut<MenuType>) {
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
                let height = initial_height - ui.available_height();
                ui.set_max_height(height)
            });
        }
        MenuType::ServerSelect => todo!(),
        MenuType::GameSelect => todo!(),
    });
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(MenuState::Loading)
            .init_resource::<MenuType>()
            .add_system(create_menu.run_in_state(MenuState::Menu))
            .add_enter_system(MenuState::Menu, |mut t: ResMut<MenuType>| {
                *t = MenuType::MainMenu
            });
    }
}
