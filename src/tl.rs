use super::*;
use std::ops::Deref;
use std::sync::Arc;

pub struct Tl<T> {
    // TODO Retry TrustRc (simple Rc inside) when possible
    cell: Arc<TrustCell<T>>,
}

// impl<T> Drop for Tl<T> {
//     fn drop(&mut self) {
//         println!("Drop Tl with strong = {}", Arc::strong_count(&self.cell));
//     }
// }

impl<T> Clone for Tl<T> {
    fn clone(&self) -> Self {
        Self {
            cell: self.cell.clone(),
        }
    }
}

impl<T> Deref for Tl<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.cell.get(thread_index())
    }
}

impl<T: 'static + ManualCopy<T>> Tl<T> {
    pub fn to_mut(&self) -> &mut T {
        // TODO Dev check if caller come from different places
        // even in different sync calls, then should panic

        {
            let d = get_dirties().to_mut(thread_index());
            let tmp = Box::new(self.clone());
            let ptr = tmp.cell.arr.get();
            let mut is_unique = true;

            for it in d.iter_mut() {
                if it.1.get_ptr() == ptr as usize {
                    if it.0 == 1 {
                        panic!("Only allow one mutation each sync");
                    } else {
                        it.0 = 1;
                    }

                    is_unique = false;
                    break;
                }
            }

            if is_unique {
                d.push((1, tmp));
            }
        }

        self.cell.to_mut(MUTATE_THREAD_INDEX)
    }
}

impl<T: 'static + ManualCopy<T>> Dirty for Tl<T> {
    fn sync(&self, from: usize, to: usize) {
        self.cell.inner_manual_copy(from, to);
    }

    fn get_ptr(&self) -> usize {
        self.cell.arr.get() as usize
    }

    fn register_listener(&self, f: Box<Fn()>) -> ListenerHandleRef {
        let l = get_listeners().to_mut(thread_index());
        let ptr = self.get_ptr();
        if !l.contains_key(&ptr) {
            l.insert(ptr, vec![]);
        }

        let l = l.get_mut(&ptr).unwrap();
        let h = ListenerHandle { ptr };
        l.push((h, f));

        ListenerHandleRef {
            handle: &l[l.len() - 1].0,
        }
    }

    fn re_add(&self) {
        self.to_mut();
    }
}

impl<T: Default + Clone + ManualCopy<T>> Default for Tl<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone> Tl<T> {
    pub fn new(value: T) -> Self {
        // TODO Find a way that flexible with thread,
        // but also not using std::mem::zeroed (error with Rc)
        let tmp1 = value.clone();
        let a = [value, tmp1];
        // let tmp2 = value.clone();
        // let a = [value, tmp1, tmp2];

        Self {
            cell: Arc::new(TrustCell::new(a)),
        }
    }
}
