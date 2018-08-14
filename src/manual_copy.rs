use std::cmp;
use std::time::Instant;
// use rayon::prelude::*;

pub trait ManualCopy<T> {
    fn copy_from(&mut self, &mut T);
}

impl ManualCopy<u8> for u8 {
    fn copy_from(&mut self, other: &mut u8) {
        *self = *other;
    }
}

impl ManualCopy<u64> for u64 {
    fn copy_from(&mut self, other: &mut u64) {
        *self = *other;
    }
}

impl ManualCopy<usize> for usize {
    fn copy_from(&mut self, other: &mut usize) {
        *self = *other;
    }
}

impl ManualCopy<isize> for isize {
    fn copy_from(&mut self, other: &mut isize) {
        *self = *other;
    }
}

impl ManualCopy<Instant> for Instant {
    fn copy_from(&mut self, other: &mut Instant) {
        *self = *other;
    }
}

impl ManualCopy<String> for String {
    fn copy_from(&mut self, other: &mut String) {
        self.clear();
        self.push_str(other);
    }
}

impl<T: Clone> ManualCopy<Option<T>> for Option<T> {
    fn copy_from(&mut self, other: &mut Option<T>) {
        *self = match *other {
            None => None,
            Some(ref v) => Some(v.clone()),
        }
    }
}

impl<T1: Clone, T2: Clone> ManualCopy<(T1, T2)> for (T1, T2) {
    fn copy_from(&mut self, other: &mut (T1, T2)) {
        // TODO If U: copy, try to use memcpy (=)
        self.0 = other.0.clone();
        self.1 = other.1.clone();
    }
}

impl<U: Send + Sync + Clone> ManualCopy<Vec<U>> for Vec<U> {
    fn copy_from(&mut self, other: &mut Vec<U>) {
        // TODO If U: Copy, try to use memcpy (copy_from_slice)
        let slen = self.len();
        let olen = other.len();

        if slen < olen {
            for i in slen..olen {
                self.push(other[i].clone());
            }
        } else if slen > olen {
            self.truncate(olen)
        }

        let min_len = cmp::min(slen, olen);
        for i in 0..min_len {
            self[i] = other[i].clone();
        }
        // TODO Should use parallel to sync, if faster than single thread
        // self.as_mut_slice()[..min_len].par_iter_mut().enumerate().for_each(|(i, it)| {
        //     *it = other[i].clone();
        // });
    }
}
