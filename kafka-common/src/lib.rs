use std::io::Cursor;

use murmur3::murmur3_32;
use uuid::Uuid;

pub fn partition_of(identifier: Uuid, num_partitions: i32) -> std::io::Result<i32> {
    Ok(
        murmur3_32(&mut Cursor::new(identifier.as_bytes()), 0)?.rem_euclid(num_partitions as u32)
            as i32,
    )
}
