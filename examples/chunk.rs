use serde::{Deserialize, Serialize};
use vinox_voxel::prelude::*;

fn main() {
    let mut registry = BlockRegistry::default();
    registry.0.insert(
        "voxel:test".to_string(),
        Block {
            identifier: "voxel:test".to_string(),
            visibility: Some(VoxelVisibility::Opaque),
            ..Default::default()
        },
    );
    let og_chunk = ChunkData::<BlockData, BlockRegistry>::default();

    let encoded: Vec<u8> = bincode::serialize(&og_chunk).unwrap();
    let mut decoded: ChunkData<BlockData, BlockRegistry> =
        bincode::deserialize(&encoded[..]).unwrap();
    decoded.set(
        RelativeVoxelPos::new(0, 0, 0),
        BlockData {
            identifier: "voxel:test".to_string(),
            last_tick: None,
            properties: None,
        },
    );
    // assert_eq!(
    println!(
        "{}{}",
        og_chunk.is_empty(Some(&registry)),
        decoded.is_empty(Some(&registry))
    );
    // );
    println!(
        "unser: {:?}, de: {:?}",
        og_chunk.get(RelativeVoxelPos::new(0, 0, 0)),
        decoded.get(RelativeVoxelPos::new(0, 0, 0)),
    );
}
