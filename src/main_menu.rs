use bevy::prelude::*;
use bevy_egui::EguiContext;
use egui::{Color32, RichText};
use iyes_loopless::{
    prelude::{AppLooplessStateExt, IntoConditionalSystem},
    state::NextState,
};

use crate::{multiplayer::MultiplayerState, SingleplayerState};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MenuState {
    Loading,
    MainMenu,
    InGame,
}

fn create_main_menu(mut commands: Commands, mut ctx: ResMut<EguiContext>) {
    egui::Window::new("")
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .show(ctx.ctx_mut(), |ui| {
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
        });
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(MenuState::Loading)
            .add_system(create_main_menu.run_in_state(MenuState::MainMenu))
            .add_enter_system(MenuState::MainMenu, || println!("Finished loading"));
    }
}
