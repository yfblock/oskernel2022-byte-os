pub mod fat32;
pub mod file;
mod partition;
pub mod filetree;

pub use partition::Partition;
// pub use partition::get_partitions;

pub fn init() {
    fat32::init();
}