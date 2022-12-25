use bevy::prelude::*;
use bevy_egui::EguiContext;
use egui::{Color32, RichText};
use iyes_loopless::state::CurrentState;
use iyes_loopless::{prelude::IntoConditionalSystem, state::NextState};

use crate::minefield::Minefield;
use crate::{
    main_menu::{standard_window, Menu},
    Singleplayer,
};

fn fail_screen(mut commands: Commands, ctx: ResMut<EguiContext>, minefield: Query<&Minefield>) {
    let remaining = minefield.single().remaining_blank;

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

fn create_screen(commands: &mut Commands, mut ctx: ResMut<EguiContext>, message: String) {
    use Singleplayer::*;
    standard_window(&mut ctx, |ui| {
        ui.vertical_centered(|ui| {
            let initial_height = ui.available_height();
            ui.label(RichText::new(message).size(32.0).color(Color32::GOLD));
            if ui.button("Retry").clicked() {
                commands.insert_resource(NextState(PreGame));
            }
            if ui.button("Main Menu").clicked() {
                commands.insert_resource(NextState(Inactive));
                commands.insert_resource(NextState(Menu::MainMenu));
            }
            let height = initial_height - ui.available_height();
            ui.set_max_height(height)
        });
    });
}

fn pause_menu(mut commands: Commands, mut ctx: ResMut<EguiContext>) {
    standard_window(&mut ctx, |ui| {
        ui.vertical_centered(|ui| {
            if ui.button("resume").clicked() {
                commands.insert_resource(NextState(Menu::Ingame));
            };
            if ui.button("retry").clicked() {
                commands.insert_resource(NextState(Singleplayer::PreGame));
                commands.insert_resource(NextState(Menu::Ingame));
            };
            if ui.button("exit").clicked() {
                commands.insert_resource(NextState(Menu::MainMenu));
                commands.insert_resource(NextState(Singleplayer::Inactive));
            };
        })
    });
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(fail_screen.run_in_state(Singleplayer::GameFailed))
            .add_system(success_screen.run_in_state(Singleplayer::GameSuccess))
            .add_system(pause_menu.run_if(
                |s1: Res<CurrentState<Singleplayer>>, s2: Res<CurrentState<Menu>>| {
                    s1.0 != Singleplayer::Inactive && s2.0 == Menu::Pause
                },
            ));
    }
}
