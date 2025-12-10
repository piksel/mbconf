/// Inline "fork" of [nostd_cursor](https://github.com/davidtraff/nostd-cursor/).
#[allow(unused)]

#[derive(PartialEq, Debug)]
    pub enum Error {
        UnexpectedEof
    }
    type Result<T> = core::result::Result<T, Error>;

    pub struct Cursor<T> {
        inner: T,
        position: usize,
    }

    impl<T> Cursor<T> {
        pub fn new(inner: T) -> Self {
            Self {
                inner,
                position: 0,
            }
        }

        pub fn position(&self) -> usize {
            self.position
        }

        pub fn set_position(&mut self, position: usize) {
            self.position = position;
        }

        pub fn get_ref(&self) -> &T {
            &self.inner
        }
    }

    impl<T: AsRef<[u8]>> Cursor<T> {
        pub fn remaining_slice(&self) -> &[u8] {
            let len = self.position.min(self.inner.as_ref().len());

            &self.inner.as_ref()[len..]
        }

        pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            let slice = self.remaining_slice();
            let amt = buf.len().min(slice.len());
            let a = &slice[..amt];

            if amt == 1 {
                buf[0] = a[0];
            } else {
                buf[..amt].copy_from_slice(a);
            }

            self.position += amt;
            
            Ok(amt)
        }

        pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
            let slice = self.remaining_slice();
            if buf.len() > slice.len() {
                return Err(Error::UnexpectedEof);
            }
            let a = &slice[..buf.len()];

            if buf.len() == 1 {
                buf[0] = a[0];
            } else {
                buf.copy_from_slice(a);
            }

            self.position += buf.len();

            Ok(())
        }
    }

    impl Cursor<&mut [u8]> {
        pub fn remaining_slice_mut(&mut self) -> &mut [u8] {
            let len = self.position.min(self.inner.as_mut().len());

            &mut self.inner.as_mut()[len..]
        }

        pub fn write(&mut self, data: &[u8]) -> Result<usize> {
            let slice = self.remaining_slice_mut();
            let amt = data.len().min(slice.len());

            slice[..amt].copy_from_slice(&data[..amt]);

            self.position += amt;
            
            Ok(amt)
        }
    }

    impl<const N: usize> Cursor<[u8; N]> {
        pub fn remaining_slice_mut(&mut self) -> &mut [u8] {
            let len = self.position.min(self.inner.as_mut().len());

            &mut self.inner.as_mut()[len..]
        }

        pub fn write(&mut self, data: &[u8]) -> Result<usize> {
            let slice = self.remaining_slice_mut();
            let amt = data.len().min(slice.len());

            slice[..amt].copy_from_slice(&data[..amt]);

            self.position += amt;
            
            Ok(amt)
        }
    }

    impl<T: AsRef<[u8]>> AsRef<T> for Cursor<T> {
        fn as_ref(&self) -> &T {
            &self.inner
        }
    }