use bevy::{
    math::vec2, prelude::*, render::camera::RenderTarget, sprite::MaterialMesh2dBundle, transform,
};
use rand::Rng;

use crate::{ClickedPos, Dimensions, MainCamera};

#[derive(Component)]
pub struct Body {
    mass: f32,
    radius: f32,
    velocity: Vec2,
    acceleration: Vec2,
}

#[derive(Component)]
pub struct VelArrow {
    pub pos: Vec2,
}

// TODO use another formula. I think this doesn't work that well
// TODO try this https://en.wikipedia.org/wiki/Elastic_collision#Two-dimensional
fn elastic_collision(body1: &Body, body2: &Body, pos1: Vec2, pos2: Vec2) -> Option<(Vec2, Vec2)> {
    let normal_vec = pos2 - pos1;

    let unit_normal_vec = normal_vec / (normal_vec.x.powi(2) + normal_vec.y.powi(2)).sqrt();
    let unit_tangent_vec = vec2(-unit_normal_vec.y, unit_normal_vec.x);

    let vel_1_normal = unit_normal_vec.dot(body1.velocity);
    let vel_1_tangent = unit_tangent_vec.dot(body1.velocity);

    let vel_2_normal = unit_normal_vec.dot(body2.velocity);
    let vel_2_tangent = unit_tangent_vec.dot(body2.velocity);

    let new_vel_1_normal = (vel_1_normal * (body1.mass - body2.mass)
        + 2. * body2.mass * vel_2_normal)
        / (body1.mass + body2.mass);

    let new_vel_2_normal = (vel_2_normal * (body2.mass - body1.mass)
        + 2. * body1.mass * vel_1_normal)
        / (body1.mass + body2.mass);

    let new_normal_vec_1 = unit_normal_vec * new_vel_1_normal;
    let new_normal_vec_2 = unit_normal_vec * new_vel_2_normal;

    let new_tangent_vec_1 = unit_tangent_vec * vel_1_tangent;
    let new_tangent_vec_2 = unit_tangent_vec * vel_2_tangent;

    let final_vec_1 = new_normal_vec_1 + new_tangent_vec_1;
    let final_vec_2 = new_normal_vec_2 + new_tangent_vec_2;

    return Some((final_vec_1, final_vec_2));
}

fn elastic_collision_2(body1: &Body, body2: &Body, pos1: Vec2, pos2: Vec2) -> Option<(Vec2, Vec2)> {
    let rel_pos_1 = pos1 - pos2;
    let rel_pos_2 = pos2 - pos1;

    let pos_length_1 = rel_pos_1.x.powi(2) + rel_pos_1.y.powi(2);
    let pos_length_2 = rel_pos_2.x.powi(2) + rel_pos_2.y.powi(2);

    let v1 = (2. * body2.mass / (body2.mass + body1.mass))
        * (body1.velocity - body2.velocity).dot(rel_pos_1)
        / pos_length_1
        * rel_pos_1;
    let v2 = (2. * body1.mass / (body2.mass + body1.mass))
        * (body2.velocity - body1.velocity).dot(rel_pos_2)
        / pos_length_2
        * rel_pos_2;

    Some((body1.velocity - v1, body2.velocity - v2))
}

pub fn interact_bodies(mut query: Query<(&mut Body, &GlobalTransform, &mut Transform)>) {
    let mut iter = query.iter_combinations_mut();

    while let Some(
        [(mut body1, g_transform1, mut transform1), (mut body2, g_transform2, mut transform2)],
    ) = iter.fetch_next()
    {
        let pos1 = vec2(g_transform1.translation().x, g_transform1.translation().y);
        let pos2 = vec2(g_transform2.translation().x, g_transform2.translation().y);

        let distance = ((pos1.x - pos2.x).powi(2) + (pos1.y - pos2.y).powi(2)).sqrt();

        let res = body1.velocity - body2.velocity;

        if distance * 0.98 < body1.radius + body2.radius {
            if let Some((new_vel_1, new_vel_2)) = elastic_collision_2(&body1, &body2, pos1, pos2) {
                body1.velocity = new_vel_1;
                body2.velocity = new_vel_2;
            };
        }
    }
}

pub fn move_bodies(
    mut query: Query<(&mut Body, &mut Transform)>,
    dim: Res<Dimensions>,
    time: Res<Time>,
) {
    for (mut body, mut transform) in &mut query {
        let new_vel = body.velocity + body.acceleration;
        body.velocity = new_vel;

        let width = dim.0 as f32;
        let height = dim.1 as f32;

        transform.translation.x += body.velocity.x * time.delta_seconds();
        transform.translation.y += body.velocity.y * time.delta_seconds();

        if transform.translation.x - body.radius < -0.5 * width {
            transform.translation.x = -0.5 * width + body.radius;
            body.velocity.x *= -1.;
            body.acceleration.x *= -1.;
        } else if transform.translation.x + body.radius > 0.5 * width {
            transform.translation.x = 0.5 * width - body.radius;
            body.velocity.x *= -1.;
            body.acceleration.x *= -1.;
        };
        if transform.translation.y - body.radius < -0.5 * height {
            transform.translation.y = -0.5 * height + body.radius;
            body.velocity.y *= -1.;
            body.acceleration.y *= -1.
        } else if transform.translation.y + body.radius > 0.5 * height {
            transform.translation.y = 0.5 * height - body.radius;
            body.velocity.y *= -1.;
            body.acceleration.y *= -1.
        }
    }
}

