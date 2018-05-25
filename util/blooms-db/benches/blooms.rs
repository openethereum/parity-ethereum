#![feature(test)]

extern crate test;
extern crate tempdir;
extern crate blooms_db;
extern crate ethbloom;

use std::iter;
use test::Bencher;
use tempdir::TempDir;
use blooms_db::Database;
use ethbloom::Bloom;

#[bench]
fn blooms_filter_1_million(b: &mut Bencher) {
	let tempdir = TempDir::new("").unwrap();
	let mut database = Database::open(tempdir.path()).unwrap();
	database.insert_blooms(999_999, iter::once(&Bloom::from(0))).unwrap();
	let bloom = Bloom::from(0x001);
	database.insert_blooms(200_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(400_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(600_000, iter::once(&bloom)).unwrap();
	database.insert_blooms(800_000, iter::once(&bloom)).unwrap();
	database.flush().unwrap();

	b.iter(|| {
		let matches = database.iterate_matching(0, 999_999, &bloom).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
		assert_eq!(matches, vec![200_000, 400_000, 600_000, 800_000]);
	});
}
