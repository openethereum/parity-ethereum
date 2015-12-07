pub trait Reader<'a, 'view>: Sized {
	type Prototype;
	type PayloadInfo;
	type Data;
	type Item;

	fn new(bytes: &'a [u8]) -> Self;
	fn raw(&'view self) -> &'a [u8];	
	fn prototype(&self) -> Self::Prototype;
	fn payload_info(&self) -> Self::PayloadInfo;
	fn data(&'view self) -> Self::Data;
	fn item_count(&self) -> usize;
	fn size(&self) -> usize;
	fn at(&'view self, index: usize) -> Self::Item;
	fn is_null(&self) -> bool;
	fn is_empty(&self) -> bool;
	fn is_list(&self) -> bool;
	fn is_data(&self) -> bool;
	fn is_int(&self) -> bool;
}

pub trait Stream {
}
