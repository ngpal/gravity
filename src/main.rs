use std::{thread, time::Duration};

use bevy::prelude::*;

const BIG_G: f32 = 50.;
const STAR_MASS: f32 = 100.;
const STAR_RADIUS: f32 = 10.;
const PLANET_MASS: f32 = 1.;
const PLANET_RADIUS: f32 = 5.;
const INITIAL_PLANET_X: f32 = 5.;
const TRAIL_LENGTH: f32 = 100.;

#[derive(Component)]
struct Planet(Vec3);

#[derive(Component)]
struct Mass(f32);

#[derive(Component)]
struct Trail(f32);

#[derive(Component)]
struct Star(Vec3);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                update_bodies,
                recenter_camera,
                manage_trail,
                collision_check,
            )
                .chain(),
        )
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
        Star(Vec3::ZERO),
        Mass(STAR_MASS),
        Transform::from_xyz(0., 0., 0.),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(PLANET_RADIUS))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Mass(PLANET_MASS),
        Planet(Vec3::new(INITIAL_PLANET_X, 0., 0.)),
        Transform::from_xyz(0., 100., 0.),
    ));
}

fn update_bodies(
    mut planets: Query<(&Mass, &mut Planet, &mut Transform)>,
    mut star: Query<(&Mass, &mut Star, &mut Transform), Without<Planet>>,
) {
    let (star_mass, mut star_vel, mut star_transform) = star.single_mut();
    let mut star_a = Vec3::ZERO;

    for (planet_mass, mut planet_vel, mut planet_transform) in planets.iter_mut() {
        // Find distance
        let planet_pos = planet_transform.translation;
        let static_pos = star_transform.translation;

        let distance_sq = planet_pos.distance_squared(static_pos);

        // Get direction
        let direction = (static_pos - planet_pos).normalize();

        // Get acceleration
        let a = BIG_G * (star_mass.0 / distance_sq);
        let a_vec = direction * a;

        star_a += -direction * (BIG_G * (planet_mass.0 / distance_sq));

        // Update velocity
        planet_vel.0 += a_vec;

        // Update position
        planet_transform.translation += planet_vel.0;
    }

    // Update star velocity and position
    star_vel.0 += star_a;
    star_transform.translation += star_vel.0;

    thread::sleep(Duration::from_millis(10));
}

fn recenter_camera(
    mut camera: Query<&mut Transform, With<Camera2d>>,
    star: Query<&Transform, (With<Star>, Without<Camera2d>)>,
) {
    let star_translate = star.single().translation;
    let mut camera = camera.single_mut();
    camera.translation = star_translate
}

fn manage_trail(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut trails: Query<(Entity, &mut Transform, &mut Trail)>,
    planets: Query<&Transform, (With<Planet>, Without<Trail>)>,
) {
    for &transform in planets.iter() {
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(PLANET_RADIUS))),
            MeshMaterial2d(materials.add(Color::srgba(1., 1., 1., 0.25))),
            Trail(TRAIL_LENGTH),
            transform,
        ));
    }

    for (entity, mut transform, mut trail) in trails.iter_mut() {
        trail.0 -= 1.;
        transform.scale -= 1. / TRAIL_LENGTH;

        if trail.0 == 0. {
            commands.entity(entity).despawn();
        }
    }
}

fn collision_check(
    mut commands: Commands,
    planets: Query<(Entity, &Transform), With<Planet>>,
    star: Query<&Transform, (With<Star>, Without<Planet>)>,
) {
    let star_translation = star.single().translation;
    for (entity, transform) in planets.iter() {
        if transform.translation.distance(star_translation) < STAR_RADIUS + PLANET_RADIUS {
            commands.entity(entity).despawn();
        }
    }
}
