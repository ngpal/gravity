use std::{thread, time::Duration};

use bevy::prelude::*;

const BIG_G: f32 = 10.;
const STAR_MASS: f32 = 100.;
const STAR_RADIUS: f32 = 20.;
const PLANET_MASS: f32 = 10.;
const PLANET_RADIUS: f32 = 10.;
const INITIAL_PLANET_X: f32 = 2.;

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Mass(f32);

#[derive(Component)]
struct Trail(u8);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(Update, (update, manage_trail).chain())
        .run();
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(STAR_RADIUS))),
        MeshMaterial2d(materials.add(Color::BLACK)),
        Mass(STAR_MASS),
        Transform::from_xyz(0., 0., 0.),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(PLANET_RADIUS))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Mass(PLANET_MASS),
        Velocity(Vec3::new(INITIAL_PLANET_X, 0., 0.)),
        Transform::from_xyz(0., 100., 0.),
    ));
}

fn update(
    mut moving: Query<(&Mass, &mut Velocity, &mut Transform)>,
    mut stationary: Query<(&Mass, &Transform), Without<Velocity>>,
) {
    let (_planet_mass, mut planet_vel, mut planet_transform) = moving.single_mut();
    let (static_mass, static_transform) = stationary.single_mut();

    // Find distance
    let planet_pos = planet_transform.translation;
    let static_pos = static_transform.translation;

    let distance_sq = planet_pos.distance_squared(static_pos);

    // Get direction
    let direction = (static_pos - planet_pos).normalize();

    // Get acceleration
    let a = BIG_G * (static_mass.0 / distance_sq);
    let a_vec = direction * a;

    // Update velocity
    planet_vel.0 += a_vec;

    // Update position
    planet_transform.translation += planet_vel.0;

    thread::sleep(Duration::from_millis(10));
}

fn manage_trail(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut trails: Query<(Entity, &mut Transform, &mut Trail), Without<Velocity>>,
    planets: Query<&Transform, With<Velocity>>,
) {
    for &transform in planets.iter() {
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(PLANET_RADIUS))),
            MeshMaterial2d(materials.add(Color::srgba(1., 1., 1., 0.25))),
            Trail(100),
            transform,
        ));
    }

    for (entity, mut transform, mut trail) in trails.iter_mut() {
        trail.0 -= 1;
        transform.scale -= 0.01;

        if trail.0 == 0 {
            commands.entity(entity).despawn();
        }
    }
}
