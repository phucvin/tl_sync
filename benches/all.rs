#![feature(test)]

extern crate rayon;
extern crate test;
extern crate tl_sync;

use rayon::prelude::*;
use test::Bencher;
use tl_sync::*;

#[bench]
fn sync_1mb_and_10k_100bytes(bencher: &mut Bencher) {
    init_dirties();

    let a: Tl<Vec<u8>> = Tl::new(vec![1; 1024 * 1024]);
    let mut b: Vec<Tl<Vec<u8>>> = vec![];
    for _i in 1..100 {
        b.push(Tl::new(vec![1; 1000 * 100]));
    }

    bencher.iter(|| {
        a.sync(1, 0);
        b.par_iter().for_each(|it| it.sync(1, 0));
    });
}
