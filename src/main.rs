use bevy::prelude::*;

const BIG_G: f32 = 100.;

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Mass(f32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        .run();
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(20.))),
        MeshMaterial2d(materials.add(Color::BLACK)),
        Mass(25.),
        Transform::from_xyz(0., 0., 0.),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(10.))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Mass(10.),
        Velocity(Vec3::new(5., 0., 0.)),
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
}
