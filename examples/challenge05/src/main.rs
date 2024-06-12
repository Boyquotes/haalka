// A UI on the right with a 3D scene of the character on the left.
//     The character can be simple 3D shapes.
// The UI is composed of multiple buttons to select options.
//     The selected option is highlighted.
//     There are too many buttons to fit in the box, so the box can be scrolled vertically. You can
//         duplicate buttons or choose a small box size to simulate this.
// Changing the selection in the UI changes the 3D shapes in the 3D scene.
// On the top of the UI is a text field for the character name.

use bevy::prelude::*;
use haalka::*;
use strum::{self, IntoEnumIterator};

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
        .add_systems(Startup, (setup, ui_root).chain())
        .run();
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const CLICKED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const BUTTON_WIDTH: Val = Val::Px(250.);
const BUTTON_HEIGHT: Val = Val::Px(65.);

#[derive(Clone, Copy, PartialEq, strum::Display, strum::EnumIter)]
#[strum(serialize_all = "lowercase")]
enum Shape {
    Sphere,
    Plane,
    Cuboid,
    Cylinder,
    Capsule3d,
    Torus,
}

static SELECTED_SHAPE: Lazy<Mutable<Shape>> = Lazy::new(|| Mutable::new(Shape::Cuboid));
static SCROLL_POSITION: Lazy<Mutable<f32>> = Lazy::new(default);

fn button(shape: Shape, hovered: Mutable<bool>) -> impl Element {
    let selected = SELECTED_SHAPE.signal().eq(shape);
    let (pressed, pressed_signal) = Mutable::new_and_signal(false);
    let hovered_signal = hovered.signal();
    let selected_hovered_broadcaster =
        map_ref!(selected, pressed_signal, hovered_signal => (*selected || *pressed_signal, *hovered_signal))
            .broadcast();
    let border_color_signal = {
        selected_hovered_broadcaster
            .signal()
            .map(|(selected, hovered)| {
                if selected {
                    Color::RED
                } else if hovered {
                    Color::WHITE
                } else {
                    Color::BLACK
                }
            })
            .map(BorderColor)
    };
    let background_color_signal = {
        selected_hovered_broadcaster
            .signal()
            .map(|(selected, hovered)| {
                if selected {
                    CLICKED_BUTTON
                } else if hovered {
                    HOVERED_BUTTON
                } else {
                    NORMAL_BUTTON
                }
            })
            .map(BackgroundColor)
    };
    El::<NodeBundle>::new()
        .width(BUTTON_WIDTH)
        .height(BUTTON_HEIGHT)
        .with_style(|style| style.border = UiRect::all(Val::Px(5.)))
        .align_content(Align::center())
        .border_color_signal(border_color_signal)
        .background_color_signal(background_color_signal)
        .hovered_sync(hovered)
        .pressed_sync(pressed)
        .on_click(move || SELECTED_SHAPE.set_neq(shape))
        .child(El::<TextBundle>::new().text(Text::from_section(
            shape.to_string(),
            TextStyle {
                font_size: 40.0,
                color: Color::rgb(0.9, 0.9, 0.9),
                ..default()
            },
        )))
}

fn ui_root(world: &mut World) {
    El::<NodeBundle>::new()
        .width(Val::Percent(100.))
        .height(Val::Percent(100.))
        .align_content(Align::center())
        .child(
            Stack::<NodeBundle>::new()
                .width(Val::Percent(100.))
                .height(Val::Percent(100.))
                .layer(
                    Column::<NodeBundle>::new()
                        .align(Align::new().center_y().right())
                        .with_style(|style| {
                            style.padding.right = Val::Percent(20.);
                            style.row_gap = Val::Px(20.);
                        })
                        .item({
                            let focused = Mutable::new(false);
                            let name = Mutable::new(String::new());
                            let name_shape_syncer = name.signal_cloned().for_each_sync(|name| {
                                if let Some((i, shape)) =
                                    Shape::iter().enumerate().find(|(_, shape)| shape.to_string() == name)
                                {
                                    SELECTED_SHAPE.set_neq(shape);
                                    if let Val::Px(height) = BUTTON_HEIGHT {
                                        SCROLL_POSITION.set_neq(i as f32 * -height);
                                    }
                                }
                            });
                            TextInput::new()
                                .update_raw_el(move |raw_el| raw_el.hold_tasks([spawn(name_shape_syncer)]))
                                .width(BUTTON_WIDTH)
                                .height(Val::Px(40.))
                                .mode(CosmicWrap::InfiniteLine)
                                .scroll_disabled()
                                .cursor_color(CursorColor(Color::WHITE))
                                .fill_color(CosmicBackgroundColor(NORMAL_BUTTON))
                                .attrs(TextAttrs::new().color(Color::WHITE))
                                .placeholder(
                                    PlaceHolder::new()
                                        .text("name")
                                        .attrs(TextAttrs::new().color(Color::GRAY)),
                                )
                                .focus_signal(focused.signal())
                                .focused_sync(focused)
                                .on_change_sync(name)
                        })
                        .item({
                            let hovereds = MutableVec::new_with_values(
                                (0..Shape::iter().count()).map(|_| Mutable::new(false)).collect(),
                            );
                            Column::<NodeBundle>::new()
                                .height(Val::Px(200.))
                                .align(Align::new().center_x())
                                .scrollable_on_hover(ScrollabilitySettings {
                                    flex_direction: FlexDirection::Column,
                                    overflow: Overflow::clip_y(),
                                    scroll_handler: BasicScrollHandler::new()
                                        .direction(ScrollDirection::Vertical)
                                        .pixels(20.)
                                        .into(),
                                })
                                .viewport_y_signal(SCROLL_POSITION.signal())
                                .items({
                                    let hovereds = hovereds.lock_ref().into_iter().cloned().collect::<Vec<_>>();
                                    Shape::iter()
                                        .zip(hovereds)
                                        .map(move |(shape, hovered)| button(shape, hovered))
                                })
                        }),
                ),
        )
        .spawn(world);
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    spawn(SELECTED_SHAPE.signal().for_each(|shape| {
        async_world().apply(move |world: &mut World| {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            *world.query::<&mut Handle<Mesh>>().single_mut(world) = meshes.add(match shape {
                Shape::Sphere => Sphere::default().mesh().ico(5).unwrap(),
                Shape::Plane => Plane3d::default().mesh().size(1., 1.).into(),
                Shape::Cuboid => Cuboid::default().into(),
                Shape::Cylinder => Cylinder::default().into(),
                Shape::Capsule3d => Capsule3d::default().into(),
                Shape::Torus => Torus::default().into(),
            });
        })
    }))
    .detach();
    commands.spawn(PbrBundle {
        material: materials.add(Color::rgb_u8(87, 108, 50)),
        transform: Transform::from_xyz(-1., 0., 1.),
        ..default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1_500_000.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0., 8., 0.),
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(3., 3., 3.).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });
}
