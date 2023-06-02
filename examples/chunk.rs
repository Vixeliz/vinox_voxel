use serde::{Deserialize, Serialize};
use vinox_voxel::prelude::*;

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, Debug)]
pub struct Block {
    identifier: String,
}

impl Voxel for Block {
    fn is_empty(&self) -> bool {
        false
    }

    fn is_true_empty(&self) -> bool {
        false
    }

    fn is_opaque(&self) -> bool {
        false
    }

    fn identifier(&self) -> String {
        self.identifier.clone()
    }
}

fn main() {
    let og_chunk = ChunkData::<Block>::default();

    let encoded: Vec<u8> = bincode::serialize(&og_chunk).unwrap();
    let mut decoded: ChunkData<Block> = bincode::deserialize(&encoded[..]).unwrap();
    decoded.set(
        RelativeVoxelPos::new(0, 0, 0),
        Block {
            identifier: "Test".to_string(),
        },
    );
    assert_eq!(og_chunk.is_empty(), decoded.is_empty());
    println!(
        "unser: {:?}, de: {:?}",
        og_chunk.get(RelativeVoxelPos::new(0, 0, 0)),
        decoded.get(RelativeVoxelPos::new(0, 0, 0)),
    );
}
