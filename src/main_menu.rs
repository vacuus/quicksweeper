use bevy::prelude::*;
use bevy_egui::EguiContext;
use egui::vec2;
use iyes_loopless::prelude::{AppLooplessStateExt, IntoConditionalSystem};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MenuState {
    Loading,
    MainMenu,
}

fn create_main_menu(mut ctx: ResMut<EguiContext>, window_props: Res<WindowDescriptor>) {
    egui::Area::new("main_menu")
        .fixed_pos([window_props.width/2.0, window_props.height/2.0])
        .show(ctx.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                ui.label("Quicksweeper");
                if ui.button("Singleplayer mode").clicked() {
                    // TODO: Transition to singleplayer mode
                }
            })
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
