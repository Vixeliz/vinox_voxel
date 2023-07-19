use crate::prelude::*;
use serde::Serialize;
use std::ops::Deref;

pub const EMPTY: VoxelVisibility = VoxelVisibility::Empty;
pub const OPAQUE: VoxelVisibility = VoxelVisibility::Opaque;
pub const TRANSPARENT: VoxelVisibility = VoxelVisibility::Transparent;

#[derive(Clone, Debug)]
pub struct Quad {
    pub voxel: [usize; 3],
    pub start: (i8, i8, i8),
    pub end: (i8, i8, i8),
    pub cube: usize,
    pub data: RenderedBlockData,
}

#[derive(Default)]
pub struct QuadGroups {
    pub groups: [Vec<Quad>; 6],
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Side {
    pub axis: Axis,
    pub positive: bool,
}

impl Side {
    pub fn new(axis: Axis, positive: bool) -> Self {
        Self { axis, positive }
    }

    pub fn normal(&self) -> [f32; 3] {
        match (&self.axis, &self.positive) {
            (Axis::X, true) => [1.0, 0.0, 0.0],   // X+
            (Axis::X, false) => [-1.0, 0.0, 0.0], // X-
            (Axis::Y, true) => [0.0, 1.0, 0.0],   // Y+
            (Axis::Y, false) => [0.0, -1.0, 0.0], // Y-
            (Axis::Z, true) => [0.0, 0.0, 1.0],   // Z+
            (Axis::Z, false) => [0.0, 0.0, -1.0], // Z-
        }
    }

    pub fn normals(&self) -> [[f32; 3]; 4] {
        [self.normal(), self.normal(), self.normal(), self.normal()]
    }
}

pub struct Face<'a> {
    side: Side,
    quad: &'a Quad,
}

impl From<usize> for Side {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::new(Axis::X, false), // X-
            1 => Self::new(Axis::X, true),  // X+
            2 => Self::new(Axis::Y, false), // Y-
            3 => Self::new(Axis::Y, true),  // Y+
            4 => Self::new(Axis::Z, false), // Z-
            5 => Self::new(Axis::Z, true),  // Z+
            _ => unreachable!(),
        }
    }
}
impl QuadGroups {
    pub fn iter(&self) -> impl Iterator<Item = Face> {
        self.groups
            .iter()
            .enumerate()
            .flat_map(|(index, quads)| quads.iter().map(move |quad| (index, quad)))
            .map(|(index, quad)| Face {
                side: index.into(),
                quad,
            })
    }

    pub fn iter_with_ao<
        'a,
        V: Voxel<R> + Clone + Serialize + Eq + Default + RenderedVoxel<V, R>,
        R: VoxRegistry<V> + Clone + Default,
    >(
        &'a self,
        chunk: &'a ChunkBoundary<V, R>,
    ) -> impl Iterator<Item = FaceWithAO<'a>> {
        self.iter().map(|face| FaceWithAO::new(face, chunk))
    }

    pub fn clear(&mut self) {
        self.groups.iter_mut().for_each(|g| g.clear());
    }
}

pub fn face_aos<
    V: Voxel<R> + Clone + Serialize + Eq + Default + RenderedVoxel<V, R>,
    R: VoxRegistry<V> + Clone + Default,
>(
    face: &Face,
    chunk: &ChunkBoundary<V, R>,
) -> [u32; 4] {
    let [x, y, z] = face.voxel();
    // let (x, y, z) = (x as u32, y as u32, z as u32);
    match (face.side.axis, face.side.positive) {
        (Axis::X, false) => side_aos([
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
        ]),
        (Axis::X, true) => side_aos([
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
        ]),
        (Axis::Y, false) => side_aos([
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y - 1, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y - 1, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y - 1, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y - 1, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
        ]),
        (Axis::Y, true) => side_aos([
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y + 1, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y + 1, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y + 1, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y + 1, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
        ]),
        (Axis::Z, false) => side_aos([
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y - 1, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y - 1, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y + 1, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y + 1, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z - 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z - 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
        ]),
        (Axis::Z, true) => side_aos([
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y - 1, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y - 1, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y + 1, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y + 1, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
            (
                chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z + 1)],
                &chunk
                    .geometry_pal
                    .palette
                    .get(
                        chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z + 1)]
                            .geo_index
                            .unwrap_or_default(),
                    )
                    .unwrap()
                    .clone(),
            ),
        ]),
    }
}

