// Simple 3D Scene with a character (sphere).
//     Can be moved around with WASD/arrow keys.
// A health bar and character name is anchored to the character in world-space.
// The health starts at 10 and decreases by 1 every second. The health should be stored and managed
// in Bevy ECS. When reaching 0 HP, the character should be despawned together with UI.

use bevy::prelude::*;
use colorgrad::{self, Gradient};
use haalka::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    position: WindowPosition::Centered(MonitorSelection::Primary),
                    ..default()
                }),
                ..default()
            }),
            HaalkaPlugin,
        ))
        .add_systems(PreStartup, setup)
        .add_systems(Startup, ui_root)
        .add_systems(
            Update,
            (
                wasd,
                sync_tracking_healthbar_position,
                decay,
                sync_health_mutable,
                despawn_when_dead,
            )
                .chain()
                .run_if(any_with_component::<Player>()),
        )
        .insert_resource(StyleDataResource::default())
        .insert_resource(HealthTickTimer(Timer::from_seconds(
            HEALTH_TICK_RATE,
            TimerMode::Repeating,
        )))
        .run();
}

const SPEED: f32 = 10.0;
const RADIUS: f32 = 0.5;
const MINI: (f32, f32) = (200., 15.);
const MAXI: (f32, f32) = (500., 30.);
const NAME: &str = "league_of_legends_enjoyer";
const CAMERA_POSITION: Vec3 = Vec3::new(8., 10.5, 8.);
const PLAYER_POSITION: Vec3 = Vec3::new(0., RADIUS, 0.);
const PLAYER_HEALTH: u32 = 1;
const HEALTH_TICK_RATE: f32 = 1.;

#[derive(Clone, Copy, Default, PartialEq)]
struct StyleData {
    left: f32,
    top: f32,
    scale: f32,
}

#[derive(Resource, Default)]
struct StyleDataResource(Mutable<StyleData>);

#[derive(Component)]
struct Health(u32);

#[derive(Component)]
struct HealthMutable(Mutable<u32>);

fn sync_health_mutable(health_query: Query<(&Health, &HealthMutable), Changed<Health>>) {
    if let Ok((health, health_mutable)) = health_query.get_single() {
        health_mutable.0.set(health.0);
    }
}

#[derive(Component)]
struct Player;

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(50.0).into()),
        material: materials.add(Color::rgb_u8(87, 108, 50).into()),
        ..default()
    });
    commands.spawn((
        Player,
        Health(PLAYER_HEALTH),
        HealthMutable(Mutable::new(PLAYER_HEALTH)),
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: RADIUS,
                ..default()
            })),
            transform: Transform::from_translation(PLAYER_POSITION),
            material: materials.add(Color::rgb_u8(228, 147, 58).into()),
            ..default()
        },
    ));
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0., 8., 0.),
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(CAMERA_POSITION).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn wasd(
    keys: Res<Input<KeyCode>>,
    camera: Query<&Transform, (With<Camera3d>, Without<Player>)>,
    mut player: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let mut direction = Vec3::ZERO;
    let mut player = player.single_mut();
    let camera = camera.single();
    if keys.pressed(KeyCode::W) {
        direction += camera.forward();
    }
    if keys.pressed(KeyCode::A) {
        direction += camera.left();
    }
    if keys.pressed(KeyCode::S) {
        direction += camera.back();
    }
    if keys.pressed(KeyCode::D) {
        direction += camera.right();
    }
    let movement = direction.normalize_or_zero() * SPEED * time.delta_seconds();
    player.translation.x += movement.x;
    player.translation.z += movement.z;
}

fn sync_tracking_healthbar_position(
    style_data_resource: Res<StyleDataResource>,
    player: Query<(&Transform, With<Player>), Changed<Transform>>,
    camera: Query<(&Camera, &Transform), (With<Camera3d>, Without<Player>)>,
    mut ui_scale: ResMut<UiScale>,
) {
    let (camera, camera_transform) = camera.single();
    let player_transform = player.single().0;
    let scale = camera_transform.translation.distance(player_transform.translation);
    if let Some((left, top)) = camera
        .world_to_viewport(&GlobalTransform::from(*camera_transform), player_transform.translation)
        .map(|p| p.into())
    {
        style_data_resource.0.set_neq(StyleData { left, top, scale });
    }
    let starting_distance = CAMERA_POSITION.distance(PLAYER_POSITION);
    // ui_scale.0 = starting_distance as f64 / scale as f64;
}

