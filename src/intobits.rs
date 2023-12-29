use std::{mem::size_of, ops::{ShlAssign, BitAnd, Shl}, iter::{Repeat, Take}, process::Output};

pub struct IntoBitsIterator<T> {
    acc: T,
    remains: usize,
}

impl <T:Shl<T, Output=T> + BitAnd<T, Output=T> + PartialEq<T> + From<u8> + Copy> IntoBitsIterator<T> {
    pub fn new(v: T) -> Self {
        Self {
            acc: v,
            remains: size_of::<T>() * 8,
        }
    }
}

impl <T:Shl<T, Output=T> + BitAnd<T, Output=T> + PartialEq<T> + From<u8> + Copy> Iterator for IntoBitsIterator<T> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remains == 0 {
            return None
        } else {
            let mask = Into::<T>::into(1) << Into::<T>::into(size_of::<T>() as u8 * 8 - 1);
            let ret = (self.acc & mask) != Into::<T>::into(0);
            self.acc = self.acc << Into::<T>::into(1);
            self.remains -= 1;
            Some(if ret { 1 } else { 0 })
        }
    }
}

pub struct TypeIntoBitIteratorWrapper<T> {
    val: T
}

impl <T> TypeIntoBitIteratorWrapper<T> {
    pub fn new(val: T) -> Self{
        Self { val }
    }
}

impl IntoIterator for TypeIntoBitIteratorWrapper<u8> {
    type Item = u8;

    type IntoIter = IntoBitsIterator<u8>;

    fn into_iter(self) -> Self::IntoIter {
        IntoBitsIterator::new(self.val)
    }
}

pub struct RepeatNWrapper<T> {
    val: T,
    count: usize,
}

impl <T> RepeatNWrapper<T> {
    pub fn new(val: T, count: usize) -> Self{
        Self { val, count }
    }
}

impl IntoIterator for RepeatNWrapper<u8> {
    type Item = u8;

    type IntoIter = Take<Repeat<u8>>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::repeat(self.val).take(self.count)
    }
}