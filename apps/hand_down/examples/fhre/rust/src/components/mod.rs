//! Demo Components
//!
//! 示例组件，用于演示 FHRE 的渲染能力。
//! 这些组件是示例的一部分，不是 FHRE 核心库的一部分。

pub mod button;
pub mod cube;
pub mod soccer_ball;

pub use button::{Button, ButtonState};
pub use cube::{Cube, CubeFace};
pub use soccer_ball::SoccerBall;
