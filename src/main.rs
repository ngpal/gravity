use std::{thread, time::Duration};

use bevy::{
    core::FrameCount, input::gestures::PinchGesture, prelude::*, utils::hashbrown::HashMap,
};

const BIG_G: f32 = 50.;

const STAR_MASS: f32 = 100.;
const STAR_RADIUS: f32 = 10.;

const PLANET_MASS: f32 = 1.;
const PLANET_RADIUS: f32 = 5.;
const INITIAL_PLANET_X: f32 = 5.6;

const TRAIL_LENGTH: f32 = 30.;

const TRAIL_FREQ: u32 = 2;

#[derive(Component)]
struct Planet(f32);

#[derive(Component)]
struct Star;

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Mass(f32);

#[derive(Component)]
struct Trail(f32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                update_bodies,
                recenter_camera,
                (create_trail, clean_trail).run_if(skip_frames),
                absorbtion,
                zoom,
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
        Star,
        Velocity(Vec3::ZERO),
        Mass(STAR_MASS),
        Transform::from_xyz(0., 0., 0.),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(PLANET_RADIUS))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Planet(PLANET_RADIUS),
        Velocity(Vec3::new(INITIAL_PLANET_X, 0., 0.)),
        Mass(PLANET_MASS),
        Transform::from_xyz(0., 75., 0.),
    ));

    let modifier = 1.;
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(PLANET_RADIUS * modifier))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Planet(PLANET_RADIUS * modifier),
        Velocity(Vec3::new(INITIAL_PLANET_X, 0., 0.)),
        Mass(PLANET_MASS * modifier),
        Transform::from_xyz(0., 150., 0.),
    ));

    let modifier = 1.5;
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(PLANET_RADIUS * modifier))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Planet(PLANET_RADIUS * modifier),
        Velocity(Vec3::new(INITIAL_PLANET_X, 0., 0.)),
        Mass(PLANET_MASS * modifier),
        Transform::from_xyz(0., 200., 0.),
    ));
}

fn update_bodies(mut bodies: Query<(Entity, &Mass, &mut Velocity, &mut Transform)>) {
    thread::sleep(Duration::from_millis(10));

    let mut acc = HashMap::new();
    for (a_e, a_mass, _, a_transform) in bodies.iter() {
        acc.entry(a_e).or_insert(Vec3::new(0., 0., 0.));
        for (b_e, _, _, b_transform) in bodies.iter() {
            if a_e == b_e {
                continue;
            }

            let distance_sq = a_transform
                .translation
                .distance_squared(b_transform.translation);

            let direction = (b_transform.translation - a_transform.translation).normalize();
            let a = -direction * (BIG_G * (a_mass.0 / distance_sq));

            let cur = match acc.get(&b_e) {
                Some(a) => a,
                None => &Vec3::new(0., 0., 0.),
            };

            acc.insert(b_e, cur + a);
        }
    }

    // Update
    for (e, _, mut vel, mut transform) in bodies.iter_mut() {
        vel.0 += acc.get(&e).unwrap();
        transform.translation += vel.0;
    }
}

fn recenter_camera(
    mut camera: Query<&mut Transform, With<Camera2d>>,
    star: Query<&Transform, (With<Star>, Without<Camera2d>)>,
) {
    let star_translate = star.single().translation;
    let mut camera = camera.single_mut();
    camera.translation = star_translate
}

fn skip_frames(time: Res<FrameCount>) -> bool {
    time.0 % TRAIL_FREQ == 0
}

fn create_trail(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    planets: Query<(&Transform, &Planet), Without<Trail>>,
) {
    for (&transform, &ref planet) in planets.iter() {
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(planet.0))),
            MeshMaterial2d(materials.add(Color::srgba(1., 1., 1., 0.25))),
            Trail(TRAIL_LENGTH),
            transform,
        ));
    }
}

fn clean_trail(
    par_commands: ParallelCommands,
    mut trails: Query<(Entity, &mut Transform, &mut Trail)>,
) {
    trails
        .par_iter_mut()
        .for_each(|(entity, mut transform, mut trail)| {
            trail.0 -= 1.;
            transform.scale -= 1. / TRAIL_LENGTH;

            if trail.0 == 0. {
                par_commands.command_scope(|mut commands| {
                    commands.entity(entity).despawn();
                })
            }
        })
}

fn absorbtion(
    mut commands: Commands,
    mut star: Query<(&Transform, &mut Mass), (With<Star>, Without<Planet>)>,
    planets: Query<(Entity, &Transform, &Mass), With<Planet>>,
) {
    let (star_transform, mut star_mass) = star.single_mut();
    for (entity, transform, mass) in planets.iter() {
        if transform.translation.distance(star_transform.translation)
            < STAR_RADIUS + PLANET_RADIUS + 2.
        {
            commands.entity(entity).despawn();
            star_mass.0 += mass.0;
        }
    }
}

fn zoom(
    mut evr_gesture_pinch: EventReader<PinchGesture>,
    mut query_camera: Query<&mut OrthographicProjection, With<Camera2d>>,
) {
    for ev_pinch in evr_gesture_pinch.read() {
        let mut projection = query_camera.single_mut();
        projection.scale += ev_pinch.0 * -0.8;

        if projection.scale < 0.1 {
            projection.scale = 0.1
        } else if projection.scale > 10.0 {
            projection.scale = 10.0
        }
    }
}
