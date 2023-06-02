use std::collections::HashMap;

use crate::prelude::*;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use strum::EnumString;

pub trait VoxRegistry<V: Voxel<Self> + Sized>
where
    Self: Sized,
{
    fn is_empty(&self, vox: V) -> bool;
}

pub trait Voxel<R: VoxRegistry<Self> + Sized>
where
    Self: Sized,
{
    /// Implement this to tell if a voxel is empty
    fn is_empty(&self, registry: Option<&R>) -> bool;
    /// Impelement this for true emptiness. For example you may want light to propogate through blocks that are partially empty so thats what is_empty returns true_empty should be blocks that don't have custom geometry and are not opaque.
    fn is_true_empty(&self, registry: Option<&R>) -> bool;
    /// Oposite of is_empty
    fn is_opaque(&self, registry: Option<&R>) -> bool;
    /// Identifier must be something that implements eq
    fn identifier(&self) -> String;
}

#[derive(EnumString, Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone, Copy, Hash)]
pub enum VoxelVisibility {
    #[default]
    Empty,
    Opaque,
    Transparent,
}

// Anything optional here that is necessary for the game to function but we have a default value for ie texture or geometry
// NOTE: We will also take in any children blocks this block may have. ie any slab, fence, stair variant etc
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct Block {
    pub identifier: String, // TODO: Make sure that we only allow one namespace:name pair
    pub textures: Option<HashMap<Option<String>, Option<String>>>,
    pub geometry: Option<BlockGeometry>,
    pub auto_geo: Option<Vec<BlockGeometry>>, // Contains strings of geometry we wan't to auto generate
    pub tex_variance: Option<[Option<bool>; 6]>,
    pub visibility: Option<VoxelVisibility>,
    pub has_item: Option<bool>, // Basically whether or not we should auto generate an item for this block                                // pub properties: Option<Vec<Box<BlockData>>>,
}

#[derive(Deref, DerefMut, Default, Clone, Serialize, Deserialize)]
pub struct BlockRegistry(pub HashMap<String, Block>);

impl VoxRegistry<BlockData> for BlockRegistry {
    fn is_empty(&self, vox: BlockData) -> bool {
        self.0
            .get(&vox.identifier)
            .is_some_and(|x| x.visibility.is_some_and(|y| y == VoxelVisibility::Empty))
    }
}

#[cfg(feature = "block")]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub enum Property {
    Bool,
    BoolArray,
    Int,
    IntArray,
    Float,
    FloatArray,
}

#[cfg(feature = "block")]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct BlockData {
    pub identifier: String,
    pub last_tick: Option<u64>,
    pub properties: Option<Vec<(String, Property)>>,
}

#[cfg(feature = "block")]
impl Voxel<BlockRegistry> for BlockData {
    fn is_empty(&self, registry: Option<&BlockRegistry>) -> bool {
        !registry.is_some_and(|z| {
            z.get(&self.identifier)
                .is_some_and(|x| x.visibility.is_some_and(|y| y != VoxelVisibility::Empty))
        })
    }

    fn is_opaque(&self, registry: Option<&BlockRegistry>) -> bool {
        registry.is_some_and(|z| {
            z.get(&self.identifier)
                .is_some_and(|x| x.visibility.is_some_and(|y| y == VoxelVisibility::Opaque))
        })
    }

    fn is_true_empty(&self, registry: Option<&BlockRegistry>) -> bool {
        !registry.is_some_and(|z| {
            z.get(&self.identifier)
                .is_some_and(|x| x.visibility.is_some_and(|y| y != VoxelVisibility::Empty))
        }) && registry.is_some_and(|z| {
            z.get(&self.identifier).is_some_and(|x| {
                x.geometry.clone().unwrap_or_default().get_geo_namespace() == "vinox:block"
            })
        })
    }

    fn identifier(&self) -> String {
        self.identifier.clone()
    }
}

#[cfg(feature = "block")]
impl Default for BlockData {
    fn default() -> Self {
        BlockData {
            identifier: "vinox:air".to_string(),
            last_tick: None,
            properties: None,
        }
    }
}

#[cfg(feature = "block")]
impl BlockData {
    pub fn new(namespace: String, name: String) -> Self {
        BlockData {
            identifier: namespace + &name,
            ..Default::default()
        }
    }
}
