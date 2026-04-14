//! Animation Graph
//!
//! A graph-based system for blending multiple animations together.
//! Simplified version of Bevy's AnimationGraph for embedded systems.

use super::clip::{AnimationClip, AnimationClipHandle};
use super::player::ActiveAnimation;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// Handle to an animation graph
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationGraphHandle {
    pub(crate) id: u64,
}

impl AnimationGraphHandle {
    /// Create a null handle
    pub fn null() -> Self {
        Self { id: 0 }
    }
    
    /// Check if handle is null
    pub fn is_null(&self) -> bool {
        self.id == 0
    }
}

impl Default for AnimationGraphHandle {
    fn default() -> Self {
        Self::null()
    }
}

/// Index of a node in the animation graph
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AnimationNodeIndex(pub usize);

impl AnimationNodeIndex {
    /// Create a new node index
    pub fn new(index: usize) -> Self {
        Self(index)
    }
}

/// Type of animation graph node
#[derive(Clone, Debug)]
pub enum AnimationNodeType {
    /// Clip node - plays an animation clip
    Clip(AnimationClipHandle),
    /// Blend node - blends children with weights
    Blend,
    /// Additive blend node - adds children together
    Add,
}

/// A node in the animation graph
#[derive(Clone, Debug)]
pub struct AnimationGraphNode {
    /// Node type
    pub node_type: AnimationNodeType,
    /// Node weight for blending
    pub weight: f32,
    /// Child node indices
    pub children: Vec<AnimationNodeIndex>,
}

impl AnimationGraphNode {
    /// Create a new clip node
    pub fn clip(clip_handle: AnimationClipHandle, weight: f32) -> Self {
        Self {
            node_type: AnimationNodeType::Clip(clip_handle),
            weight,
            children: Vec::new(),
        }
    }
    
    /// Create a new blend node
    pub fn blend(weight: f32) -> Self {
        Self {
            node_type: AnimationNodeType::Blend,
            weight,
            children: Vec::new(),
        }
    }
    
    /// Create a new additive blend node
    pub fn add(weight: f32) -> Self {
        Self {
            node_type: AnimationNodeType::Add,
            weight,
            children: Vec::new(),
        }
    }
    
    /// Add a child node
    pub fn add_child(&mut self, child: AnimationNodeIndex) {
        self.children.push(child);
    }
}

/// Animation graph for blending animations
#[derive(Clone, Debug)]
pub struct AnimationGraph {
    /// All nodes in the graph
    nodes: Vec<AnimationGraphNode>,
    /// Root node index
    root: AnimationNodeIndex,
}

impl AnimationGraph {
    /// Create a new empty animation graph
    pub fn new() -> Self {
        let root_node = AnimationGraphNode::blend(1.0);
        Self {
            nodes: alloc::vec![root_node],
            root: AnimationNodeIndex(0),
        }
    }
    
    /// Create a graph from a single clip
    pub fn from_clip(clip_handle: AnimationClipHandle) -> (Self, AnimationNodeIndex) {
        let mut graph = Self::new();
        let node = graph.add_clip(clip_handle, 1.0, graph.root);
        (graph, node)
    }
    
    /// Get the root node index
    pub fn root(&self) -> AnimationNodeIndex {
        self.root
    }
    
    /// Get a node by index
    pub fn node(&self, index: AnimationNodeIndex) -> Option<&AnimationGraphNode> {
        self.nodes.get(index.0)
    }
    
    /// Get a mutable node by index
    pub fn node_mut(&mut self, index: AnimationNodeIndex) -> Option<&mut AnimationGraphNode> {
        self.nodes.get_mut(index.0)
    }
    
    /// Add a clip node
    pub fn add_clip(
        &mut self,
        clip_handle: AnimationClipHandle,
        weight: f32,
        parent: AnimationNodeIndex,
    ) -> AnimationNodeIndex {
        let node = AnimationGraphNode::clip(clip_handle, weight);
        let index = AnimationNodeIndex(self.nodes.len());
        self.nodes.push(node);
        
        if let Some(parent_node) = self.nodes.get_mut(parent.0) {
            parent_node.add_child(index);
        }
        
        index
    }
    
    /// Add a blend node
    pub fn add_blend(&mut self, weight: f32, parent: AnimationNodeIndex) -> AnimationNodeIndex {
        let node = AnimationGraphNode::blend(weight);
        let index = AnimationNodeIndex(self.nodes.len());
        self.nodes.push(node);
        
        if let Some(parent_node) = self.nodes.get_mut(parent.0) {
            parent_node.add_child(index);
        }
        
        index
    }
    
    /// Add an additive blend node
    pub fn add_add(&mut self, weight: f32, parent: AnimationNodeIndex) -> AnimationNodeIndex {
        let node = AnimationGraphNode::add(weight);
        let index = AnimationNodeIndex(self.nodes.len());
        self.nodes.push(node);
        
        if let Some(parent_node) = self.nodes.get_mut(parent.0) {
            parent_node.add_child(index);
        }
        
        index
    }
    
    /// Evaluate the graph and return blended animations
    pub fn evaluate(&self, animations: &mut Vec<(AnimationClipHandle, f32)>) {
        animations.clear();
        self.evaluate_node(self.root, 1.0, animations);
    }
    
    /// Recursively evaluate a node
    fn evaluate_node(
        &self,
        node_index: AnimationNodeIndex,
        parent_weight: f32,
        animations: &mut Vec<(AnimationClipHandle, f32)>,
    ) {
        let Some(node) = self.nodes.get(node_index.0) else {
            return;
        };
        
        let weight = node.weight * parent_weight;
        
        match &node.node_type {
            AnimationNodeType::Clip(clip_handle) => {
                animations.push((*clip_handle, weight));
            }
            AnimationNodeType::Blend => {
                // Normalize child weights
                let total_weight: f32 = node.children.iter()
                    .filter_map(|child| self.nodes.get(child.0))
                    .map(|child| child.weight)
                    .sum();
                
                if total_weight > 0.0 {
                    for child in &node.children {
                        self.evaluate_node(*child, weight, animations);
                    }
                }
            }
            AnimationNodeType::Add => {
                // Additive blending - don't normalize
                for child in &node.children {
                    self.evaluate_node(*child, weight, animations);
                }
            }
        }
    }
    
    /// Get all clip nodes in the graph
    pub fn clip_nodes(&self) -> Vec<(AnimationNodeIndex, AnimationClipHandle)> {
        let mut clips = Vec::new();
        for (i, node) in self.nodes.iter().enumerate() {
            if let AnimationNodeType::Clip(handle) = &node.node_type {
                clips.push((AnimationNodeIndex(i), *handle));
            }
        }
        clips
    }
}

impl Default for AnimationGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Blend node helper
pub struct BlendNode;

impl BlendNode {
    /// Blend two values with a weight
    pub fn blend<T: Blendable>(a: T, b: T, t: f32) -> T {
        a.blend(b, t)
    }
}

/// Trait for types that can be blended
pub trait Blendable: Copy {
    /// Blend with another value (t in 0..1)
    fn blend(self, other: Self, t: f32) -> Self;
}

impl Blendable for f32 {
    fn blend(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

/// Clip node helper
pub struct ClipNode;

impl ClipNode {
    /// Create a new clip node
    pub fn new(clip_handle: AnimationClipHandle, weight: f32) -> AnimationGraphNode {
        AnimationGraphNode::clip(clip_handle, weight)
    }
}
