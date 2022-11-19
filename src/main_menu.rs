use bevy::prelude::*;
use bevy_egui::EguiContext;
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

fn create_main_menu(
    mut commands: Commands,
    mut ctx: ResMut<EguiContext>,
    window_props: Res<WindowDescriptor>,
) {
    println!("{}, {}", window_props.width, window_props.height);

    egui::Area::new("main_menu")
        .fixed_pos([window_props.width / 2.0, window_props.height / 2.0])
        .show(ctx.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                ui.label("Quicksweeper");
                if ui.button("Singleplayer mode").clicked() {
                    commands.insert_resource(NextState(SingleplayerState::PreGame));
                    commands.insert_resource(NextState(MenuState::InGame));
                }
                if ui.button("Multiplayer mode").clicked() {
                    commands.insert_resource(NextState(MultiplayerState::PreGame));
                    commands.insert_resource(NextState(MenuState::InGame));
                }
                ui.shrink_height_to_current();
            });
        });

    dbg!(ctx.ctx_mut().used_size());
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(MenuState::Loading)
            .add_system(create_main_menu.run_in_state(MenuState::MainMenu))
            .add_enter_system(MenuState::MainMenu, || println!("Finished loading"));
    }
}
