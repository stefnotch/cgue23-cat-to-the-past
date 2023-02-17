use cgmath::{Quaternion, Vector2, Vector3, Zero};

use super::mesh::Mesh;

pub struct SceneGraph {
    root: SceneNode,
}

struct SceneNode {
    transform: Transform,
    bounding_box: BoundingBox,
    data: SceneNodeData,
    children: Vec<SceneNode>,
}

trait ToSceneNodeData {
    /// for inserting
    fn to_scene_node_data(self) -> SceneNodeData;
}
trait FromSceneNodeData {
    /// for getting
    fn from_scene_node_data(value: &SceneNodeData) -> Option<&Self>
    where
        Self: Sized;
}

enum SceneNodeData {
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

struct Model {
    geometry: Mesh,
    material: (), //Material, // references a shader and its inputs
}

struct RigidBody {
    // physics engine stuff
}

struct Light {
    color: Vector3<f32>,
    intensity: f32,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self {
            root: SceneNode::new(),
        }
    }

    fn get_data_recursive<'a, T>(&'a self) -> Vec<&'a T>
    where
        T: FromSceneNodeData,
    {
        let mut result = Vec::new();
        {
            self.root.get_data_recursive(&mut result);
        }
        result
    }
}

impl SceneNode {
    fn new() -> Self {
        Self {
            transform: Transform::new(),
            bounding_box: BoundingBox::new(),
            data: SceneNodeData::Empty,
            children: Vec::new(),
        }
    }

    // TODO: Figure out why the lifetime annotations work like that
    fn get_data_recursive<'a, 'b, T>(&'a self, mut result: &'b mut Vec<&'a T>)
    where
        T: FromSceneNodeData,
    {
        if let Some(value) = T::from_scene_node_data(&self.data) {
            result.push(value);
        }

        self.children.iter().for_each(|child| {
            child.get_data_recursive(&mut result);
        });
    }
}

impl ToSceneNodeData for Model {
    fn to_scene_node_data(self) -> SceneNodeData {
        SceneNodeData::Model(self)
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
