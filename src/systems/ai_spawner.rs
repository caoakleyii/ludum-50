use std::collections::HashMap;
use rand::prelude::*;
use bevy::prelude::*;
use bevy_prototype_lyon::{shapes, prelude::{RectangleOrigin, DrawMode, GeometryBuilder, FillMode, StrokeMode}};
use strum::IntoEnumIterator;
use crate::{components::{DirectionName, Direction}, resources::BallChainBotAnimations};
use crate::components::*;

#[derive(Component)]
pub struct AISpawner;

pub fn insert_ai_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>
) {
    let mut animation_map: HashMap<StateKind, HashMap<DirectionName, Handle<TextureAtlas>>>  = HashMap::new();

    for state in StateKind::iter() {
        let mut hash_map = HashMap::new();
        for direction in DirectionName::iter() {
            let path = format!("ball_chain_bot\\{}\\{}.png", direction, state);
            let image_handle: Handle<Image> = asset_server.load(&path);
            let texture_atlas = TextureAtlas::from_grid(
                                                image_handle,
                                                Vec2::new(192.0, 192.0),
                                                 state.ball_chain_bot_frame_data().x as usize,
                                                 state.ball_chain_bot_frame_data().y as usize
                                            );
            let texture_atlas_handle = texture_atlases.add(texture_atlas); 

            hash_map.insert(direction, texture_atlas_handle);
        }
        animation_map.insert(
            state,
            hash_map
        );
    }

    commands.insert_resource(BallChainBotAnimations {
        animation_map
    });

    commands.spawn()
    .insert(Timer::from_seconds(5.0, true))
    .insert(AISpawner);
}

pub fn ball_chain_bot_spawner(
    ball_chain_bot_animations: Res<BallChainBotAnimations>,
    delta_time: Res<Time>,
    mut commands: Commands,
    mut query: Query<&mut Timer, With<AISpawner>>,
    ai_bots: Query<&AI>,
    ground_query: Query<&Ground>
) {
    let mut timer = query.single_mut();
    timer.tick(delta_time.delta());

    if !timer.just_finished() {
        return;
    }

    if !ai_bots.is_empty() {
        return;
    }

    let ground = ground_query.single();
    let mut rng = rand::thread_rng();
    if ground.0 < 1.0 {
        return;
    }

    let x_pos = rng.gen_range(-ground.0/4.0..ground.0/4.0);

    let ai_hitbox = CollisionShape{
        width: 30.0,
        height: 30.0,
        mask: CollisionMasks::AI,
        collides_with: CollisionMasks::Player as i32 | CollisionMasks::PlayerAttack as i32 | CollisionMasks::Ground as i32,
        ..Default::default() 
    };

    let texture_atlas_handle = ball_chain_bot_animations.animation_map.get(&StateKind::Idle).unwrap().get(&DirectionName::Right).unwrap();
    let ai = commands.spawn()
        .insert_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite { color: Color::WHITE, ..Default::default() },
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform::from_xyz(x_pos,-125.0,0.0),
            ..Default::default()
        })
        .insert_bundle(
            (
                AI, 
                BallChainBot,
                Stateful { ..Default::default() },
                Velocity { ..Default::default() },
                Aim { ..Default::default() },
                Direction { angle: 0.0, name: DirectionName::Right, new_direction: false, flip_x: 1.0 },
                Timer::from_seconds(0.1, true),
                ai_hitbox
            )
        )
        .insert_bundle(StatsBundle { 
            max_health: MaxHealth { value: 45.0},
            current_health: CurrentHealth { value: 45.0 },
            speed: Speed { value: 25.0 },
            attack_range: AttackRange { value: 90.0 },
            ..Default::default() 
        })
        .insert(
            MeleeAttack {
                damage: 5.0,
                width: 60.0,
                height: 20.0,
                knockback_force: 45.0,
                offset: Vec3::new(30.0, 0.0, 0.0),
                active_frame_length: 0.2,
                timer: Timer::from_seconds(0.4, true),
                full_attack_timer: Timer::from_seconds(1.2, false),
                mask: CollisionMasks::AIAttack,
                collides_with: CollisionMasks::Player as i32,
                cool_down_timer: Timer::from_seconds(3.0, true),
                ..Default::default()
            }
        )
        .id();

    let health_bar = shapes::Rectangle { 
        origin: RectangleOrigin::TopLeft,
        extents: Vec2::new(20.0, 3.0)
    };

    let health_bar_entity = commands.spawn_bundle(GeometryBuilder::build_as(
        &health_bar,
        DrawMode::Outlined {
            fill_mode: FillMode::color(Color::RED),
            outline_mode: StrokeMode::new(Color::RED, 0.0),
        },
        Transform::from_xyz(-10.0, 16.0, 0.0),
    ))
    .insert(HealthBar)
    .insert(Parent(ai))
    .id();

    let health_bar_outline = shapes::Rectangle { 
        origin: RectangleOrigin::TopLeft,
        extents: Vec2::new(21.0, 4.0)
    };

    commands.spawn_bundle(GeometryBuilder::build_as(
        &health_bar_outline,
        DrawMode::Outlined {
            fill_mode: FillMode::color(Color::rgba(1.0, 1.0, 1.0, 1.0)),
            outline_mode: StrokeMode::new(Color::BLACK, 1.0),
        },
        Transform::from_xyz(-0.5, 0.5, 0.0),
    ))
    .insert(Parent(health_bar_entity));
}