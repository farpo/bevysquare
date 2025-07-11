#![windows_subsystem = "windows"]
use std::ops::Range;
const SQUARE_SIZE: f32 = 50.;

use bevy::{color::palettes::css::GREEN, prelude::*, window::WindowResolution};
#[derive(Component)]
struct GameMember;
#[derive(Component)]
struct Player;
#[derive(Component)]
struct Enemy;
#[derive(Component)]
struct Velocity(Vec2);
#[derive(Component)]
struct Food;
#[derive(Resource)]
struct ScoreCounter(usize);
#[derive(Event, Default)]
struct EatFoodEvent;
#[derive(Component)]
struct EnterGameButton;
#[derive(Component)]
struct ScoreText;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Square Game".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            })
        )
        .init_state::<AppState>()
        .add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(AppState::Menu), setup_menu)
        .add_systems(Update, menu.run_if(in_state(AppState::Menu)))
        .add_systems(OnExit(AppState::Menu), cleanup_menu)
        .add_systems(OnEnter(AppState::InGame), (spawn_player, spawn_food))
        .add_systems(
            FixedUpdate,
            (
                move_player,
                move_enemies,
                check_enemies,
                check_food,
                spawn_enemy,
                update_score,
            )
                .chain()
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(OnExit(AppState::InGame), cleanup_game)
        .add_event::<EatFoodEvent>()
        .insert_resource(ScoreCounter(0))
        .run();
}
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    InGame,
}
fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
fn setup_menu(mut commands: Commands) {
    commands.spawn((
        EnterGameButton,
        Node {
            // center button
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![(
            Button,
            Node {
                width: Val::Px(150.),
                height: Val::Px(65.),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::WHITE),
            children![(
                Text::new("Play"),
                TextFont {
                    font_size: 33.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            )],
        )],
    ));
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
fn menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                next_state.set(AppState::InGame);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, button: Query<Entity, With<EnterGameButton>>) {
    let entity = button.single().unwrap();
    commands.entity(entity).despawn();
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let square_mesh = meshes.add(Rectangle::new(SQUARE_SIZE, SQUARE_SIZE));
    let blue = materials.add(Color::linear_rgb(0., 0., 1.));

    commands.spawn((
        GameMember,
        Player,
        Mesh2d(square_mesh),
        MeshMaterial2d(blue),
        Transform::from_xyz(0., 0., 0.),
    ));
    commands
        .spawn((
            // Create a Text with multiple child spans.
            Text::new("Score: "),
            TextFont {
                // This font is loaded and will be used instead of the default font.
                font_size: 42.0,
                ..default()
            },
        ))
        .with_child((
            TextSpan::default(),
            (
                TextFont {
                    font_size: 33.0,
                    // If no font is specified, the default font (a minimal subset of FiraMono) will be used.
                    ..default()
                },
                TextColor(GREEN.into()),
            ),
            ScoreText,
        ));
}
fn spawn_food(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Query<&Window>,
) {
    let square_mesh = meshes.add(Rectangle::new(SQUARE_SIZE, SQUARE_SIZE));
    let green = materials.add(Color::linear_rgb(0., 1., 0.));
    let res = &windows.single().unwrap().resolution;
    let transform = make_random_position(width_range(res), heigth_range(res));
    commands.spawn((
        GameMember,
        Food,
        Mesh2d(square_mesh),
        MeshMaterial2d(green),
        Transform::from_translation(transform),
    ));
}
fn move_player(
    mut player: Query<&mut Transform, With<Player>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let mut player = player.single_mut().unwrap();
    let (camera, camera_transform) = camera_q.single().unwrap();
    if let Some(pos) = windows
        .single()
        .unwrap()
        .cursor_position()
        .and_then(|pos| camera.viewport_to_world_2d(camera_transform, pos).ok())
    {
        player.translation.x = pos.x;
        player.translation.y = pos.y;
    }
}
fn check_food(
    player: Query<&Transform, With<Player>>,
    mut food: Query<&mut Transform, (With<Food>, Without<Player>)>,
    windows: Query<&Window>,
    mut counter: ResMut<ScoreCounter>,
    mut events: EventWriter<EatFoodEvent>,
) {
    let player = player.single().unwrap();
    let mut food = food.single_mut().unwrap();
    let rx = player.translation.x - SQUARE_SIZE..player.translation.x + SQUARE_SIZE;
    let ry = player.translation.y - SQUARE_SIZE..player.translation.y + SQUARE_SIZE;
    if rx.contains(&food.translation.x) && ry.contains(&food.translation.y) {
        let res = &windows.single().unwrap().resolution;
        food.translation = make_random_position(width_range(res), heigth_range(res));
        counter.0 += 1;
        events.write_default();
    }
}
fn check_enemies(
    player: Query<&Transform, With<Player>>,
    food: Query<&Transform, (With<Enemy>, Without<Player>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let player = player.single().unwrap();
    let rx = player.translation.x - SQUARE_SIZE..player.translation.x + SQUARE_SIZE;
    let ry = player.translation.y - SQUARE_SIZE..player.translation.y + SQUARE_SIZE;
    for Transform { translation, .. } in food.iter() {
        if rx.contains(&translation.x) && ry.contains(&translation.y) {
            next_state.set(AppState::Menu);
        }
    }
}
fn spawn_enemy(
    mut commands: Commands,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut events: EventReader<EatFoodEvent>,
) {
    if events.is_empty() {
        return;
    }
    events.clear();
    println!("Enemy spawned");
    let square_mesh = meshes.add(Rectangle::new(SQUARE_SIZE, SQUARE_SIZE));
    let red = materials.add(Color::linear_rgb(1., 0., 0.));
    let res = &windows.single().unwrap().resolution;
    let transform = make_random_position(
        -(res.width() / 2.)..res.width() / 2.,
        -(res.height() / 2.)..res.height() / 2.,
    );
    let velocity = make_random_velocity();
    commands.spawn((
        GameMember,
        Enemy,
        Transform::from_translation(transform),
        Velocity(velocity),
        Mesh2d(square_mesh),
        MeshMaterial2d(red),
    ));
}
fn move_enemies(
    mut enemies: Query<(&mut Transform, &mut Velocity), With<Enemy>>,
    windows: Query<&Window>,
) {
    let res = &windows.single().unwrap().resolution;
    for (mut transform, mut velocity) in enemies.iter_mut() {
        apply_velocity(&mut transform.translation, velocity.0);
        let rx = width_range(res);
        let ry = heigth_range(res);
        let velocity = &mut velocity.0;
        let mut translation = transform.translation;
        if translation.x < rx.start {
            translation.x = rx.start;
            velocity.x *= -1.0;
        } else if translation.x > rx.end {
            translation.x = rx.end;
            velocity.x *= -1.0;
        }
        if translation.y < ry.start {
            translation.y = ry.start;
            velocity.y *= -1.0;
        } else if translation.y > ry.end {
            translation.y = ry.end;
            velocity.y *= -1.0;
        }
        transform.translation = translation;
    }
}
fn update_score(score: Res<ScoreCounter>, mut text: Query<&mut TextSpan, With<ScoreText>>) {
    for mut text in text.iter_mut() {
        **text = format!("{}", score.0)
    }
}
fn cleanup_game(
    mut commands: Commands,
    members: Query<Entity, With<GameMember>>,
    mut score: ResMut<ScoreCounter>,
) {
    for entity in members.iter() {
        commands.entity(entity).despawn();
    }
    score.0 = 0;
}
fn width_range(res: &WindowResolution) -> Range<f32> {
    -(res.width() / 2.)..res.width() / 2.
}
fn heigth_range(res: &WindowResolution) -> Range<f32> {
    -(res.height() / 2.)..res.height() / 2.
}
fn make_random_position(x_range: Range<f32>, y_range: Range<f32>) -> Vec3 {
    let x = rand::random_range(x_range);
    let y = rand::random_range(y_range);
    Vec3::new(x, y, 0.)
}
fn make_random_velocity() -> Vec2 {
    let x = rand::random_range(0.0..1.0) + 1.;
    let y = rand::random_range(0.0..1.0) + 1.;
    Vec2::new(x, y).normalize() * 4.
}
fn apply_velocity(pos: &mut Vec3, velocity: Vec2) {
    pos.x += velocity.x;
    pos.y += velocity.y;
}
