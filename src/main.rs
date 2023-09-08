#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::time::Duration;

use bevy::{
    app::AppExit,
    input::gamepad::{GamepadRumbleIntensity, GamepadRumbleRequest},
    prelude::*,
    sprite::collide_aabb::collide,
    text::BreakLineOn,
    window::WindowMode,
};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use rand::prelude::*;

const COLORS: [Color; 12] = [
    Color::GOLD,
    Color::CRIMSON,
    Color::PURPLE,
    Color::RED,
    Color::ORANGE_RED,
    Color::ORANGE,
    Color::PINK,
    Color::SALMON,
    Color::TOMATO,
    Color::LIME_GREEN,
    Color::BLUE,
    Color::YELLOW,
];

const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 640.0;
const SPEED: f32 = 300.0;
const PLAYER_SIZE: Vec3 = Vec3::new(30.0, 30.0, 0.0);
const TARGET_SIZE: Vec3 = Vec3::new(28.0, 28.0, 0.0);
const TEXT_BAR_HEIGHT: f32 = 32.0;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Target;

#[derive(Component)]
struct Collider;

#[derive(Event, Default)]
struct HitEvent;

#[derive(Resource)]
struct HitSound(Handle<AudioSource>);

#[derive(Resource)]
struct Scoreboard {
    score: usize,
}

#[derive(Component)]
struct ScoreText;

#[derive(Resource)]
struct MyGamepad(Gamepad);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        mode: WindowMode::Windowed,
                        resolution: (WIDTH, HEIGHT).into(),
                        title: "Game of Squares".to_string(),
                        resizable: false,
                        // Bind to canvas included in `index.html`
                        canvas: Some("#bevy".to_owned()),
                        // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                        prevent_default_event_handling: false,
                        // enabled_buttons: bevy::window::EnabledButtons {
                        //     maximize: false,
                        //     ..Default::default()
                        // },
                        ..default()
                    }),
                    ..default()
                })
                .build()
                .add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin),
        )
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Scoreboard { score: 0 })
        .add_event::<HitEvent>()
        .add_systems(Startup, setup)
        .add_systems(PostStartup, create_new_target)
        .add_systems(
            FixedUpdate,
            (
                gamepad_system,
                check_the_hit,
                player_movement.before(check_the_hit),
                play_hit_sound.after(check_the_hit),
                gamepad_rumble_on_hit.after(check_the_hit),
                spawn_new_target.after(check_the_hit),
                update_score.after(check_the_hit),
            ),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, scoreboard: Res<Scoreboard>) {
    commands.spawn(Camera2dBundle::default());

    let collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(HitSound(collision_sound));

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: PLAYER_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: Color::GREEN,
                ..default()
            },
            ..default()
        },
        Player,
    ));
    let box_size = Vec3::new(WIDTH, TEXT_BAR_HEIGHT, 0.0);
    commands
        .spawn(NodeBundle {
            style: Style {
                padding: UiRect {
                    left: Val::Px(10.0),
                    right: Val::Px(10.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                },
                width: Val::Px(box_size.x),
                height: Val::Px(box_size.y),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },

            background_color: BackgroundColor(Color::rgb(0.20, 0.3, 0.70)),
            ..default()
        })
        .with_children(|builder| {
            builder.spawn(TextBundle {
                text: Text {
                    sections: vec![TextSection::new(
                        "Controls: WSAD, Arrows. Press Q for exit. ",
                        TextStyle {
                            font_size: 22.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    )],
                    alignment: TextAlignment::Left,
                    linebreak_behavior: BreakLineOn::AnyCharacter,
                },

                ..default()
            });

            builder.spawn((
                TextBundle {
                    text: Text {
                        sections: vec![
                            TextSection::new(
                                "Score: ",
                                TextStyle {
                                    font_size: 22.0,
                                    color: Color::VIOLET,

                                    ..default()
                                },
                            ),
                            TextSection::new(
                                scoreboard.score.to_string(),
                                TextStyle {
                                    font_size: 22.0,
                                    color: Color::VIOLET,

                                    ..default()
                                },
                            ),
                        ],
                        alignment: TextAlignment::Right,
                        linebreak_behavior: BreakLineOn::AnyCharacter,
                    },

                    ..default()
                },
                ScoreText,
            ));
        });
}

fn update_score(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text, With<ScoreText>>) {
    let score = scoreboard.score;

    let mut score_text = query.single_mut();

    score_text.sections[1].value = format!("{score}");
}

fn create_new_target(
    mut commands: Commands,
    player_position: Query<(&Transform, &Sprite), With<Player>>,
) {
    println!("create_new_target");
    let mut rng = rand::thread_rng();

    let (player_transform, player_sprite) = player_position.single();
    let player_x = player_transform.translation.x;
    let player_y = player_transform.translation.y;

    println!("player_x {}", player_x);
    println!("player_y {}", player_y);

    let final_x: f32;
    let final_y: f32;

    loop {
        let range_x = (-WIDTH / 2.0 + TARGET_SIZE.x / 2.0)..=(WIDTH / 2.0 - TARGET_SIZE.x / 2.0);
        let range_y = (-HEIGHT / 2.0 + TEXT_BAR_HEIGHT + TARGET_SIZE.y / 2.0)
            ..=(HEIGHT / 2.0 - TARGET_SIZE.y / 2.0);

        let target_x = rng.gen_range(range_x);
        let target_y = rng.gen_range(range_y);
        if target_x + TARGET_SIZE.x / 2.0 < player_x - PLAYER_SIZE.x / 2.0
            || target_y + TARGET_SIZE.y / 2.0 < player_y - PLAYER_SIZE.y / 2.0
        {
            final_x = target_x;
            final_y = target_y;
            break;
        }
        continue;
    }

    let color: Color;

    loop {
        let new_color = COLORS.choose(&mut rng).unwrap().clone();
        if new_color != player_sprite.color {
            color = new_color;
            break;
        }
        continue;
    }

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(final_x, final_y, 0.0),
                scale: TARGET_SIZE,
                ..default()
            },
            sprite: Sprite { color, ..default() },
            ..default()
        },
        Target,
        Collider,
    ));
}

