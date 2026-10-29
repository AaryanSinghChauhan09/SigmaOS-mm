//! Buddy allocator — power-of-two block allocator for SigmaOS.
//! Principles: Manual Memory Control, Cache Conscious Design,
//! Zero-One-Infinity (any number of blocks), Memory Discipline.
use std::collections::BTreeMap;

pub struct BuddyAllocator {
    base: usize,
    total_size: usize,
    min_block: usize,
    free_lists: BTreeMap<usize, Vec<usize>>, // size -> list of free block starts
}

impl BuddyAllocator {
    pub fn new(base: usize, total_size: usize, min_block: usize) -> Self {
        assert!(total_size.is_power_of_two(), "total_size must be power of two");
        assert!(min_block.is_power_of_two(), "min_block must be power of two");
        assert!(min_block <= total_size);
        let mut free_lists = BTreeMap::new();
        free_lists.insert(total_size, vec![base]);
        Self { base, total_size, min_block, free_lists }
    }

    pub fn allocate(&mut self, size: usize) -> Option<usize> {
        let size = size.max(self.min_block).next_power_of_two();
        // Find the smallest available block >= size
        let block_size = *self.free_lists.range(size..).find(|(_, v)| !v.is_empty())?.0;
        let mut addr = self.free_lists.get_mut(&block_size)?.pop()?;
        // Split down to requested size
        let mut current = block_size;
        while current > size {
            current /= 2;
            let buddy = addr + current;
            self.free_lists.entry(current).or_default().push(buddy);
        }
        Some(addr)
    }

    pub fn free(&mut self, addr: usize, size: usize) {
        let mut size = size.max(self.min_block).next_power_of_two();
        let mut addr = addr;
        // Coalesce with buddy
        loop {
            let buddy = addr ^ size;
            if size >= self.total_size { break; }
            let list = self.free_lists.entry(size).or_default();
            if let Some(pos) = list.iter().position(|&a| a == buddy) {
                list.remove(pos);
                addr = addr.min(buddy);
                size *= 2;
            } else {
                break;
            }
        }
        self.free_lists.entry(size).or_default().push(addr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_returns_address() {
        // Arrange
        let mut alloc = BuddyAllocator::new(0, 4096, 64);
        // Act
        let addr = alloc.allocate(64);
        // Assert
        assert!(addr.is_some());
    }

    #[test]
    fn test_free_after_allocate() {
        // Arrange
        let mut alloc = BuddyAllocator::new(0, 4096, 64);
        let addr = alloc.allocate(64).unwrap();
        // Act — free the block
        alloc.free(addr, 64);
        // Assert — can allocate again
        let addr2 = alloc.allocate(64);
        assert!(addr2.is_some());
    }

    #[test]
    fn test_multiple_allocations() {
        // Arrange
        let mut alloc = BuddyAllocator::new(0, 4096, 64);
        // Act — allocate 4 blocks of 512 each = 2048 bytes
        let addrs: Vec<_> = (0..4).filter_map(|_| alloc.allocate(512)).collect();
        // Assert — all distinct addresses
        assert_eq!(addrs.len(), 4);
        let unique: std::collections::HashSet<_> = addrs.iter().collect();
        assert_eq!(unique.len(), 4);
    }
}
