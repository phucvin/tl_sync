#[cfg(test)]
mod simples {
	use test::Bencher;
	use rayon::prelude::*;
	use super::super::*;

	#[bench]
	fn case01(bencher: &mut Bencher) {
		init_dirties();

		let a: Tl<Vec<u8>> = Tl::new(vec![1; 1024 * 1024]);
	    let mut b: Vec<Tl<Vec<u8>>> = vec![];
	    for _i in 1..100 {
	        b.push(Tl::new(vec![1; 1024 * 100]));
	    }

	    bencher.iter(|| {
	        a.sync(1, 0);
	        b.par_iter().for_each(|it| it.sync(1, 0));
	    });
	}
}