pub fn face_lights<
    V: Voxel<R> + Clone + Serialize + Eq + Default + RenderedVoxel<V, R>,
    R: VoxRegistry<V> + Clone + Default,
>(
    face: &Face,
    chunk: &ChunkBoundary<V, R>,
) -> [f32; 4] {
    let [x, y, z] = face.voxel();
    // let (x, y, z) = (x as u32, y as u32, z as u32);
    match (face.side.axis, face.side.positive) {
        (Axis::X, false) => side_light([
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y, z + 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z + 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z - 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y, z - 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z - 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z + 1)]
                .light
                .unwrap_or(16),
        ]),
        (Axis::X, true) => side_light([
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y, z - 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z - 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z + 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y, z + 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z + 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z - 1)]
                .light
                .unwrap_or(16),
        ]),
        (Axis::Y, false) => side_light([
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z + 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y - 1, z + 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z + 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z - 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y - 1, z - 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z - 1)]
                .light
                .unwrap_or(16),
        ]),
        (Axis::Y, true) => side_light([
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y + 1, z + 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z + 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z - 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y + 1, z - 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z - 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z + 1)]
                .light
                .unwrap_or(16),
        ]),
        (Axis::Z, false) => side_light([
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y, z - 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z - 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y - 1, z - 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z - 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y, z - 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z - 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y + 1, z - 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z - 1)]
                .light
                .unwrap_or(16),
        ]),
        (Axis::Z, true) => side_light([
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y, z + 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y - 1, z + 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y - 1, z + 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y - 1, z + 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y, z + 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y + 1, z + 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y + 1, z + 1)]
                .light
                .unwrap_or(16),
            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y + 1, z + 1)]
                .light
                .unwrap_or(16),
        ]),
    }
}

pub struct FaceWithAO<'a> {
    face: Face<'a>,
    aos: [u32; 4],
    light: [f32; 4],
}

impl<'a> FaceWithAO<'a> {
    pub fn new<
        V: Voxel<R> + Clone + Serialize + Eq + Default + RenderedVoxel<V, R>,
        R: VoxRegistry<V> + Clone + Default,
    >(
        face: Face<'a>,
        chunk: &ChunkBoundary<V, R>,
    ) -> Self {
        let aos = face_aos(&face, chunk);
        let light = face_lights(&face, chunk);
        Self { face, aos, light }
    }

    pub fn aos(&self) -> [u32; 4] {
        self.aos
    }

    pub fn light(&self) -> [f32; 4] {
        self.light
    }

    pub fn indices(&self, start: u32) -> [u32; 6] {
        let aos = self.aos();

        if (aos[1] + aos[2]) > (aos[0] + aos[3]) {
            [start, start + 2, start + 1, start + 1, start + 2, start + 3]
        } else {
            [start, start + 3, start + 1, start, start + 2, start + 3]
        }
    }
}

pub(crate) fn ao_value(side1: bool, corner: bool, side2: bool) -> u32 {
    match (side1, corner, side2) {
        (true, _, true) => 0,
        (true, true, false) | (false, true, true) => 1,
        (false, false, false) => 3,
        _ => 2,
    }
}

