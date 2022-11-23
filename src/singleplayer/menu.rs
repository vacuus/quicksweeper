use bevy::prelude::*;
use bevy_egui::EguiContext;
use egui::{Color32, RichText};
use iyes_loopless::{
    prelude::{IntoConditionalSystem},
    state::{NextState},
};

use crate::{main_menu::MenuState, SingleplayerState};
use super::minefield::Minefield;

fn fail_screen(mut commands: Commands, ctx: ResMut<EguiContext>, minefield: Query<&Minefield>) {
    let remaining = minefield.single().remaining_blank();

    create_screen(
        &mut commands,
        ctx,
        format!("You failed with {remaining} tiles left"),
    );
}

fn success_screen(mut commands: Commands, ctx: ResMut<EguiContext>) {
    create_screen(
        &mut commands,
        ctx,
        "Congratulations!".to_string(), // TODO: Calculate score
    );
}

pub fn create_screen(commands: &mut Commands, mut ctx: ResMut<EguiContext>, message: String) {
    use SingleplayerState::*;
    egui::Window::new("")
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .show(ctx.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                let initial_height = ui.available_height();
                ui.label(RichText::new(message).size(32.0).color(Color32::GOLD));
                if ui.button("Retry").clicked() {
                    commands.insert_resource(NextState(PreGame));
                }
                if ui.button("Main Menu").clicked() {
                    commands.insert_resource(NextState(Inactive));
                    commands.insert_resource(NextState(MenuState::MainMenu));
                }
                let height = initial_height - ui.available_height();
                ui.set_max_height(dbg!(height))
            });
        });
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(fail_screen.run_in_state(SingleplayerState::GameFailed))
            .add_system(success_screen.run_in_state(SingleplayerState::GameSuccess));
    }
}
