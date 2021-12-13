use x86_64::{
    structures::paging::PageTable,
    // VirtAddr,
};
// 4 KiB page
#[repr(align(4096))]
pub struct Page {
	pub data: [PageTable; 512]
}