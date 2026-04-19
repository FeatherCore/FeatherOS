//! Animation Properties
//!
//! Defines which properties can be animated and how to apply animation values to components.

use crate::math::{Vec2, Vec3, Color};
use crate::main_world::{Component, Entity};
use super::curve::Animatable;

/// Properties that can be animated on entities
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnimationProperty {
    // === Translation (3D position) ===
    /// Transform position X
    TranslationX,
    /// Transform position Y
    TranslationY,
    /// Transform position Z
    TranslationZ,

    // === Rotation (3D Euler angles, degrees) ===
    /// Rotation around X axis (pitch)
    RotationX,
    /// Rotation around Y axis (yaw)
    RotationY,
    /// Rotation around Z axis (roll)
    RotationZ,

    // === Scale (3D uniform or per-axis) ===
    /// Scale factor X
    ScaleX,
    /// Scale factor Y
    ScaleY,
    /// Scale factor Z
    ScaleZ,

    // === Sprite / UI properties ===
    /// Sprite color R
    ColorR,
    /// Sprite color G
    ColorG,
    /// Sprite color B
    ColorB,
    /// Sprite color A (alpha)
    ColorA,
    /// Sprite width
    SpriteWidth,
    /// Sprite height
    SpriteHeight,

    // === Custom ===
    /// Custom property index
    Custom(u32),
}

impl AnimationProperty {
    /// Get the value type for this property
    pub fn value_type(&self) -> PropertyValueType {
        match self {
            AnimationProperty::TranslationX |
            AnimationProperty::TranslationY |
            AnimationProperty::TranslationZ |
            AnimationProperty::RotationX |
            AnimationProperty::RotationY |
            AnimationProperty::RotationZ |
            AnimationProperty::ScaleX |
            AnimationProperty::ScaleY |
            AnimationProperty::ScaleZ |
            AnimationProperty::ColorR |
            AnimationProperty::ColorG |
            AnimationProperty::ColorB |
            AnimationProperty::ColorA |
            AnimationProperty::SpriteWidth |
            AnimationProperty::SpriteHeight => PropertyValueType::Float,
            AnimationProperty::Custom(_) => PropertyValueType::Float,
        }
    }
}

/// Value types for animation properties
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PropertyValueType {
    /// Single float value
    Float,
    /// 2D vector
    Vec2,
    /// 3D vector
    Vec3,
    /// Color
    Color,
}

/// Animated property value
#[derive(Clone, Copy, Debug)]
pub enum AnimatedValue {
    /// Float value
    Float(f32),
    /// Vec2 value
    Vec2(Vec2),
    /// Vec3 value
    Vec3(Vec3),
    /// Color value
    Color(Color),
}

impl AnimatedValue {
    /// Get as float
    pub fn as_float(&self) -> Option<f32> {
        match self {
            AnimatedValue::Float(v) => Some(*v),
            _ => None,
        }
    }
    
    /// Get as Vec2
    pub fn as_vec2(&self) -> Option<Vec2> {
        match self {
            AnimatedValue::Vec2(v) => Some(*v),
            _ => None,
        }
    }
    
    /// Get as Vec3
    pub fn as_vec3(&self) -> Option<Vec3> {
        match self {
            AnimatedValue::Vec3(v) => Some(*v),
            _ => None,
        }
    }
    
    /// Get as Color
    pub fn as_color(&self) -> Option<Color> {
        match self {
            AnimatedValue::Color(v) => Some(*v),
            _ => None,
        }
    }
}

/// Trait for fields that can be animated
pub trait AnimatedField<C: Component>: Clone + 'static {
    /// The value type of this field
    type Value: Animatable;
    
    /// Get the current value from the component
    fn get(&self, component: &C) -> Self::Value;
    
    /// Set a new value on the component
    fn set(&self, component: &mut C, value: Self::Value);
}

/// A field on a component that can be animated
#[derive(Clone, Copy)]
pub struct ComponentField<C: Component, T: Animatable> {
    /// Getter function
    getter: fn(&C) -> T,
    /// Setter function
    setter: fn(&mut C, T),
}

impl<C: Component, T: Animatable> ComponentField<C, T> {
    /// Create a new component field
    pub fn new(getter: fn(&C) -> T, setter: fn(&mut C, T)) -> Self {
        Self { getter, setter }
    }
}

