use std::io::Read;

pub struct ReadableVec<'a, T> {
    pub vector: &'a mut Vec<T>,
}

impl<T: Clone> Read for ReadableVec<'_, T> where u8: From<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut added = 0;
        if self.vector.len() < buf.len() {
            return Err(std::io::Error::other("vector too short"));
        }
        while added <= buf.len() - 1 {
            buf[added] = self.vector[0].clone().into();
            self.vector.remove(0);
            added += 1;
        };

        return Ok(buf.len());
    }
}