pub(crate) fn side_aos(neighbors: [(RenderedBlockData, &BlockGeo); 8]) -> [u32; 4] {
    let ns = [
        neighbors[0].0.visibility == OPAQUE,
        neighbors[0].1 == &BlockGeo::default(),
        neighbors[1].0.visibility == OPAQUE,
        neighbors[1].1 == &BlockGeo::default(),
        neighbors[2].0.visibility == OPAQUE,
        neighbors[2].1 == &BlockGeo::default(),
        neighbors[3].0.visibility == OPAQUE,
        neighbors[3].1 == &BlockGeo::default(),
        neighbors[4].0.visibility == OPAQUE,
        neighbors[4].1 == &BlockGeo::default(),
        neighbors[5].0.visibility == OPAQUE,
        neighbors[5].1 == &BlockGeo::default(),
        neighbors[6].0.visibility == OPAQUE,
        neighbors[6].1 == &BlockGeo::default(),
        neighbors[7].0.visibility == OPAQUE,
        neighbors[7].1 == &BlockGeo::default(),
    ];

    [
        ao_value(ns[0], ns[1], ns[2]),
        ao_value(ns[2], ns[3], ns[4]),
        ao_value(ns[6], ns[7], ns[0]),
        ao_value(ns[4], ns[5], ns[6]),
    ]
}

pub(crate) fn side_light(neighbors: [u8; 8]) -> [f32; 4] {
    [
        (light_to_intern(neighbors[0])
            + light_to_intern(neighbors[1])
            + light_to_intern(neighbors[2]))
            / 3.0,
        (light_to_intern(neighbors[2])
            + light_to_intern(neighbors[3])
            + light_to_intern(neighbors[4]))
            / 3.0,
        (light_to_intern(neighbors[6])
            + light_to_intern(neighbors[7])
            + light_to_intern(neighbors[0]))
            / 3.0,
        (light_to_intern(neighbors[4])
            + light_to_intern(neighbors[5])
            + light_to_intern(neighbors[6]))
            / 3.0,
    ]
}

impl<'a> Deref for FaceWithAO<'a> {
    type Target = Face<'a>;

    fn deref(&self) -> &Self::Target {
        &self.face
    }
}

impl<'a> Face<'a> {
    pub fn indices(&self, start: u32) -> [u32; 6] {
        [start, start + 2, start + 1, start + 1, start + 2, start + 3]
    }