impl<C: Component + Clone, T: Animatable> AnimatedField<C> for ComponentField<C, T> {
    type Value = T;
    
    fn get(&self, component: &C) -> T {
        (self.getter)(component)
    }
    
    fn set(&self, component: &mut C, value: T) {
        (self.setter)(component, value);
    }
}

/// Animation target - identifies what entity and property to animate
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimationTarget {
    /// Target entity
    pub entity: Entity,
    /// Property to animate
    pub property: AnimationProperty,
}

impl AnimationTarget {
    /// Create a new animation target
    pub fn new(entity: Entity, property: AnimationProperty) -> Self {
        Self { entity, property }
    }
}

/// Target identifier for animation clips (similar to Bevy's AnimationTargetId)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AnimationTargetId(pub u64);

impl AnimationTargetId {
    /// Create a new target ID from a raw value
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    
    /// Create a target ID from a path (e.g., "Arm/Hand/Finger")
    pub fn from_path(path: &str) -> Self {
        // Simple hash of the path
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in path.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        Self(hash)
    }
    
    /// Create target IDs from path components
    pub fn from_iter<'a>(parts: impl Iterator<Item = &'a str>) -> Self {
        let path: alloc::string::String = parts.collect::<alloc::vec::Vec<_>>().join("/");
        Self::from_path(&path)
    }
}

impl Default for AnimationTargetId {
    fn default() -> Self {
        Self(0)
    }
}

/// Macro to define an animated field
#[macro_export]
macro_rules! animated_field {
    ($component:ty, $field:ident) => {
        $crate::animation::property::ComponentField::new(
            |c: &$component| c.$field,
            |c: &mut $component, v| c.$field = v,
        )
    };
}

pub fn apply_animated_value(
    property: AnimationProperty,
    value: f32,
    transform: Option<&mut crate::node::Transform>,
    sprite: Option<&mut crate::main_world::Sprite>,
) {
    use AnimationProperty::*;
    
    match property {
        TranslationX => { if let Some(t) = transform { t.position.x = value; } }
        TranslationY => { if let Some(t) = transform { t.position.y = value; } }
        TranslationZ => { if let Some(t) = transform { t.position.z = value; } }
        RotationX => { if let Some(t) = transform { t.rotation.x = value; } }
        RotationY => { if let Some(t) = transform { t.rotation.y = value; } }
        RotationZ => { if let Some(t) = transform { t.rotation.z = value; } }
        ScaleX => { if let Some(t) = transform { t.scale.x = value; } }
        ScaleY => { if let Some(t) = transform { t.scale.y = value; } }
        ScaleZ => { if let Some(t) = transform { t.scale.z = value; } }
        ColorR => { if let Some(s) = sprite { s.color.r = value as u8; } }
        ColorG => { if let Some(s) = sprite { s.color.g = value as u8; } }
        ColorB => { if let Some(s) = sprite { s.color.b = value as u8; } }
        ColorA => { if let Some(s) = sprite { s.color.a = value as u8; } }
        SpriteWidth => { if let Some(s) = sprite { s.width = value; } }
        SpriteHeight => { if let Some(s) = sprite { s.height = value; } }
        Custom(_) => {}
    }
}

/// Trait for components that can receive animation values.
///
/// Implement this trait for any component that should receive animation values.
/// The animation system's apply systems will automatically call `apply_animation()`
/// for each animated property on the entity.
///
/// This is fhre's equivalent of Bevy's property animation mechanism — it decouples
/// the animation system from specific component types.
///
/// # Example
/// ```rust
/// impl AnimationReceiver for Cube {
///     fn apply_animation(&mut self, property: AnimationProperty, value: f32) {
///         match property {
///             AnimationProperty::RotationX => { self.rotation.x = value; }
///             AnimationProperty::RotationY => { self.rotation.y = value; }
///             AnimationProperty::RotationZ => { self.rotation.z = value; }
///             AnimationProperty::ScaleX => { self.size = value; }
///             _ => {}
///         }
///     }
/// }
/// ```
pub trait AnimationReceiver {
    /// Apply an animated property value to this component.
    ///
    /// Called by the `animate_targets` system after sampling an AnimationClip.
    fn apply_animation(&mut self, property: AnimationProperty, value: f32);
}
