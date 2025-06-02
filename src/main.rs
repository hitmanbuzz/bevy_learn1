use avian3d::{math::{Scalar, Vector}, prelude::*};
use bevy::{
    prelude::*,
    text::FontSmoothing, window::{PresentMode, PrimaryWindow},
};
use bevy_dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};


/// It states that the object is `Player` if stated in the field
#[derive(Component)]
struct Player;

/// It states that the object is `MyCamera` if stated in the field
#[derive(Component)]
struct MyCamera;

/// It states that the object is `Ground` if stated in the field
#[derive(Component)]
struct Ground;

/// It states that the object is `Bullet` if stated in the field
#[derive(Component)]
struct Bullet;

/// It states that the object has a filed that supports `shoot`
#[derive(Resource)]
struct ShootState {
    is_shoot: bool,
}

/// It states that the object has a time limit to live
#[derive(Component)]
struct Lifetime {
    timer: Timer,
}

/// The movement speed of the object that is specfied with this structure in the field
#[derive(Component)]
struct MovementAcceleration(Scalar);

/// The field/properties of a wall
struct WallProperties {
    /// `position`: *Vector3 consisting of x,y and z position of the wall*
    position: Vec3,
    /// `size`: *Vector3 consisting of length, width and height of the wall*
    size: Vec3,
}

/// And empty struct that set the color in srgb values
struct OverlayColor;

impl OverlayColor {
    /// Set the color to `Red`
    const RED: Color = Color::srgb(1.0, 0.0, 0.0);
    /// Set the color to `Green`
    const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
}

/// Structure of the Vsync status
struct VsyncStatus;

impl VsyncStatus {
    /// Turn `on` the Vsync
    const ON: PresentMode = PresentMode::Fifo;
    /// Turn `off` the Vsync
    const OFF: PresentMode = PresentMode::Immediate;
}

/// Constant `Vector3` for ground size
const GROUND_SIZE: Vec3 = Vec3::new(10.0, 1.0, 15.0);
/// Constant `Vector3` for camera position
const CAMERA_POS: Vec3 = Vec3::new(0.0, 5.0, 18.0);
/// Constant `f32` for bullet speed
const BULLET_SPEED: f32 = 500.0;
/// Constant `f32` that will despawn the bullet after the given value time
const BULLET_DESPAWN_TIME: f32 = 3.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins, 
            PhysicsPlugins::default(),
            // PhysicsDebugPlugin::default(),
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont { 
                        font_size: 30.0,
                        font: default(),
                        font_smoothing: FontSmoothing::AntiAliased,
                        ..default()
                    },
                    text_color: OverlayColor::GREEN,
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true
                }
            },
        ))
        .insert_resource(ShootState { is_shoot: true })
        .add_systems(Startup, (setup, spawn_walls))
        .add_systems(Update, (
            move_player, 
            shoot_bullet,
            reset_shoot_flag,
            customize_config,
            game_setting,
            display_settings,
            despawn_bullet,
        ))
        .run();
}

/// Objects that are set to be spawn at `Startup`
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn a 3d Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(CAMERA_POS.x, CAMERA_POS.y, CAMERA_POS.z).looking_at(
            Vec3::new(0.0, 2.0, 3.0),
            Vec3::Y,
        ),
        MyCamera,
    ));

    // Spawn a light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 12.0, 4.0),
    ));

    // Spawn a cube (player)
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 1.1, (GROUND_SIZE.z / 2.0) - 0.5),
        MovementAcceleration(12.0),
        LinearDamping(1.0),
        LockedAxes::new()
            .lock_rotation_x()
            .lock_rotation_y()
            .lock_rotation_z(),
        Player,
    ));

    // Spawn a ground where player and other object can stay above it
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(GROUND_SIZE.x, GROUND_SIZE.y, GROUND_SIZE.z),
        Mesh3d(meshes.add(Cuboid::new(GROUND_SIZE.x, GROUND_SIZE.y, GROUND_SIZE.z))),
        MeshMaterial3d(materials.add(Color::srgb_u8(145, 77, 4))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Ground
    ));

    // Spawn a sphere (testing purposes)
    commands.spawn((
        RigidBody::Dynamic,
        Collider::sphere(0.5),
        Mesh3d(meshes.add(Sphere::default())),
        MeshMaterial3d(materials.add(Color::srgb_u8(76, 235, 52))),
        Transform::from_xyz(0.0, 1.1, 1.5),
        LockedAxes::new()
            .lock_translation_y()
            .lock_translation_x()
    ));
}

// First try (it worked)
// Damn, the array of position and size made me almost throw up
/// Spawn 3 side of walls for collission
fn spawn_walls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let wall_spawn_pos: [WallProperties; 3] = [
        WallProperties {
            position: Vec3::new(GROUND_SIZE.x / 2.0 + 0.5, 0.0, 0.0),
            size: Vec3::new(1.0, 6.0, GROUND_SIZE.z),
        },
        WallProperties {
            position: Vec3::new(-GROUND_SIZE.x / 2.0 - 0.5, 0.0, 0.0),
            size: Vec3::new(1.0, 6.0, GROUND_SIZE.z),
        },
        WallProperties {
            position: Vec3::new(0.0, 0.0, -GROUND_SIZE.z / 2.0 - 0.5),
            size: Vec3::new(GROUND_SIZE.x, 6.0, 1.0),
        }
    ];

    for i in 0..3 {
        let spawn_prop = &wall_spawn_pos[i];
        commands.spawn((
            RigidBody::Static,
            Collider::cuboid(spawn_prop.size.x, spawn_prop.size.y, spawn_prop.size.z),
            Mesh3d(meshes.add(Cuboid::new(spawn_prop.size.x, spawn_prop.size.y, spawn_prop.size.z))),
            MeshMaterial3d(materials.add(Color::srgb_u8(189, 159, 26))),
            Transform::from_xyz(spawn_prop.position.x, spawn_prop.position.y, spawn_prop.position.z),
        ));
    }
}

