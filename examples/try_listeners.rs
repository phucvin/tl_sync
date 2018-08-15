extern crate tl_sync;

use std::cell::RefCell;
use std::rc::Rc;
use tl_sync::*;

#[derive(Default, Clone)]
struct Emitter {
    l: Rc<RefCell<Vec<Box<Fn()>>>>,
}

impl Emitter {
    fn add_listener(&self, f: Box<Fn()>) {
        let mut l = self.l.borrow_mut();

        l.push(f);
    }

    fn notify(&self) {
        let l = self.l.borrow();

        println!("notify to {} listeners", l.len());
        l.iter().for_each(|it| it());
    }
}

#[derive(Default, Clone)]
struct Element {
    on_click: Emitter,
}

impl Drop for Element {
    fn drop(&mut self) {
        println!("\t\tDROP Element");
    }
}

struct Screen {
    elements: Wrc<RefCell<Vec<Element>>>,
    data: Wrc<RefCell<Vec<u8>>>,
}

impl Screen {
    fn clone_weak(&self) -> Self {
        Self {
            elements: self.elements.clone_weak(),
            data: self.data.clone_weak(),
        }
    }

    fn make_strong(&self) -> Self {
        Self {
            elements: self.elements.make_strong(),
            data: self.data.make_strong(),
        }
    }

    fn setup(&self) {
        let mut elements = self.elements.borrow_mut();

        elements.push(Default::default());
        let ref e = elements[0];

        {
            let this = self.clone_weak();

            e.on_click.add_listener(Box::new(move || {
                let this = this.make_strong();
                this.layout();
            }));
        }

        {
            let this = self.clone_weak();

            e.on_click.add_listener(Box::new(move || {
                let this = this.make_strong();
                this.animation();
            }));
        }
    }

    fn animation(&self) {
        let elements = self.elements.borrow();
        let data = self.data.borrow();

        println!(
            "animation for {} elements with data {:?}",
            elements.len(),
            data
        );
    }

    fn layout(&self) {
        let mut elements = self.elements.borrow_mut();
        let mut data = self.data.borrow_mut();

        elements.push(Default::default());
        for it in data.iter_mut() {
            *it *= 3;
        }
        println!("layout");
    }

    fn notify_all(&self) {
        let elements = self.elements.borrow().clone();

        elements.iter().for_each(|it| it.on_click.notify());
    }
}

fn main() {
    let screen = Screen {
        elements: Wrc::new(RefCell::new(vec![])),
        data: Wrc::new(RefCell::new(vec![1, 2, 3, 4, 5])),
    };

    screen.setup();
    screen.notify_all();
    println!("--");
}
