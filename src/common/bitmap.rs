use alloc::vec::Vec;

/// Simple bitmap
#[derive(Clone)]
pub struct Bitmap {
    length: usize,
    storage: Vec<u8>,
}

impl Bitmap {
    pub fn new(size: usize) -> Self {
        let mut vec: Vec<u8> = Vec::new();
        vec.resize((size / 8) + 1, 0);
        Self {
            length: size,
            storage: vec,
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn get(&self, i: usize) -> bool {
        if i >= self.length {
            panic!("get() with a larger size than bitmap's size");
        }
        let idx = i >> 3;
        let bitoff = i & 7;
        match self.storage[idx] & (1u8 << bitoff) {
            0 => false,
            _ => true,
        }
    }

    fn modify(&mut self, i: usize, set: bool) {
        if i >= self.length {
            panic!("modify() with a larger size than bitmap's size");
        }
        let idx = i >> 3;
        let bitoff = i & 7;
        match set {
            false => {
                self.storage[idx] &= !(1u8 << bitoff);
            }
            true => {
                self.storage[idx] |= 1u8 << bitoff;
            }
        };
    }

    pub fn set(&mut self, i: usize) {
        self.modify(i, true);
    }

    pub fn clear(&mut self, i: usize) {
        self.modify(i, false);
    }

    pub fn set_direct(&mut self, data: &[u8], offset: usize) {
        for i in offset..self.storage.len() {
            if i - offset >= data.len() {
                break;
            }
            self.storage[i] = data[i - offset];
        }
    }
}

#[cfg(test)]
fn test() {
    let mut bmp = Bitmap::new(100);

    bmp.set(9);
    assert_eq!(bmp.get(9), true);
    bmp.clear(9);
    assert_eq!(bmp.get(9), false);

    bmp.set_direct(&[0xff], 0);
    assert_eq!(bmp.get(0), true);
    bmp.set_direct(&[0xff], 2);
    assert_eq!(bmp.get(16), true);
    assert_eq!(bmp.get(8), false);
    bmp.set_direct(&[0xf0], 3);
    assert_eq!(bmp.get(24), false);
}