/// Move the player with the given keybinds (Only support left and right movement)
/// 
/// `A` & `D` are the keybinds to move left and right respectively
fn move_player(
    key_input: Res<ButtonInput<KeyCode>>,
    timer: Res<Time>,
    mut query: Query<(&mut LinearVelocity, &MovementAcceleration), With<Player>>,
) {
    let delta_time = timer.delta_secs();

    for (mut linear_velocity, movement_acceleration) in &mut query {
        let left = key_input.pressed(KeyCode::KeyA);
        let right = key_input.pressed(KeyCode::KeyD);

        let horizontal = right as i8 - left as i8;

        let direction = Vector::new(horizontal as Scalar, 0.0, 0.0)
            .normalize_or_zero();

        if direction != Vector::ZERO {
            linear_velocity.x += direction.x * movement_acceleration.0 * delta_time;
        }
    }
}

/// The function that shoot bullets from the middle part of the player (cube)
/// 
/// It will shoot the bullet when the `spacebar` key is pressed
fn shoot_bullet(
    key_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    timer: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut shoot_state: ResMut<ShootState>,
) {
    if shoot_state.is_shoot && key_input.just_pressed(KeyCode::Space) {
        let delta_time = timer.delta_secs();
        let player_transform = player_query.single().unwrap();

        // Spawn the bullet
        commands.spawn((
            Bullet,
            Lifetime {
                timer: Timer::from_seconds(BULLET_DESPAWN_TIME, TimerMode::Once),
            },
            RigidBody::Kinematic,
            Collider::cuboid(0.2, 0.2, 0.2),
            Mesh3d(meshes.add(Cuboid::new(0.2, 0.2, 0.2))),
            MeshMaterial3d(materials.add(Color::srgb_u8(66, 245, 81))),
            Transform::from_xyz(
                player_transform.translation.x, 
                player_transform.translation.y, 
                player_transform.translation.z - 1.0, // Adjust spawn position
            ),
            LinearDamping(1.0),
            LinearVelocity(Vec3::new(0.0, 0.0, -BULLET_SPEED * 3.0 * delta_time)),
        ));

        // Set is_shoot to false
        shoot_state.is_shoot = false;
    }
}

/// Despawn/Remove each bullet that has been shooted after 3 seconds
fn despawn_bullet(
    mut commands: Commands,
    timer: Res<Time>,
    query: Query<(Entity, &mut Lifetime), With<Bullet>>
) {
    for (entity, mut lifetime) in query {
        lifetime.timer.tick(timer.delta());
        
        // If the timer reached 3 seconds, it will despawn that specific bullet
        if lifetime.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Reset the shoot status flag to true
fn reset_shoot_flag(
    mut shoot_state: ResMut<ShootState>,
) {
    // Reset the shoot flag
    shoot_state.is_shoot = true;
}

/// It will not be used due to the specific game work
/// 
/// But the function still works perfectly
#[allow(dead_code)]
fn update_camera_with_player(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<MyCamera>, Without<Player>)>,
) {
    if let Ok(player_transform) = player_query.single() {
        if let Ok(mut camera_transform) = camera_query.single_mut() {
            camera_transform.translation = player_transform.translation + CAMERA_POS;
            camera_transform.look_at(player_transform.translation, Vec3::Y);
        }
    }
}

/// Show the Vsync status ON|OFF in the console (terminal)
fn display_settings(
    windows: Query<&mut Window, With<PrimaryWindow>>
) {
    let window = windows.single().unwrap();

    let vsync_value: String = (|| {
        if window.present_mode == VsyncStatus::ON {
            return String::from("ON");
        } else {
            return String::from("OFF");
        }
    })();
    
    println!("Vsync Status: {}", vsync_value);
}

/// Change the Vsync setting with the given keybinds
fn game_setting(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    input: Res<ButtonInput<KeyCode>>
) {
    let mut window = windows.single_mut().unwrap();

    if input.pressed(KeyCode::ControlLeft) && input.just_pressed(KeyCode::AltLeft) {
        if window.present_mode == VsyncStatus::ON {
            window.present_mode = VsyncStatus::OFF;
        } else {
            window.present_mode = VsyncStatus::ON;
        }
    }
}

/// Change the fps counter color with the given keybinds
fn customize_config(
    input: Res<ButtonInput<KeyCode>>, 
    mut overlay: ResMut<FpsOverlayConfig>
) {
    if input.just_pressed(KeyCode::Digit1) {
        if overlay.text_color == OverlayColor::GREEN {
            overlay.text_color = OverlayColor::RED;
        } else {
            overlay.text_color = OverlayColor::GREEN;
        }
    }
    if input.just_pressed(KeyCode::Digit2) {
        overlay.text_config.font_size -= 2.0;
    }
    if input.just_pressed(KeyCode::Digit3) {
        overlay.text_config.font_size += 2.0;
    }
    if input.just_pressed(KeyCode::Digit4) {
        overlay.enabled = !overlay.enabled;
    }
}
