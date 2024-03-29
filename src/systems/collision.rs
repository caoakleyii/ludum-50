use bevy::core::Time;
use bevy::math::{Vec3, Vec2};
use bevy::prelude::{Query, Transform, Res, Entity, Commands};


use crate::components::CollisionShape;

pub fn process_collisions(
    mut collidables_query: Query<(&mut CollisionShape, &Transform, Entity)>,
){ 
    // For each collidables, compare against other collidables
    let mut combinations = collidables_query.iter_combinations_mut();
    
    while let Some([
        (mut collision_shape, transform, entity),
        (mut other_collision_shape, other_transform, other_entity)]) = combinations.fetch_next() 
    {
        if collision_shape.uuid == other_collision_shape.uuid {
            continue;
        }

        if !can_collide(&collision_shape, &other_collision_shape) {
            continue;
        }

        let is_colliding = collide(
            transform.translation,
            collision_shape.size(),
            other_transform.translation,
            other_collision_shape.size()
        );
        
        match is_colliding {
            false => {
                if collision_shape.collisions.contains(&other_entity) {
                    collision_shape.collisions.remove(&other_entity);
                    collision_shape.collisions_just_ended.insert(other_entity);
                } else {
                    collision_shape.collisions_just_ended.remove(&other_entity);
                }
                if other_collision_shape.collisions.contains(&entity) {
                    other_collision_shape.collisions.remove(&entity);
                    other_collision_shape.collisions_just_ended.insert(entity);
                } else {
                    other_collision_shape.collisions_just_ended.remove(&entity);
                }
            },
            true => {
                collision_shape.collisions.insert(other_entity);
                other_collision_shape.collisions.insert(entity);
            }
        }

    }
}

fn collide(a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> bool {
    let a_min = a_pos.truncate() - a_size / 2.0;
    let a_max = a_pos.truncate() + a_size / 2.0;

    let b_min = b_pos.truncate() - b_size / 2.0;
    let b_max = b_pos.truncate() + b_size / 2.0;
    
    return a_min.x < b_max.x && a_max.x > b_min.x && a_min.y < b_max.y && a_max.y > b_min.y;
}

fn can_collide(shape_a: &CollisionShape, shape_b: &CollisionShape) -> bool {
    return (shape_a.mask as i32 & shape_b.collides_with != 0) &&
        (shape_b.mask as i32 & shape_a.collides_with != 0);
}

pub fn tick_collision_shapes(
    mut commands: Commands,
    delta_time: Res<Time>,
    mut collidables_query: Query<(&mut CollisionShape, Entity)>
) {
    for (mut shape, entity) in collidables_query.iter_mut() {
        shape.timer.tick(delta_time.delta());

        if shape.timer.repeating() {
            continue;
        }

        if shape.timer.finished() {
           commands.entity(entity).remove::<CollisionShape>();
           commands.entity(entity).despawn();
        }
    }
}