fn healthbar(
    max: u32,
    health: impl Signal<Item = u32> + Send + Sync + 'static,
    color_gradient: Gradient,
) -> Stack<NodeBundle> {
    let health = health.broadcast();
    let percent_health = health.signal().map(move |h| h as f32 / max as f32).broadcast();
    Stack::<NodeBundle>::new()
        .layer(
            El::<NodeBundle>::new()
                .with_style(|style| {
                    style.height = Val::Percent(100.);
                })
                .on_signal_with_style(percent_health.signal(), move |style, percent_health| {
                    style.width = Val::Percent(percent_health as f32 * 100.)
                })
                .background_color_signal(percent_health.signal().map(move |percent_health| {
                    let [r, g, b, ..] = color_gradient.at(percent_health as f64).to_rgba8();
                    Color::rgb_u8(r, g, b).into()
                })),
        )
        .layer(
            El::<TextBundle>::new()
                .with_style(|style| style.height = Val::Percent(100.).clone())
                .align(Align::new().left())
                .text_signal(health.signal().map(|health| {
                    Text::from_section(
                        health.to_string(),
                        TextStyle {
                            color: Color::WHITE,
                            ..default()
                        },
                    )
                })),
        )
}

fn ui_root(world: &mut World) {
    let style_data = world.resource::<StyleDataResource>().0.clone();
    let health = world.query::<&HealthMutable>().single(&world).0.clone();
    let starting_distance = CAMERA_POSITION.distance(PLAYER_POSITION);
    El::<NodeBundle>::new()
        .with_style(|style| {
            style.width = Val::Percent(100.);
            style.height = Val::Percent(100.);
        })
        .child_signal(health.signal().map(|health| health == 0).dedupe().map_bool(
            || {
                El::<NodeBundle>::new()
                    .align(Align::center())
                    .with_style(|style| {
                        style.width = Val::Px(250.);
                        style.height = Val::Px(80.);
                    })
                    .background_color(Color::BLACK.into())
                    .align_content(Align::center())
                    .child(El::<TextBundle>::new().text(Text::from_section(
                        "respawn",
                        TextStyle {
                            font_size: 60.,
                            color: Color::WHITE,
                            ..default()
                        },
                    )))
            },
            move || {
                Stack::<NodeBundle>::new()
                    .with_style(|style| {
                        style.width = Val::Percent(100.);
                        style.height = Val::Percent(100.);
                        style.padding.bottom = Val::Px(10.);
                    })
                    .layer(
                        Column::<NodeBundle>::new()
                            .with_style(|style| {
                                style.row_gap = Val::Px(MINI.1 / 2.);
                                // style.width = Val::Px(MINI.0);
                                // style.height = Val::Px(MINI.1);
                            })
                            .on_signal_with_style(style_data.signal(), |style, StyleData { left, top, .. }| {
                                style.left = Val::Px(left - MINI.0 / 2.);
                                style.top = Val::Px(top - 30. * 2. - MINI.1);
                                // style.
                                // println!("scale: {}", scale);
                            })
                            // .on_signal_with_transform(style_data.signal(), move |transform, StyleData { scale, .. }|
                            // {     transform.scale = Vec3::splat(starting_distance /
                            // scale); })
                            .item(
                                El::<TextBundle>::new()
                                    .with_style(|style| {
                                        // style.width = Val::Percent(100.);
                                        style.width = Val::Px(MINI.0);
                                    })
                                    .text(
                                        Text::from_section(
                                            NAME,
                                            TextStyle {
                                                font_size: 14.0,
                                                color: Color::WHITE,
                                                ..default()
                                            },
                                        )
                                        .with_alignment(TextAlignment::Center),
                                    ),
                            )
                            .item(
                                healthbar(
                                    PLAYER_HEALTH,
                                    health.signal(),
                                    colorgrad::CustomGradient::new()
                                        .html_colors(&["purple", "yellow"])
                                        .build()
                                        .unwrap(),
                                )
                                .with_style(|style| {
                                    style.width = Val::Px(MINI.0);
                                    style.height = Val::Px(MINI.1);
                                }),
                            ),
                    )
                    .layer(
                        healthbar(
                            PLAYER_HEALTH,
                            health.signal(),
                            colorgrad::CustomGradient::new()
                                .html_colors(&["red", "green"])
                                .build()
                                .unwrap(),
                        )
                        .align(Align::new().bottom().center_x())
                        .with_style(|style| {
                            style.width = Val::Px(MAXI.0);
                            style.height = Val::Px(MAXI.1);
                        }),
                    )
                    .apply(|el| El::<NodeBundle>::new().child(el))
            },
        ))
        .spawn(world);
}

#[derive(Resource)]
struct HealthTickTimer(Timer);

fn decay(mut health: Query<&mut Health>, mut health_tick_timer: ResMut<HealthTickTimer>, time: Res<Time>) {
    if health_tick_timer.0.tick(time.delta()).finished() {
        let mut health = health.single_mut();
        health.0 = health.0.saturating_sub(1);
        health_tick_timer.0.reset();
    }
}

fn despawn_when_dead(mut commands: Commands, query: Query<(Entity, &Health), Changed<Health>>) {
    if let Ok((entity, health)) = query.get_single() {
        if health.0 == 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
