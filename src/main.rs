use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    window::{PrimaryWindow, WindowMode},
};
use bevy_rapier2d::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

#[derive(Resource, Default)]
struct Score(u32);

#[derive(Resource, Default)]
struct SegmentsAreIntersecting(bool);

#[derive(Component)]
pub struct RotationSpeed(f32);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    Playing,
    GameOver,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .insert_state(GameState::Playing)
        .add_systems(
            Startup,
            (setup, create_annulus_segment, create_rotating_line),
        )
        .add_systems(
            Update,
            (
                reverse_rotate_direction,
                rotate_line,
                move_anulus_segment,
                check_for_collision,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, toggle_fullscreen)
        .add_systems(OnEnter(GameState::GameOver), game_over_screen)
        .run();
}

fn check_for_collision(
    mut collision_events: EventReader<CollisionEvent>,
    mut segments_are_intersecting: ResMut<SegmentsAreIntersecting>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity1, entity2, _) => {
                println!("Collision started between {:?} and {:?}", entity1, entity2);
                segments_are_intersecting.0 = true;
            }
            CollisionEvent::Stopped(entity1, entity2, _) => {
                println!("Collision stopped between {:?} and {:?}", entity1, entity2);
                segments_are_intersecting.0 = false;
            }
        }
    }
}

fn rotate_line(time: Res<Time>, mut query: Query<(&RotationSpeed, &mut Transform)>) {
    for (rotation_speed, mut transform) in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_z(-rotation_speed.0 * time.delta_secs());
    }
}

fn game_over_screen(mut commands: Commands, _score: Res<Score>) {
    println!("Game Over");
    commands.spawn((
        Text::new("Game Over"),
        Transform::from_translation(Vec3::new(0., 0., 0.)),
        TextFont {
            font_size: 100.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            top: Val::Percent(50.0),
            left: Val::Percent(50.0),
            ..default()
        },
    ));
    // .with_child((
    //     Text::new(format!("Score: {}", score.0)),
    //     TextFont {
    //         font_size: 100.0,
    //         ..default()
    //     },
    // ));
}

fn reverse_rotate_direction(
    mut query: Query<&mut RotationSpeed>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut score: ResMut<Score>,
    segments_are_intersecting: Res<SegmentsAreIntersecting>,
    mut score_text: Query<&mut Text, With<ScoreText>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        for mut rotation_speed in query.iter_mut() {
            rotation_speed.0 *= -1.;
            if segments_are_intersecting.0 {
                println!("Score Increased");
                score.0 += 1;
                for mut text in score_text.iter_mut() {
                    **text = format!("Score: {}", score.0);
                }
            } else {
                next_state.set(GameState::GameOver);
            }
        }
    }
}

fn move_anulus_segment(
    mut query: Query<&mut Transform, With<TargetZone>>,
    score: Res<Score>,
    mut rotation_speed: Query<&mut RotationSpeed>,
) {
    if !score.is_changed() {
        return;
    };
    let mut rng = rand::thread_rng();
    for mut transform in query.iter_mut() {
        let random_angle = rng.gen_range(0.0..2.0 * PI);
        transform.rotation = Quat::from_rotation_z(random_angle);
    }

    for mut rotation_speed in rotation_speed.iter_mut() {
        if rotation_speed.0 < 10. {
            rotation_speed.0 += 0.5;
        }
    }
}

#[derive(Component)]
struct ScoreText;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let background_circle = Annulus::new(45., 50.);
    let color = Color::BLACK;

    commands.spawn((
        Mesh2d(meshes.add(background_circle)),
        MeshMaterial2d(materials.add(color)),
        Transform {
            translation: Vec3::new(0., 0., 0.),
            scale: Vec3::splat(6.),
            ..default()
        },
    ));

    commands.insert_resource(Score::default());
    commands.insert_resource(SegmentsAreIntersecting::default());

    commands.spawn((Text::new("Score: 0"), ScoreText));
}