    pub fn positions<
        V: Voxel<R> + Clone + Serialize + Eq + Default + RenderedVoxel<V, R>,
        R: VoxRegistry<V> + Clone + Default,
    >(
        &self,
        voxel_size: f32,
        chunk: &ChunkBoundary<V, R>,
    ) -> [[f32; 3]; 4] {
        let (min_one, min_two, max_one, max_two, min_self, max_self) = (
            (self.quad.start.0 as f32 / 16.0),
            (self.quad.start.1 as f32 / 16.0),
            (self.quad.end.0 as f32 / 16.0),
            (self.quad.end.1 as f32 / 16.0),
            (self.quad.start.2 as f32 / 16.0),
            (self.quad.end.2 as f32 / 16.0),
        );
        let positions = match (&self.side.axis, &self.side.positive) {
            (Axis::X, false) => [
                [min_self, min_one, max_two],
                [min_self, min_one, min_two],
                [min_self, max_one, max_two],
                [min_self, max_one, min_two],
            ],
            (Axis::X, true) => [
                [max_self, min_one, min_two],
                [max_self, min_one, max_two],
                [max_self, max_one, min_two],
                [max_self, max_one, max_two],
            ],
            (Axis::Y, false) => [
                [min_one, min_self, max_two],
                [max_one, min_self, max_two],
                [min_one, min_self, min_two],
                [max_one, min_self, min_two],
            ],
            (Axis::Y, true) => [
                [min_one, max_self, max_two],
                [min_one, max_self, min_two],
                [max_one, max_self, max_two],
                [max_one, max_self, min_two],
            ],
            (Axis::Z, false) => [
                [min_one, min_two, min_self],
                [max_one, min_two, min_self],
                [min_one, max_two, min_self],
                [max_one, max_two, min_self],
            ],
            (Axis::Z, true) => [
                [max_one, min_two, max_self],
                [min_one, min_two, max_self],
                [max_one, max_two, max_self],
                [min_one, max_two, max_self],
            ],
        };

        let (x, y, z) = (
            (self.quad.voxel[0] - 1) as f32,
            (self.quad.voxel[1] - 1) as f32,
            (self.quad.voxel[2] - 1) as f32,
        );
        let mut temp_arr = [
            glam::Vec3::new(
                x * voxel_size + positions[0][0] * voxel_size,
                y * voxel_size + positions[0][1] * voxel_size,
                z * voxel_size + positions[0][2] * voxel_size,
            ),
            glam::Vec3::new(
                x * voxel_size + positions[1][0] * voxel_size,
                y * voxel_size + positions[1][1] * voxel_size,
                z * voxel_size + positions[1][2] * voxel_size,
            ),
            glam::Vec3::new(
                x * voxel_size + positions[2][0] * voxel_size,
                y * voxel_size + positions[2][1] * voxel_size,
                z * voxel_size + positions[2][2] * voxel_size,
            ),
            glam::Vec3::new(
                x * voxel_size + positions[3][0] * voxel_size,
                y * voxel_size + positions[3][1] * voxel_size,
                z * voxel_size + positions[3][2] * voxel_size,
            ),
        ];
        let geo = chunk
            .geometry_pal
            .palette
            .get(self.quad.data.geo_index.unwrap_or_default())
            .unwrap();
        let cube_pivot = geo.cubes.get(self.quad.cube).unwrap().pivot;
        let cube_rotation = geo.cubes.get(self.quad.cube).unwrap().rotation;
        let block_pivot = geo.pivot;
        let block_rotation = geo.rotation;
        if cube_rotation != (0, 0, 0) || block_rotation != (0, 0, 0)
        // && self.quad.data.direction.is_none()
        // && self.quad.data.top.is_none()
        {
            let pivot = glam::Vec3::new(
                block_pivot.0 as f32 / 16.0 + x,
                block_pivot.1 as f32 / 16.0 + y,
                block_pivot.2 as f32 / 16.0 + z,
            ); // TO emulate how itll be getting from geometry
            let rotation = glam::Quat::from_euler(
                glam::EulerRot::XYZ,
                (block_rotation.0 as f32).to_radians(),
                (block_rotation.1 as f32).to_radians(),
                (block_rotation.2 as f32).to_radians(),
            );
            let pivot_cube = glam::Vec3::new(
                cube_pivot.0 as f32 / 16.0 + x,
                cube_pivot.1 as f32 / 16.0 + y,
                cube_pivot.2 as f32 / 16.0 + z,
            ); // TO emulate how itll be getting from geometry
            let rotation_cube = glam::Quat::from_euler(
                glam::EulerRot::XYZ,
                (cube_rotation.0 as f32).to_radians(),
                (cube_rotation.1 as f32).to_radians(),
                (cube_rotation.2 as f32).to_radians(),
            );
            for point in temp_arr.iter_mut() {
                *point = pivot + rotation * (*point - pivot);
                *point = pivot_cube + rotation_cube * (*point - pivot_cube);
                // if let Some(direction) = self.quad.data.direction {
                //     let pivot = Vec3::new(0.5 + x, 0.5 + y, 0.5 + z); // TO emulate how itll be getting from geometry
                //     let rotation = match direction {
                //         storage::Direction::North => {
                //             Quat::from_euler(EulerRot::XYZ, -90.0_f32.to_radians(), 0.0, 0.0)
                //         }
                //         storage::Direction::South => {
                //             Quat::from_euler(EulerRot::XYZ, 90.0_f32.to_radians(), 0.0, 0.0)
                //         }
                //         storage::Direction::West => {
                //             Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 90.0_f32.to_radians())
                //         }
                //         storage::Direction::East => {
                //             Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, -90.0_f32.to_radians())
                //         }
                // };
                // *point = pivot + rotation * (*point - pivot);
                // }
                // if let Some(top) = self.quad.data.top {
                //     let pivot = Vec3::new(0.5 + x, 0.5 + y, 0.5 + z); // TO emulate how itll be getting from geometry
                //     let rotation = if top {
                //         Quat::from_euler(EulerRot::XYZ, 180.0_f32.to_radians(), 0.0, 0.0)
                //     } else {
                //         Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0)
                //     };
                //     *point = pivot + rotation * (*point - pivot);
                // }
            }
        }
        let mut final_arr = [
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        ];

        for (point_num, point) in temp_arr.iter_mut().enumerate() {
            final_arr[point_num] = Into::<[f32; 3]>::into(*point);
        }

        final_arr
    }

