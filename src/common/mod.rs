pub mod bitmap;
pub mod consts;
pub mod error;

pub mod utility {
    pub fn hexdump(data: &[u8]) {
        use alloc::prelude::*;
        let mut index: usize = 0;
        for bytes in data.chunks(16) {
            println!(
                "{:08x}: {}",
                index,
                bytes
                    .iter()
                    .map(|x| format!("{:02x}", x))
                    .collect::<Vec<String>>()
                    .join(" ")
            );
            index += 0x10;
        }
    }
}
