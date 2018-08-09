use std::cmp;

pub trait ManualCopy<T> {
    fn copy_from(&mut self, &T);
}

impl ManualCopy<usize> for usize {
    fn copy_from(&mut self, other: &usize) {
        *self = *other;
    }
}

impl ManualCopy<String> for String {
    fn copy_from(&mut self, other: &String) {
        self.clear();
        self.push_str(other);
    }
}

impl<T: Clone> ManualCopy<Option<T>> for Option<T> {
    fn copy_from(&mut self, other: &Option<T>) {
        *self = match *other {
            None => None,
            Some(ref v) => Some(v.clone()),
        }
    }
}

impl<T1: Clone, T2: Clone> ManualCopy<(T1, T2)> for (T1, T2) {
    fn copy_from(&mut self, other: &(T1, T2)) {
        // TODO If U: copy, try to use memcpy (=)
        self.0 = other.0.clone();
        self.1 = other.1.clone();
    }
}

impl<U: Clone> ManualCopy<Vec<U>> for Vec<U> {
    fn copy_from(&mut self, other: &Vec<U>) {
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
    }
}
