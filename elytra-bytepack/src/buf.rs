use core::{ops::{Add, AddAssign, Deref}, usize};

pub struct Buf<const N: usize> {
    inner: [u8; N],
    position: usize,
}

macro_rules! impl_buf_add_all {
    ( $a:literal, $( $b:tt )* ) => {
        impl_buf_add!($a => $($b)*);
        impl_buf_add_all!($($b)*);
    };
    ( $a:literal ) => {
        impl_buf_add!($a);
    };
}

macro_rules! impl_buf_add {
    ($a:literal => $( $b:literal ),+ ) => {
        impl_buf_add!($a);
        $( impl_buf_add!($a, $b); )*
    };
    ($a:literal) => {
        impl Add<Buf<$a>> for Buf<$a> where 
        {
            type Output = Buf<{$a + $a}>;
            fn add(self, rhs: Buf<$a>) -> Self::Output {
                Self::Output::from_parts(self, rhs)
            }
        }
    };
    ($a:literal, $b:literal) => {
        impl Add<Buf<$b>> for Buf<$a> where 
        {
            type Output = Buf<{$a + $b}>;
            fn add(self, rhs: Buf<$b>) -> Self::Output {
                Self::Output::from_parts(self, rhs)
            }
        }
        impl Add<Buf<$a>> for Buf<$b> where 
        {
            type Output = Buf<{$b + $a}>;
            fn add(self, rhs: Buf<$a>) -> Self::Output {
                Self::Output::from_parts(self, rhs)
            }
        }
    };
}

impl_buf_add_all!(1, 2, 3, 4, 5, 6, 7, 8);


impl <const N: usize> Add<Buf<N>> for Buf<0> {
    type Output = Buf<N>;
    fn add(self, rhs: Buf<N>) -> Self::Output {
        rhs
    }
}

impl <const N: usize, const A: usize> AddAssign<[u8; A]> for Buf<N> {
    fn add_assign(&mut self, rhs: [u8; A]) {
        self.append(rhs);
    }
}

impl <const N: usize> Deref for Buf<N> {
    type Target = [u8; N];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl <const N: usize> From<[u8; N]> for Buf<N> {
    fn from(inner: [u8; N]) -> Self {
        Self {
            inner,
            position: inner.len()
        }
    }
}

impl <const N: usize> Buf<N> {
    pub fn new() -> Self {
        Self {
            inner: [0; N],
            position: 0,
        }
    }

    fn from_parts<const A: usize, const B: usize>(a: Buf<A>, b: Buf<B>) -> Self {
        let mut buf = Self::new();
        let _ = buf.append(*a);
        let _ = buf.append(*b);
        buf
    }

    pub fn remaining_slice_mut(&mut self) -> &mut [u8] {
        let len = self.position.min(self.inner.as_mut().len());
        &mut self.inner.as_mut()[len..]
    }

    pub fn write(&mut self, data: &[u8]) -> usize {
        let slice = self.remaining_slice_mut();
        let amt = data.len().min(slice.len());

        slice[..amt].copy_from_slice(&data[..amt]);

        self.position += amt;
        
        amt
    }

    pub fn append<const A: usize>(&mut self, data: [u8; A]) -> usize {
        self.write(&data)
    }
    
    pub fn into_bytes(self) -> [u8; N] {
        self.inner
    }
}

#[macro_export]
macro_rules! buf {
    (_: $b:expr) => {
        Buf::from($b)
    };
    ($n:literal: $b:expr) => {
        Buf::<$n>::from($b)
    };
    ($n:literal) => {
        Buf::<$n>::new()
    };
    ($b:expr) => {
        Buf::from($b)
    };
}