    pub fn normals(&self) -> [[f32; 3]; 4] {
        self.side.normals()
    }

    pub fn uvs<
        V: Voxel<R> + Clone + Serialize + Eq + Default + RenderedVoxel<V, R>,
        R: VoxRegistry<V> + Clone + Default,
    >(
        &self,
        asset_registry: &AssetRegistry,
        matched_ind: usize,
        _: mint::Vector3<i32>,
        chunk: &ChunkBoundary<V, R>,
    ) -> [[f32; 2]; 4] {
        if let Some(textures) = self.quad.data.textures {
            let texture_uv = textures[matched_ind];
            let geo = chunk
                .geometry_pal
                .palette
                .get(self.quad.data.geo_index.unwrap_or_default())
                .unwrap();
            let uv = geo.cubes.get(self.quad.cube).unwrap().uv;
            let mut face_tex = [[0.0; 2]; 4];
            let min_x = texture_uv.x;
            let min_y = texture_uv.y;
            let face_index = match (&self.side.axis, &self.side.positive) {
                (Axis::X, false) => 0,
                (Axis::X, true) => 1,
                (Axis::Y, false) => 2,
                (Axis::Y, true) => 3,
                (Axis::Z, false) => 4,
                (Axis::Z, true) => 5,
            };
            let (min_x, min_y) = (
                min_x + uv.get(face_index).unwrap().0 .0 as f32,
                min_y + uv.get(face_index).unwrap().0 .1 as f32,
            );
            let (max_x, max_y) = (
                min_x + uv.get(face_index).unwrap().1 .0 as f32,
                min_y + uv.get(face_index).unwrap().1 .1 as f32,
            );
            let (min_x, min_y, max_x, max_y) = (
                min_x / asset_registry.texture_size.x,
                min_y / asset_registry.texture_size.y,
                max_x / asset_registry.texture_size.x,
                max_y / asset_registry.texture_size.y,
            );

            face_tex[2][0] = min_x;
            face_tex[2][1] = min_y;
            face_tex[3][0] = max_x;
            face_tex[3][1] = min_y;
            face_tex[0][0] = min_x;
            face_tex[0][1] = max_y;
            face_tex[1][0] = max_x;
            face_tex[1][1] = max_y;
            // let flip_num = if self.quad.data.tex_variance[face_index] {
            //     let mut rng: StdRng = SeedableRng::seed_from_u64(world_pos.reflect_hash().unwrap());
            //     rng.gen_range(0..6)
            // } else {
            //     0
            // };
            // match flip_num {
            //     0 => {
            //         face_tex[2][0] = min_x;
            //         face_tex[2][1] = min_y;
            //         face_tex[3][0] = max_x;
            //         face_tex[3][1] = min_y;
            //         face_tex[0][0] = min_x;
            //         face_tex[0][1] = max_y;
            //         face_tex[1][0] = max_x;
            //         face_tex[1][1] = max_y;
            //     }
            //     1 => {
            //         face_tex[2][0] = max_x;
            //         face_tex[2][1] = max_y;
            //         face_tex[3][0] = min_x;
            //         face_tex[3][1] = max_y;
            //         face_tex[0][0] = max_x;
            //         face_tex[0][1] = min_y;
            //         face_tex[1][0] = min_x;
            //         face_tex[1][1] = min_y;
            //     }
            //     2 => {
            //         face_tex[2][0] = max_x;
            //         face_tex[2][1] = min_y;
            //         face_tex[3][0] = min_x;
            //         face_tex[3][1] = min_y;
            //         face_tex[0][0] = max_x;
            //         face_tex[0][1] = max_y;
            //         face_tex[1][0] = min_x;
            //         face_tex[1][1] = max_y;
            //     }
            //     3 => {
            //         face_tex[2][0] = min_x;
            //         face_tex[2][1] = max_y;
            //         face_tex[3][0] = max_x;
            //         face_tex[3][1] = max_y;
            //         face_tex[0][0] = min_x;
            //         face_tex[0][1] = min_y;
            //         face_tex[1][0] = max_x;
            //         face_tex[1][1] = min_y;
            //     }
            //     4 => {
            //         face_tex[2][0] = max_x;
            //         face_tex[2][1] = max_y;
            //         face_tex[3][0] = max_x;
            //         face_tex[3][1] = min_y;
            //         face_tex[0][0] = min_x;
            //         face_tex[0][1] = max_y;
            //         face_tex[1][0] = min_x;
            //         face_tex[1][1] = min_y;
            //     }
            //     5 => {
            //         face_tex[2][0] = min_x;
            //         face_tex[2][1] = min_y;
            //         face_tex[3][0] = min_x;
            //         face_tex[3][1] = max_y;
            //         face_tex[0][0] = max_x;
            //         face_tex[0][1] = min_y;
            //         face_tex[1][0] = max_x;
            //         face_tex[1][1] = max_y;
            //     }
            //     _ => {}
            // }
            return face_tex;
        }
        [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]]
    }

    pub fn voxel(&self) -> [usize; 3] {
        self.quad.voxel
    }
}

