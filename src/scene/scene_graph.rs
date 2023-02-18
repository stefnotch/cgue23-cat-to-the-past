use std::sync::Arc;

use cgmath::{Matrix4, Quaternion, Vector3, Zero};

use super::{material::Material, mesh::Mesh};

// TODO: What if the entire game logic isn't allowed to change the nodes.
// *Instead*, it generates a *patch* (a redo action) and that patch is applied
// to the scene graph. This way, the scene graph is always in a consistent state
// and the game logic can't break it.
// And things like the local and world transforms can be cached a bit better.
/// A flat scene graph.
pub struct SceneGraph {
    children: Vec<SceneNode>,
}

pub struct SceneNode {
    local_transform: Transform,
    bounding_box: BoundingBox,
    data: SceneNodeData,
}

pub trait FromSceneNodeData {
    /// for getting
    fn from_scene_node_data(value: &SceneNodeData) -> Option<&Self>
    where
        Self: Sized;
}

pub enum SceneNodeData {
    Model(Model),
    RigidBody(RigidBody),
    Light(Light),
    Empty,
}

struct Transform {
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: Vector3<f32>,
}
impl Transform {
    fn new() -> Transform {
        Transform {
            position: Vector3::zero(),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    fn to_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }
}

/// Axis-aligned bounding box.
struct BoundingBox {
    min: Vector3<f32>,
    max: Vector3<f32>,
}
impl BoundingBox {
    fn new() -> BoundingBox {
        BoundingBox {
            min: Vector3::zero(),
            max: Vector3::zero(),
        }
    }
}

pub struct Model {
    pub mesh: Arc<Mesh>,
    // S: DescriptorSetsCollection
    pub material: Arc<Material>,
}

pub struct RigidBody {
    // physics engine stuff
}

pub struct Light {
    color: Vector3<f32>,
    intensity: f32,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn add<T>(&mut self, data: T)
    where
        T: Into<SceneNode>,
    {
        let node = data.into();
        self.children.push(node);
    }

    pub fn get_data_recursive<'a, T, U>(
        &'a self,
        callback: fn(&'a SceneNode) -> U,
    ) -> Vec<(&'a T, U)>
    where
        T: FromSceneNodeData,
        U: 'a,
    {
        let mut result = Vec::new();
        self.children.iter().for_each(|child| {
            child.get_data_recursive(callback, &mut result);
        });
        result
    }
}

impl SceneNode {
    fn new() -> Self {
        Self {
            local_transform: Transform::new(),
            bounding_box: BoundingBox::new(),
            data: SceneNodeData::Empty,
        }
    }

    // TODO: Figure out why the lifetime annotations work like that
    fn get_data_recursive<'a, 'b, T, U>(
        &'a self,
        callback: fn(&'a SceneNode) -> U,
        mut result: &'b mut Vec<(&'a T, U)>,
    ) where
        T: FromSceneNodeData,
    {
        if let Some(value) = T::from_scene_node_data(&self.data) {
            result.push((value, callback(self)));
        }
    }

    pub(crate) fn world_matrix(&self) -> Matrix4<f32> {
        self.local_transform.to_matrix()
    }
}

impl FromSceneNodeData for Model {
    fn from_scene_node_data(value: &SceneNodeData) -> Option<&Self> {
        match value {
            SceneNodeData::Model(v) => Some(v),
            _ => None,
        }
    }
}

impl Into<SceneNode> for Model {
    fn into(self) -> SceneNode {
        SceneNode {
            local_transform: Transform::new(),
            bounding_box: BoundingBox::new(),
            data: SceneNodeData::Model(self),
        }
    }
}
