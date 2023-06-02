use std::collections::HashMap;

use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use strum::EnumString;

// use crate::prelude::*;

#[derive(Deref, DerefMut, Default)]
pub struct GeometryRegistry(pub HashMap<String, Geometry>);

pub fn trim_geo_identifier(identifier: String) -> String {
    if let Some((prefix, _)) = identifier.split_once('.') {
        prefix.to_string()
    } else {
        identifier
    }
}

/*  Technically we could do something similiar to mc for completely custom models. However
due to personal preference i would rather only allow certain types listed below.    */
#[derive(EnumString, Default, Deserialize, Serialize, PartialEq, Eq, Debug, Clone)]
pub enum BlockGeometry {
    #[default]
    Block, // Standard Voxel --DONE
    Stairs,
    Slab,          // Both vertical and horizontal --DONE
    BorderedBlock, //Basically the bottom still touchs the normal bottom of a block but has a border around all the others --DONE
    Fence,
    Flat,           // Flat texture that can go on top of a block --DONE
    Cross,          // Crossed textures think like flowers from a popular block game --DONE
    Custom(String), // Custom models defined by the geometry file type
}

impl BlockGeometry {
    pub fn get_geo_namespace(&self) -> String {
        match self {
            BlockGeometry::Block => "vinox:block".to_string(),
            BlockGeometry::Stairs => "vinox:stair".to_string(),
            BlockGeometry::Slab => "vinox:slab".to_string(),
            BlockGeometry::BorderedBlock => "vinox:border_block".to_string(),
            BlockGeometry::Fence => "vinox:fence".to_string(),
            BlockGeometry::Flat => "vinox:flat".to_string(),
            BlockGeometry::Cross => "vinox:cross".to_string(),
            BlockGeometry::Custom(identifier) => identifier.clone(),
        }
    }
    pub fn get_geo_name(&self) -> String {
        match self {
            BlockGeometry::Block => "block".to_string(),
            BlockGeometry::Stairs => "stair".to_string(),
            BlockGeometry::Slab => "slab".to_string(),
            BlockGeometry::BorderedBlock => "border_block".to_string(),
            BlockGeometry::Fence => "fence".to_string(),
            BlockGeometry::Flat => "flat".to_string(),
            BlockGeometry::Cross => "cross".to_string(),
            BlockGeometry::Custom(identifier) => {
                identifier.to_string() // TODO: Actually let this work
                                       // identifier_to_just_name(identifier.clone()).unwrap()
            }
        }
    }

    pub fn geo_new_block(&self, name: String) -> String {
        name + "." + &self.get_geo_name()
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone, Copy, Hash)]
pub struct FaceDescript {
    pub uv: [((i8, i8), (i8, i8)); 6],
    pub discard: [bool; 6], // Should we completely ignore this face regardless
    pub texture_variance: [bool; 6], // Should we randomly rotate this faces uvs
    pub cull: [bool; 6],    // Should this face be culled if there is a block next to it
    pub origin: (i8, i8, i8),
    pub end: (i8, i8, i8),
    pub rotation: (i8, i8, i8),
    pub pivot: (i8, i8, i8), //CULLING CAN BE DONE BY CHECKING IF ANY GIVEN FACE IS TOUCHING THE SIDES OF THE NEIGHBORS FACE?
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
pub struct BlockGeo {
    pub pivot: (i8, i8, i8),
    pub rotation: (i8, i8, i8),
    pub cubes: Vec<FaceDescript>,
}

// Block is default geometry technically don't need block file cause of this
impl Default for BlockGeo {
    fn default() -> Self {
        BlockGeo {
            pivot: (0, 0, 0),
            rotation: (0, 0, 0),
            cubes: vec![FaceDescript {
                uv: [
                    ((0, 0), (16, 16)),     // West
                    ((0, 0), (16, 16)),     // East
                    ((16, 16), (-16, -16)), // Down
                    ((16, 16), (-16, -16)), // Up
                    ((0, 0), (16, 16)),     // South
                    ((0, 0), (16, 16)),     // North
                ],
                cull: [true, true, true, true, true, true],
                discard: [false, false, false, false, false, false],
                origin: (0, 0, 0),
                end: (16, 16, 16),
                rotation: (0, 0, 0),
                pivot: (0, 0, 0),
                texture_variance: [false, false, false, false, false, false],
            }],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct Geometry {
    pub namespace: String, // TODO: Make sure that we only allow one namespace:name pair
    pub name: String,      // Name of the recipe
    pub blocks: [bool; 6], // Does this block face block the face next to it so its culled
    pub element: BlockGeo,
}