fn player_movement(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut player_position: Query<&mut Transform, With<Player>>,
    mut event_exit: EventWriter<AppExit>,
) {
    for mut player_transform in &mut player_position {
        let mut direction = Vec3::ZERO;
        if input.any_pressed([KeyCode::Left, KeyCode::A]) {
            direction.x -= 1.0;
        }
        if input.any_pressed([KeyCode::Right, KeyCode::D]) {
            direction.x += 1.0;
        }
        if input.any_pressed([KeyCode::Up, KeyCode::W]) {
            direction.y += 1.0;
        }
        if input.any_pressed([KeyCode::Down, KeyCode::S]) {
            direction.y -= 1.0;
        }

        if input.pressed(KeyCode::Q) {
            exit_system(&mut event_exit);
        }

        if direction != Vec3::ZERO {
            let new_sprite_position =
                player_transform.translation + direction.normalize() * SPEED * time.delta_seconds();

            let bottom_left_bound = Vec3::new(
                -WIDTH / 2.0 + PLAYER_SIZE.x / 2.0,
                -HEIGHT / 2.0 + TEXT_BAR_HEIGHT + PLAYER_SIZE.y / 2.0,
                0.0,
            );
            let top_right_bound = Vec3::new(
                WIDTH / 2.0 - PLAYER_SIZE.x / 2.0,
                HEIGHT / 2.0 - PLAYER_SIZE.y / 2.0,
                0.0,
            );
            player_transform.translation =
                new_sprite_position.clamp(bottom_left_bound, top_right_bound);
        }
    }
}

fn check_the_hit(
    mut commands: Commands,
    mut scroreboard: ResMut<Scoreboard>,
    mut player_query: Query<(&Transform, &mut Sprite), With<Player>>,
    collider_query: Query<(Entity, &Transform, Option<&Target>), With<Collider>>,
    mut hit_events: EventWriter<HitEvent>,
) {
    let (player_transform, mut player_sprite) = player_query.single_mut();
    let player_size = player_transform.scale.truncate();

    for (collider_entity, transform, maybe_target) in &collider_query {
        let collision = collide(
            player_transform.translation,
            player_size,
            transform.translation,
            transform.scale.truncate(),
        );

        if let Some(collision) = collision {
            println!("{:#?}", collision);

            hit_events.send_default();

            if maybe_target.is_some() {
                commands.entity(collider_entity).despawn();

                let mut rng = thread_rng();

                player_sprite.color = COLORS.choose(&mut rng).unwrap().clone();

                scroreboard.score += 1;
            }
        }
    }
}

fn play_hit_sound(
    mut commands: Commands,
    collision_events: EventReader<HitEvent>,
    sound: Res<HitSound>,
) {
    if !collision_events.is_empty() {
        commands.spawn(AudioBundle {
            source: sound.0.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    }
}

fn spawn_new_target(
    commands: Commands,
    mut collision_events: EventReader<HitEvent>,
    player_position: Query<(&Transform, &Sprite), With<Player>>,
) {
    if !collision_events.is_empty() {
        println!("spawn_new_target");
        collision_events.clear();
        create_new_target(commands, player_position);
    }
}

fn exit_system(exit: &mut EventWriter<AppExit>) {
    exit.send(AppExit);
}

fn gamepad_system(
    time: Res<Time>,
    gamepads: Res<Gamepads>,
    axes: Res<Axis<GamepadAxis>>,
    mut player_position: Query<&mut Transform, With<Player>>,
) {
    let mut player_transform = player_position.single_mut();

    for gamepad in gamepads.iter() {
        let axis_lx = GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::LeftStickX,
        };
        let axis_ly = GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::LeftStickY,
        };

        if let (Some(x), Some(y)) = (axes.get(axis_lx), axes.get(axis_ly)) {
            // combine X and Y into one vector
            let left_stick_pos = Vec2::new(x, y);

            if left_stick_pos.length() > 0.5 {
                let new_sprite_position = player_transform.translation
                    + left_stick_pos.extend(0.0).normalize() * SPEED * time.delta_seconds();
                let bottom_left_bound = Vec3::new(
                    -WIDTH / 2.0 + PLAYER_SIZE.x / 2.0,
                    -HEIGHT / 2.0 + TEXT_BAR_HEIGHT + PLAYER_SIZE.y / 2.0,
                    0.0,
                );
                let top_right_bound = Vec3::new(
                    WIDTH / 2.0 - PLAYER_SIZE.x / 2.0,
                    HEIGHT / 2.0 - PLAYER_SIZE.y / 2.0,
                    0.0,
                );
                player_transform.translation =
                    new_sprite_position.clamp(bottom_left_bound, top_right_bound);
            }
        }
    }
}

fn gamepad_rumble_on_hit(
    gamepads: Res<Gamepads>,
    collision_events: EventReader<HitEvent>,
    mut rumble_requests: EventWriter<GamepadRumbleRequest>,
) {
    for gamepad in gamepads.iter() {
        if !collision_events.is_empty() {
            println!("RUMBLE!");
            rumble_requests.send(GamepadRumbleRequest::Add {
                gamepad,
                intensity: GamepadRumbleIntensity::MAX,
                duration: Duration::from_secs_f32(0.20),
            });
        }
    }
}