#[derive(Debug)]
struct GenerateData {
    pos: Vec2,
    radius: f32,
}

pub fn generate_bodies(
    dim: Res<Dimensions>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::thread_rng();

    let width = dim.0 as f32 / 2.;
    let height = dim.1 as f32 / 2.;

    let max_vel = 100.;

    let mut gen_data: Vec<GenerateData> = Vec::new();

    for _ in 0..200 {
        let radius = rng.gen_range(5.0..=15.);

        let vx = rng.gen_range(-max_vel..max_vel);
        let vy = rng.gen_range(-max_vel..max_vel);

        let r = rng.gen_range(0.0..=1.);
        let g = rng.gen_range(0.0..=1.);
        let b = rng.gen_range(0.0..=1.);

        let mut ok = false;

        let mut x = rng.gen_range(-width..=width);
        let mut y = rng.gen_range(-height..=height);

        let mut i = 0;

        while !ok {
            x = rng.gen_range(-width..=width);
            y = rng.gen_range(-height..=height);
            let test = gen_data.iter().find(|data| {
                (((*data).pos.x - x).powi(2) + ((*data).pos.y - y).powi(2)).sqrt()
                    <= (*data).radius + radius + 5.
            });

            if test.is_some() {
                println!("{:?}, {}, {}, {}", test.unwrap(), x, y, radius);
            }

            ok = test.is_none();
            i += 1;
        }

        println!("{i}");

        gen_data.push(GenerateData {
            pos: vec2(x, y),
            radius,
        });

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(radius).into()).into(),
                material: materials.add(ColorMaterial::from(Color::rgb(r, g, b))),
                transform: Transform::from_translation(Vec3::new(x, y, 0.)),
                ..default()
            },
            Body {
                mass: radius * radius,
                radius,
                velocity: vec2(vx, vy),
                acceleration: Vec2::ZERO,
            },
        ));
    }
}

pub fn body_cursor(
    // need to get window dimensions
    wnds: Res<Windows>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    buttons: Res<Input<MouseButton>>,
    mut q_arrow: Query<&mut Transform, With<VelArrow>>,
    mut clicked_pos: ResMut<ClickedPos>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // get the window that the camera is displaying to (or the primary window)
    let wnd = if let RenderTarget::Window(id) = camera.target {
        wnds.get(id).unwrap()
    } else {
        wnds.get_primary().unwrap()
    };

    // check if the cursor is inside the window and get its position
    if let Some(screen_pos) = wnd.cursor_position() {
        // get the size of the window
        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        let pos: Vec2 = world_pos.truncate();

        if buttons.just_pressed(MouseButton::Left) {
            let mut arrow_transfrom = q_arrow.single_mut();
            arrow_transfrom.translation.x = pos.x;
            arrow_transfrom.translation.y = pos.y;
            arrow_transfrom.translation.z = 10.;
            clicked_pos.0 = pos;
        }

        if buttons.pressed(MouseButton::Left) {
            let mut arrow_transfrom = q_arrow.single_mut();

            let scale = clicked_pos.0.distance_squared(pos).sqrt() / 10.;

            arrow_transfrom.scale.y = scale;

            let initial_vec = clicked_pos.0 - pos;
            let angle = Vec2::Y.angle_between(initial_vec);

            arrow_transfrom.rotation = Quat::from_rotation_z(angle);
        }

        if buttons.just_released(MouseButton::Left) {
            let vel = clicked_pos.0.distance(pos).abs() / 30.;
            if vel == 0. {
                return;
            };

            let initial_vec = clicked_pos.0 - pos;

            let angle = Vec2::Y.angle_between(initial_vec);

            let radius = 30.;
            let mut rng = rand::thread_rng();
            let r = rng.gen_range(0.0..=1.);
            let g = rng.gen_range(0.0..=1.);
            let b = rng.gen_range(0.0..=1.);

            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(radius).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::rgb(r, g, b))),
                    transform: Transform::from_translation(Vec3::new(
                        clicked_pos.0.x,
                        clicked_pos.0.y,
                        0.,
                    )),
                    ..default()
                },
                Body {
                    mass: radius * radius,
                    radius,
                    velocity: vec2(-vel * angle.sin(), vel * angle.cos()),
                    acceleration: Vec2::ZERO,
                },
            ));
        }
    }
}