fn create_rotating_line(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut line = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    let color = Color::linear_rgba(0., 1., 0., 1.);

    let mut vertices = vec![];
    for i in 0..=1 {
        let angle = if i == 1 { PI / 64. } else { -PI / 64. };
        vertices.push([ops::sin(angle) * 40., ops::cos(angle) * 40., 0.]);
        vertices.push([ops::sin(angle) * 55., ops::cos(angle) * 55., 0.]);
    }
    line.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());

    let indices = vec![0, 2, 1, 2, 3, 1];
    line.insert_indices(Indices::U32(indices.clone()));

    // Convert vertices to Vec<Vec2>
    let vertices_2d: Vec<Vec2> = vertices.iter().map(|v| Vec2::new(v[0], v[1])).collect();
    // Convert indices to Vec<[u32; 3]>
    let indices_3d: Vec<[u32; 3]> = indices
        .chunks(3)
        .map(|chunk| [chunk[0], chunk[1], chunk[2]])
        .collect();

    commands.spawn((
        Mesh2d(meshes.add(line)),
        MeshMaterial2d(materials.add(color)),
        Transform {
            translation: Vec3::new(0., 0., 2.),
            scale: Vec3::splat(6.),
            ..default()
        },
        RotationSpeed(1.),
        Collider::trimesh(vertices_2d, indices_3d),
        Sensor,
        ActiveCollisionTypes::all(),
        ActiveEvents::COLLISION_EVENTS,
    ));
}

#[derive(Component)]
pub struct TargetZone;

fn create_annulus_segment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut segment = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    let color = Color::linear_rgba(1., 0., 0., 1.);
    let resolution = 5;
    let radius_extend: f32 = 25.;

    let start_angle = -radius_extend.to_radians();
    let end_angle = radius_extend.to_radians();
    let angle_increment = (end_angle - start_angle) / resolution as f32;

    let mut vertices = vec![];
    for i in 0..=resolution {
        let angle = start_angle + i as f32 * angle_increment;
        vertices.push([ops::sin(angle) * 43., ops::cos(angle) * 43., 0.]);
        vertices.push([ops::sin(angle) * 52., ops::cos(angle) * 52., 0.]);
    }

    segment.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());

    let mut indices = vec![];
    for i in (0..resolution * 2 - 1).step_by(2) {
        indices.extend_from_slice(&[i, i + 2, i + 1]);
        indices.extend_from_slice(&[i + 1, i + 2, i + 3]);
    }
    // indices.extend_from_slice(&[0, 2, 1]);
    segment.insert_indices(Indices::U32(indices.clone()));

    // Convert vertices to Vec<Vec2>
    let vertices_2d: Vec<Vec2> = vertices.iter().map(|v| Vec2::new(v[0], v[1])).collect();
    // Convert indices to Vec<[u32; 3]>
    let indices_3d: Vec<[u32; 3]> = indices
        .chunks(3)
        .map(|chunk| [chunk[0], chunk[1], chunk[2]])
        .collect();

    commands.spawn((
        Mesh2d(meshes.add(segment)),
        MeshMaterial2d(materials.add(color)),
        Transform {
            translation: Vec3::new(0., 0., 1.),
            scale: Vec3::splat(6.),
            ..default()
        },
        TargetZone,
        Collider::trimesh(vertices_2d, indices_3d),
        Sensor,
        ActiveCollisionTypes::all(),
        ActiveEvents::COLLISION_EVENTS,
    ));
}

fn toggle_fullscreen(
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::F12) {
        for mut window in window.iter_mut() {
            let new_mode = match window.mode {
                WindowMode::Fullscreen(_) => WindowMode::Windowed,
                WindowMode::Windowed => WindowMode::Fullscreen(MonitorSelection::Primary),
                WindowMode::BorderlessFullscreen(_) => WindowMode::Windowed,
                WindowMode::SizedFullscreen(_) => WindowMode::Windowed,
            };
            window.mode = new_mode;
            window.resolution.set(1920., 1080.);
        }
    }
}