// Possibly have this just fully generate the mesh
pub fn generate_mesh<
    V: Voxel<R> + Clone + Serialize + Eq + Default + RenderedVoxel<V, R>,
    R: VoxRegistry<V> + Clone + Default,
>(
    chunk: &ChunkBoundary<V, R>,
    solid_pass: bool,
    buffer: &mut QuadGroups,
) {
    buffer.clear();
    for z in 1..ChunkBoundary::<V, R>::edge() - 1 {
        for y in 1..ChunkBoundary::<V, R>::edge() - 1 {
            for x in 1..ChunkBoundary::<V, R>::edge() - 1 {
                let voxel = chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y, z)];
                match voxel.visibility {
                    EMPTY => continue,
                    visibility => {
                        let neighbor_block = [
                            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x - 1, y, z)],
                            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x + 1, y, z)],
                            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y - 1, z)],
                            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y + 1, z)],
                            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y, z - 1)],
                            chunk.voxels()[ChunkBoundary::<V, R>::linearize(x, y, z + 1)],
                        ];
                        let geo = chunk
                            .geometry_pal
                            .palette
                            .get(voxel.geo_index.unwrap_or_default())
                            .unwrap();
                        for (cube_num, cube) in geo.cubes.iter().enumerate() {
                            for (i, neighbor) in neighbor_block.iter().enumerate() {
                                let culled = cube.cull[i];
                                if cube.discard[i] {
                                    continue;
                                }
                                let blocked = match i {
                                    0 => neighbor_block[i].blocks[1],
                                    1 => neighbor_block[i].blocks[0],
                                    2 => neighbor_block[i].blocks[3],
                                    3 => neighbor_block[i].blocks[2],
                                    4 => neighbor_block[i].blocks[5],
                                    5 => neighbor_block[i].blocks[4],
                                    _ => true,
                                };

                                let mut blocked_self = false;
                                if let Some(blocks) = neighbor_block[i].blocks_self {
                                    blocked_self = match i {
                                        0 => blocks[1],
                                        1 => blocks[0],
                                        2 => blocks[3],
                                        3 => blocks[2],
                                        4 => blocks[5],
                                        5 => blocks[4],
                                        _ => true,
                                    };
                                }

                                let other = neighbor.visibility;
                                let generate = if culled
                                    && (blocked
                                        || (neighbor.geo_index == voxel.geo_index && blocked_self))
                                {
                                    if solid_pass {
                                        match (visibility, other) {
                                            (OPAQUE, EMPTY) | (OPAQUE, TRANSPARENT) => true,

                                            (TRANSPARENT, TRANSPARENT) => {
                                                voxel.match_index != neighbor.match_index
                                            }

                                            (_, _) => false,
                                        }
                                    } else {
                                        match (visibility, other) {
                                            (TRANSPARENT, EMPTY) => true,

                                            (TRANSPARENT, TRANSPARENT) => {
                                                voxel.match_index != neighbor.match_index
                                            }

                                            (_, _) => false,
                                        }
                                    }
                                } else {
                                    (visibility == OPAQUE && solid_pass)
                                        || (visibility == TRANSPARENT && !solid_pass) && !blocked
                                };
                                let origin_one = match i {
                                    0 => cube.origin.1,
                                    1 => cube.origin.1,
                                    2 => cube.origin.0,
                                    3 => cube.origin.0,
                                    4 => cube.origin.0,
                                    5 => cube.origin.0,
                                    _ => 0,
                                };
                                let end_one = match i {
                                    0 => cube.end.1,
                                    1 => cube.end.1,
                                    2 => cube.end.0,
                                    3 => cube.end.0,
                                    4 => cube.end.0,
                                    5 => cube.end.0,
                                    _ => 0,
                                };
                                let origin_two = match i {
                                    0 => cube.origin.2,
                                    1 => cube.origin.2,
                                    2 => cube.origin.2,
                                    3 => cube.origin.2,
                                    4 => cube.origin.1,
                                    5 => cube.origin.1,
                                    _ => 0,
                                };
                                let end_two = match i {
                                    0 => cube.end.2,
                                    1 => cube.end.2,
                                    2 => cube.end.2,
                                    3 => cube.end.2,
                                    4 => cube.end.1,
                                    5 => cube.end.1,
                                    _ => 0,
                                };
                                let self_start = match i {
                                    0 => cube.origin.0,
                                    1 => cube.origin.0,
                                    2 => cube.origin.1,
                                    3 => cube.origin.1,
                                    4 => cube.origin.2,
                                    5 => cube.origin.2,
                                    _ => 0,
                                };
                                let self_end = match i {
                                    0 => cube.end.0,
                                    1 => cube.end.0,
                                    2 => cube.end.1,
                                    3 => cube.end.1,
                                    4 => cube.end.2,
                                    5 => cube.end.2,
                                    _ => 0,
                                };

                                if generate {
                                    buffer.groups[i].push(Quad {
                                        voxel: [x, y, z],
                                        start: (origin_one, origin_two, self_start),
                                        end: (end_one, end_two, self_end),
                                        cube: cube_num,
                                        data: voxel,
                                    });
                                }
                            }
                            //
                        }
                    }
                }
            }
        }
    }
    // buffer
}

