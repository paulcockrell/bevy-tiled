use bevy::prelude::*;

use crate::{Inventory, Player};

#[derive(Component, Debug)]
struct OnHud;

#[derive(Component, Debug)]
struct OnLevel;

#[derive(Component, Debug)]
pub struct OnInventory;

#[derive(Component, Debug)]
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud)
            .add_systems(Update, update_hud_inventory);
    }
}
fn update_hud_inventory(
    inventory_query: Query<&Inventory, With<Player>>,
    mut query: Query<&mut Text, With<OnInventory>>,
) {
    let Ok(inventory) = inventory_query.get_single() else {
        println!("Found mutliple inventories");
        return;
    };

    for mut text in query.iter_mut() {
        // Change the content of our text
        text.sections[0].value = inventory.to_string();
    }
}

fn setup_hud(mut commands: Commands) {
    let level_text = "Level 1";
    let inventory_text = "";

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Vw(100.0),
                    height: Val::Percent(10.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::BLACK),
                ..default()
            },
            OnHud,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        height: Val::Percent(100.0),
                        width: Val::Vw(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // Display level info
                    parent.spawn((
                        TextBundle::from_section(
                            level_text,
                            TextStyle {
                                font_size: 20.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        }),
                        OnLevel,
                    ));
                    // Display controlls
                    parent.spawn((
                        TextBundle::from_section(
                            inventory_text,
                            TextStyle {
                                font_size: 20.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        }),
                        OnInventory,
                    ));
                });
        });
}
