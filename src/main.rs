use bevy::{prelude::*, sprite::collide_aabb::collide};

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Speed(f32);

#[derive(Component)]
enum Collider {
    Static,
    Bounce,
}

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(spawn_the_things)
        .add_system(paddle_system.label("A"))
        .add_system(velocity_system.label("A"))
        .add_system(collision_system.before("A"))
        .run();
}

fn spawn_the_things(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 50.0, 0.0),
                scale: Vec3::new(30.0, 30.0, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Velocity(Vec2::new(1.0, 1.0)))
        .insert(Collider::Bounce)
        .insert(Ball);

    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::new(120.0, 30.0, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Speed(3.0))
        .insert(Velocity(Vec2::ZERO))
        .insert(Paddle)
        .insert(Collider::Static);

    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(300.0, 0.0, 0.0),
                scale: Vec3::new(30.0, 1000.0, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Velocity(Vec2::ZERO))
        .insert(Collider::Static);

    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(-300.0, 0.0, 0.0),
                scale: Vec3::new(30.0, 1000.0, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Velocity(Vec2::ZERO))
        .insert(Collider::Static);

    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, -300.0, 0.0),
                scale: Vec3::new(1000.0, 30.0, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Velocity(Vec2::ZERO))
        .insert(Collider::Static);

    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 300.0, 0.0),
                scale: Vec3::new(1000.0, 30.0, 0.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.5, 0.5, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Velocity(Vec2::ZERO))
        .insert(Collider::Static);
}

fn paddle_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Speed, &mut Velocity), With<Paddle>>,
) {
    if let Ok((speed, mut velocity)) = query.get_single_mut() {
        let direction =
            keyboard_input.pressed(KeyCode::D) as i8 - keyboard_input.pressed(KeyCode::A) as i8;
        velocity.0.x = speed.0 * direction as f32;
    }
}

fn velocity_system(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.0);
    }
}

fn collision_system(
    mut sources: Query<(Entity, &mut Velocity, &Transform, &Collider)>,
    targets: Query<(Entity, &Transform), With<Collider>>,
) {
    for (source_entity, mut source_velocity, source_transform, source_collider) in
        sources.iter_mut()
    {
        for (target_entity, target_transform) in targets.iter() {
            if source_entity == target_entity {
                continue;
            }
            if let Some(collision) = resolve_collision(
                (
                    source_transform.scale.truncate() / 2.0,
                    source_transform.translation.truncate() + source_velocity.0,
                ),
                (
                    target_transform.scale.truncate() / 2.0,
                    target_transform.translation.truncate(),
                ),
            ) {
                println!("COLLISION, {:?}", collision);
                match source_collider {
                    Collider::Static => source_velocity.0 = Vec2::ZERO,
                    Collider::Bounce => {
                        let mut flip_by = source_velocity.0.signum();
                        match collision.normal.x {
                            1.0 => flip_by.x *= 1.0,
                            -1.0 => flip_by.x *= -1.0,
                            _ => {}
                        }
                        match collision.normal.y {
                            1.0 => flip_by.y *= 1.0,
                            -1.0 => flip_by.y *= -1.0,
                            _ => {}
                        }
                        source_velocity.0 = source_velocity.0.abs() * flip_by;
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct Collision {
    normal: Vec2,
    overlap: f32,
    position: Vec2,
}

fn resolve_collision(
    (aabb_extents, aabb_position): (Vec2, Vec2),
    (aabb2_extents, aabb2_position): (Vec2, Vec2),
) -> Option<Collision> {
    let center_difference = aabb2_position - aabb_position;

    let extents_sum = aabb_extents + aabb2_extents;

    let overlap = extents_sum - center_difference.abs();

    if overlap.min_element() <= 0.0 {
        return None;
    }

    let direction = center_difference.signum();

    if overlap.x < overlap.y {
        Some(Collision {
            normal: Vec2::X * direction.x,
            overlap: overlap.x,
            position: Vec2::new(
                aabb_position.x + aabb_extents.x * direction.x,
                aabb2_position.y,
            ),
        })
    } else {
        Some(Collision {
            normal: Vec2::Y * direction.y,
            overlap: overlap.y,
            position: Vec2::new(
                aabb2_position.x,
                aabb_position.y + aabb_extents.y * direction.y,
            ),
        })
    }
}