pub fn full_mesh<
    V: Voxel<R> + Clone + Serialize + Eq + Default + RenderedVoxel<V, R>,
    R: VoxRegistry<V> + Clone + Default,
>(
    asset_registry: &AssetRegistry,
    raw_chunk: &ChunkBoundary<V, R>,
    chunk_pos: mint::Vector3<i32>,
) -> MeshedChunk {
    let mut buffer = QuadGroups::default();
    generate_mesh(raw_chunk, true, &mut buffer);
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut ao = Vec::new();
    let mut light = Vec::new();
    for face in buffer.iter_with_ao(raw_chunk) {
        indices.extend_from_slice(&face.indices(positions.len() as u32));
        positions.extend_from_slice(&face.positions(1.0, raw_chunk)); // Voxel size is 1m
        normals.extend_from_slice(&face.normals());
        ao.extend_from_slice(&face.aos());
        light.extend_from_slice(&face.light());
        let matched_index = match (face.side.axis, face.side.positive) {
            (Axis::X, false) => 2,
            (Axis::X, true) => 3,
            (Axis::Y, false) => 1,
            (Axis::Y, true) => 0,
            (Axis::Z, false) => 5,
            (Axis::Z, true) => 4,
        };
        // let _matched_neighbor = match (face.side.axis, face.side.positive) {
        //     (Axis::X, false) => (face.voxel()[0] - 1, face.voxel()[1], face.voxel()[2]),
        //     (Axis::X, true) => (face.voxel()[0] + 1, face.voxel()[1], face.voxel()[2]),
        //     (Axis::Y, false) => (face.voxel()[0], face.voxel()[1] - 1, face.voxel()[2]),
        //     (Axis::Y, true) => (face.voxel()[0], face.voxel()[1] + 1, face.voxel()[2]),
        //     (Axis::Z, false) => (face.voxel()[0], face.voxel()[1], face.voxel()[2] - 1),
        //     (Axis::Z, true) => (face.voxel()[0], face.voxel()[1], face.voxel()[2] + 1),
        // };

        uvs.extend_from_slice(
            &face.uvs(
                asset_registry,
                matched_index,
                VoxelPos::from(
                    mint::Vector3::<f32>::from(glam::Vec3::new(
                        face.voxel()[0] as f32,
                        face.voxel()[1] as f32,
                        face.voxel()[2] as f32,
                    )), // .into(),
                )
                .into(),
                // .as_vec3()
                // .into(),
                raw_chunk,
            ),
        );
    }
    let final_ao = ao_convert(ao);
    let mut final_color = Vec::new();
    for (idx, color) in final_ao.iter().enumerate() {
        final_color.extend_from_slice(&[[
            color[0] * light[idx],
            color[1] * light[idx],
            color[2] * light[idx],
            color[3],
        ]]);
    }

    let mesh = VoxMesh {
        vertices: positions.clone(),
        normals: normals.clone(),
        colors: Some(final_color.clone()),
        indices: indices.clone(),
        uvs: Some(uvs.clone()),
    };

    buffer.clear();

    //Transparent Mesh
    generate_mesh(raw_chunk, false, &mut buffer);
    let mut ao = Vec::new();
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for face in buffer.iter_with_ao(raw_chunk) {
        indices.extend_from_slice(&face.indices(positions.len() as u32));

        positions.extend_from_slice(&face.positions(1.0, raw_chunk)); // Voxel size is 1m
        normals.extend_from_slice(&face.normals());
        ao.extend_from_slice(&face.aos());
        let matched_index = match (face.side.axis, face.side.positive) {
            (Axis::X, false) => 2,
            (Axis::X, true) => 3,
            (Axis::Y, false) => 1,
            (Axis::Y, true) => 0,
            (Axis::Z, false) => 5,
            (Axis::Z, true) => 4,
        };

        uvs.extend_from_slice(
            &face.uvs(
                asset_registry,
                matched_index,
                VoxelPos::from(
                    mint::Vector3::<f32>::from(glam::Vec3::new(
                        face.voxel()[0] as f32,
                        face.voxel()[1] as f32,
                        face.voxel()[2] as f32,
                    )), // .into(),
                )
                .into(),
                raw_chunk,
            ),
        );
    }

    let transparent_mesh = VoxMesh {
        vertices: positions.clone(),
        normals: normals.clone(),
        colors: Some(final_color.clone()),
        indices: indices.clone(),
        uvs: Some(uvs.clone()),
    };

    MeshedChunk {
        chunk_mesh: mesh,
        transparent_mesh,
        pos: ChunkPos(chunk_pos),
    }
}

