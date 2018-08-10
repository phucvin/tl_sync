extern crate tl_sync;

use std::sync::Arc;
use std::thread;
use std::time;
use tl_sync::*;

#[derive(Clone)]
struct SceneRoot {
    stack: Tl<Vec<Scene>>,
    popup: Tl<Option<Scene>>,
    top_ui: Tl<Button>,
}

impl Default for SceneRoot {
    fn default() -> Self {
        Self {
            stack: Default::default(),
            popup: Tl::new(None),
            top_ui: Tl::new(Default::default()),
        }
    }
}

impl ManualCopy<SceneRoot> for SceneRoot {
    fn copy_from(&mut self, _other: &SceneRoot) {
        panic!("SHOULD NEVER BE CALLED");
    }
}

#[derive(Default, Clone)]
struct Scene {
    title: Tl<String>,
    buttons: Tl<Vec<Button>>,
    image_data: Arc<Vec<u8>>,
}

#[derive(Default, Clone)]
struct Button {
    pos: Tl<(u32, u32)>,
    txt: Tl<String>,
}

fn main() {
    init_dirties();
    {
        let r = Arc::new(SceneRoot::default());

        r.stack.to_mut().push(Scene {
            title: Tl::new("Home".into()),
            buttons: Tl::new(vec![Button {
                pos: Tl::new((100, 50)),
                txt: Tl::new("Click Me!".into()),
            }]),
            image_data: Arc::new(vec![8; 1024]),
        });

        sync_from(2);
        sync_to(1);
        println!(
            "{}: {} @ {:?}",
            *r.stack[0].title, *r.stack[0].buttons[0].txt, *r.stack[0].buttons[0].pos
        );

        let handle = {
            let r = r.clone();
            thread::Builder::new()
                .name("1_test".into())
                .spawn(move || {
                    for _ in 1..10 {
                        {
                            let tmp = &r.stack[0].buttons[0];
                            *tmp.txt.to_mut() = "Play".into();
                            *tmp.pos.to_mut() = (tmp.pos.0 - 2, tmp.pos.1 + 3);
                        }

                        sync_from(2);
                    }

                    thread::sleep(time::Duration::from_millis(10));
                    sync_to(0);
                }).unwrap()
        };

        thread::sleep(time::Duration::from_millis(10));
        handle.join().unwrap();
        println!(
            "{}: {} @ {:?}",
            *r.stack[0].title, *r.stack[0].buttons[0].txt, *r.stack[0].buttons[0].pos
        );
    }
    drop_dirties();
}
