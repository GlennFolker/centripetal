use avian2d::prelude::*;
use bevy::{
    math::{vec2, FloatOrd},
    prelude::*,
};
use bevy_mod_picking::prelude::*;

use crate::world::collider::LevelCollider;

pub const VERTEX_RADIUS: f32 = 1.2;
pub const EXTRUDE_RADIUS: f32 = 0.5;
pub const DRAG_COLOR: LinearRgba = LinearRgba::GREEN;
pub const NEW_COLOR: LinearRgba = LinearRgba::rgb(1.0, 1.0, 0.0);

#[derive(Copy, Clone, Component)]
pub struct LevelColliderWidget {
    edit: LevelColliderEdit,
}

#[derive(Copy, Clone)]
pub enum LevelColliderEdit {
    Off,
    Drag(usize, Vec2),
    New(usize, Vec2),
}

pub fn update_collider_widget(
    mut commands: Commands,
    colliders: Query<(Entity, &LevelCollider, Option<&Children>), Changed<LevelCollider>>,
    mut widgets: Query<&mut Collider, With<LevelColliderWidget>>,
) {
    'outer: for (e, collider, children) in &colliders {
        let mut colliders = Vec::with_capacity(collider.vertices.len() * 2);
        for i in 0..collider.vertices.len() {
            let a = collider.vertices[i];
            let b = collider.vertices[(i + 1) % collider.vertices.len()];

            colliders.extend([
                (a, Rotation::degrees(45.0), Collider::rectangle(VERTEX_RADIUS, VERTEX_RADIUS)),
                (a.lerp(b, 0.5), Rotation::IDENTITY, Collider::circle(EXTRUDE_RADIUS)),
            ]);
        }

        let src = Collider::compound(colliders);
        if let Some(children) = children {
            for &child in children {
                let Ok(mut dst) = widgets.get_mut(child) else { continue };
                *dst = src;

                continue 'outer
            }
        }

        let widget = commands
            .spawn((
                LevelColliderWidget {
                    edit: LevelColliderEdit::Off,
                },
                TransformBundle {
                    local: Transform::from_xyz(0.0, 0.0, 1.0),
                    ..default()
                },
                src,
                On::<Pointer<Click>>::run(click_collider_widget),
                On::<Pointer<DragStart>>::run(drag_start_collider_widget),
                On::<Pointer<Drag>>::run(drag_collider_widget),
                On::<Pointer<DragEnd>>::run(drag_end_collider_widget),
            ))
            .id();
        commands.entity(e).add_child(widget);
    }
}

pub fn draw_collider_widget(
    mut gizmos: Gizmos,
    widgets: Query<(&LevelColliderWidget, &Parent)>,
    colliders: Query<(&GlobalTransform, &LevelCollider)>,
) {
    for (&widget, parent) in &widgets {
        let Ok((&trns, collider)) = colliders.get(parent.get()) else {
            continue
        };

        let origin = trns.translation().truncate();
        let len = collider.vertices.len();

        match widget.edit {
            LevelColliderEdit::Off => {}
            LevelColliderEdit::Drag(selected, delta) => {
                let a = origin + collider.vertices[(selected + len - 1) % len];
                let b = origin + collider.vertices[selected] + delta;
                let c = origin + collider.vertices[(selected + 1) % len];

                gizmos.linestrip_2d([a, b, c], DRAG_COLOR);
                gizmos.rect_2d(b, Rot2::degrees(45.0), Vec2::splat(VERTEX_RADIUS), DRAG_COLOR);
            }
            LevelColliderEdit::New(reference, delta) => {
                let a = origin + collider.vertices[reference];
                let b = origin + collider.vertices[(reference + 1) % len];
                let new = a.lerp(b, 0.5) + delta;

                gizmos.linestrip_2d([a, new, b], NEW_COLOR);
                gizmos.circle_2d(new, EXTRUDE_RADIUS, NEW_COLOR);
            }
        }
    }
}

pub fn click_collider_widget(
    event: Listener<Pointer<Click>>,
    widgets: Query<&Parent>,
    mut colliders: Query<(&GlobalTransform, &mut LevelCollider)>,
) {
    if event.button != PointerButton::Secondary {
        return
    };

    let target = event.target();
    let Some(pos) = event.event.hit.position else { return };

    let Ok((&trns, mut collider)) = widgets.get(target).and_then(|p| colliders.get_mut(p.get())) else {
        return
    };

    if collider.vertices.len() <= 3 {
        return
    };

    let hit = (pos - trns.translation()).truncate();
    if let Some((i, ..)) = collider
        .vertices
        .iter()
        .enumerate()
        .min_by_key(|(.., &v)| FloatOrd(v.distance_squared(hit)))
    {
        collider.vertices.remove(i);
    }
}

pub fn drag_start_collider_widget(
    event: Listener<Pointer<DragStart>>,
    mut widgets: Query<(&mut LevelColliderWidget, &Parent)>,
    colliders: Query<(&GlobalTransform, &LevelCollider)>,
) {
    if event.button != PointerButton::Primary {
        return
    };

    let target = event.target();
    let Some(pos) = event.event.hit.position else { return };

    let Ok((mut widget, parent)) = widgets.get_mut(target) else {
        return
    };
    let Ok((&trns, collider)) = colliders.get(parent.get()) else {
        return
    };

    let hit = (pos - trns.translation()).truncate();
    let len = collider.vertices.len();
    let Some((selected, ..)) = collider
        .vertices
        .iter()
        .copied()
        .chain(
            collider
                .vertices
                .iter()
                .enumerate()
                .map(|(i, &v)| v.lerp(collider.vertices[(i + 1) % len], 0.5)),
        )
        .enumerate()
        .min_by_key(|&(.., v)| FloatOrd(v.distance_squared(hit)))
    else {
        return
    };

    if selected < len {
        widget.edit = LevelColliderEdit::Drag(selected, Vec2::ZERO);
    } else {
        widget.edit = LevelColliderEdit::New(selected - len, Vec2::ZERO);
    }
}

pub fn drag_collider_widget(event: Listener<Pointer<Drag>>, mut widgets: Query<&mut LevelColliderWidget>) {
    let target = event.target();
    let Ok(mut widget) = widgets.get_mut(target) else { return };

    match widget.edit {
        LevelColliderEdit::Off => {}
        LevelColliderEdit::Drag(.., ref mut delta) | LevelColliderEdit::New(.., ref mut delta) => {
            *delta += vec2(event.event.delta.x, -event.event.delta.y)
        }
    }
}

pub fn drag_end_collider_widget(
    event: Listener<Pointer<DragEnd>>,
    mut widgets: Query<(&mut LevelColliderWidget, &Parent)>,
    mut colliders: Query<&mut LevelCollider>,
) {
    let target = event.target();
    let Ok((mut widget, parent)) = widgets.get_mut(target) else {
        return
    };

    let Ok(mut collider) = colliders.get_mut(parent.get()) else {
        return
    };

    match widget.edit {
        LevelColliderEdit::Off => {}
        LevelColliderEdit::Drag(selected, delta) => collider.vertices[selected] += delta,
        LevelColliderEdit::New(reference, delta) => {
            let len = collider.vertices.len();
            let between = collider.vertices[reference].lerp(collider.vertices[(reference + 1) % len], 0.5);
            collider.vertices.insert(reference + 1, between + delta)
        }
    }

    widget.edit = LevelColliderEdit::Off;
}