#[derive(Default)]
pub struct VoxMesh {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub colors: Option<Vec<[f32; 4]>>,
    pub uvs: Option<Vec<[f32; 2]>>,
}

pub struct MeshedChunk {
    pub chunk_mesh: VoxMesh,
    pub transparent_mesh: VoxMesh,
    pub pos: ChunkPos,
}

fn light_to_intern(color: u8) -> f32 {
    match color {
        0 => 0.25,
        // 0 => 0.0,
        1 => 0.25,
        2 => 0.25,
        3 => 0.4,
        4 => 0.4,
        5 => 0.5,
        6 => 0.5,
        7 => 0.5,
        8 => 0.5,
        9 => 0.5,
        10 => 0.6,
        11 => 0.7,
        12 => 0.75,
        13 => 0.8,
        14 => 0.9,
        15 => 1.0,
        _ => 1.0,
    }
}

fn ao_convert(ao: Vec<u32>) -> Vec<[f32; 4]> {
    let mut res = Vec::new();
    for value in ao {
        match value {
            0 => res.extend_from_slice(&[[0.1, 0.1, 0.1, 1.0]]),
            1 => res.extend_from_slice(&[[0.25, 0.25, 0.25, 1.0]]),
            2 => res.extend_from_slice(&[[0.5, 0.5, 0.5, 1.0]]),
            _ => res.extend_from_slice(&[[1., 1., 1., 1.0]]),
        }
    }
    res
}
