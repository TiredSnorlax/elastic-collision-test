mod Body;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::vec2,
    prelude::*,
    sprite::MaterialMesh2dBundle,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use Body::{body_cursor, generate_bodies, interact_bodies, move_bodies, VelArrow};

#[derive(Resource)]
pub struct Dimensions(usize, usize);

#[derive(Resource)]
pub struct ClickedPos(Vec2);

#[derive(Component)]
pub struct MainCamera;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2dBundle::default(), MainCamera));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::RegularPolygon::new(10., 3).into()).into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        },
        VelArrow { pos: Vec2::ZERO },
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: 1000.,
                height: 600.,
                title: "Elastic Collisions".to_string(),
                ..default()
            },
            ..default()
        }))
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(WorldInspectorPlugin)
        .insert_resource(Dimensions(1000, 600))
        .insert_resource(ClickedPos(vec2(0., 0.)))
        .add_startup_system(setup)
        .add_startup_system(generate_bodies)
        .add_system(move_bodies)
        .add_system(interact_bodies.before(move_bodies))
        .add_system(body_cursor)
        .run()
}
