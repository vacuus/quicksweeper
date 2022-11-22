use bevy::prelude::*;
use bevy_egui::EguiContext;
use egui::{Color32, RichText};
use iyes_loopless::{
    prelude::{AppLooplessStateExt, ConditionSet, IntoConditionalSystem},
    state::{CurrentState, NextState},
};

use crate::{load::Textures, main_menu::MenuState, minefield::Minefield, SingleplayerState};

// #[derive(Component)]
// struct CompleteScreen;

// #[derive(Component)]
// struct RetryButton;
// #[derive(Component)]
// struct MainMenuButton;

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

// fn retry(
//     mut commands: Commands,
//     ev: Query<&Interaction, (Changed<Interaction>, With<RetryButton>)>,
//     // ui: Query<Entity, With<CompleteScreen>>,
// ) {
//     if ev.iter().any(|x| *x == Interaction::Clicked) {
//         commands.insert_resource(NextState(SingleplayerState::PreGame));
//         // commands.entity(ui.single()).despawn_recursive();
//     }
// }

// fn main_menu(
//     mut commands: Commands,
//     ev: Query<&Interaction, (Changed<Interaction>, With<MainMenuButton>)>,
// ) {
//     if ev.iter().any(|x| *x == Interaction::Clicked) {
//         commands.insert_resource(NextState(SingleplayerState::Inactive));
//         commands.insert_resource(NextState(MenuState::MainMenu));
//     }
// }

// fn close_menu(mut commands: Commands, ui: Query<Entity, With<CompleteScreen>>) {
//     ui.for_each(|x| commands.entity(x).despawn_recursive())
// }

pub fn create_screen(commands: &mut Commands, mut ctx: ResMut<EguiContext>, message: String) {
    // let text_style = || TextStyle {
    //     font_size: 40.0,
    //     color: Color::RED,
    //     font: font_source.font.clone(),
    // };
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

    // commands
    //     .spawn(NodeBundle {
    //         style: Style {
    //             flex_direction: FlexDirection::Column,
    //             align_items: AlignItems::Center,
    //             margin: UiRect::all(Val::Auto),
    //             ..default()
    //         },
    //         background_color: Color::NONE.into(),
    //         ..default()
    //     })
    //     .insert(CompleteScreen)
    //     .with_children(|parent| {
    //         // text
    //         parent
    //             .spawn(NodeBundle {
    //                 background_color: Color::WHITE.into(),
    //                 ..default()
    //             })
    //             .with_children(|parent| {
    //                 parent.spawn(TextBundle {
    //                     text: Text::from_section(message, text_style()),
    //                     ..default()
    //                 });
    //             });
    //         // button
    //         parent
    //             .spawn(ButtonBundle {
    //                 style: Style {
    //                     size: Size::new(Val::Px(150.0), Val::Px(65.0)),
    //                     justify_content: JustifyContent::Center,
    //                     align_items: AlignItems::Center,
    //                     ..default()
    //                 },
    //                 background_color: Color::MIDNIGHT_BLUE.into(),
    //                 ..default()
    //             })
    //             .insert(RetryButton)
    //             .with_children(|parent| {
    //                 parent.spawn(TextBundle {
    //                     text: Text::from_section("Retry", text_style()),
    //                     ..default()
    //                 });
    //             });
    //         parent
    //             .spawn(ButtonBundle {
    //                 style: Style {
    //                     size: Size::new(Val::Px(150.0), Val::Px(65.0)),
    //                     justify_content: JustifyContent::Center,
    //                     align_items: AlignItems::Center,
    //                     ..default()
    //                 },
    //                 background_color: Color::MIDNIGHT_BLUE.into(),
    //                 ..default()
    //             })
    //             .insert(MainMenuButton)
    //             .with_children(|parent| {
    //                 parent.spawn(TextBundle {
    //                     text: Text::from_section("Main Menu", text_style()),
    //                     ..default()
    //                 });
    //             });
    //     });
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(fail_screen.run_in_state(SingleplayerState::GameFailed))
            .add_system(success_screen.run_in_state(SingleplayerState::GameSuccess));
            // .add_system_set(
            //     ConditionSet::new()
            //         .run_if(|state: Res<CurrentState<SingleplayerState>>| {
            //             [
            //                 SingleplayerState::GameFailed,
            //                 SingleplayerState::GameSuccess,
            //             ]
            //             .contains(&state.0)
            //         })
            //         .with_system(retry)
            //         .with_system(main_menu)
            //         .into(),
            // )
            // .add_exit_system(SingleplayerState::GameFailed, close_menu)
            // .add_exit_system(SingleplayerState::GameSuccess, close_menu);
    }
}
