//! Virtual memory region tracker.
use std::fmt;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protection {
    None,
    Read,
    ReadWrite,
    ReadExec,
    ReadWriteExec,
}

impl fmt::Display for Protection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Protection::None         => "---",
            Protection::Read         => "r--",
            Protection::ReadWrite    => "rw-",
            Protection::ReadExec     => "r-x",
            Protection::ReadWriteExec=> "rwx",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone)]
pub struct MemRegion {
    pub start: usize,
    pub end: usize,
    pub prot: Protection,
    pub label: String,
}

impl MemRegion {
    pub fn new(start: usize, end: usize, prot: Protection, label: impl Into<String>) -> Self {
        assert!(start < end, "region start must be less than end");
        Self { start, end, prot, label: label.into() }
    }
    pub fn size(&self) -> usize { self.end - self.start }
    pub fn contains(&self, addr: usize) -> bool { addr >= self.start && addr < self.end }
    pub fn overlaps(&self, other: &MemRegion) -> bool {
        self.start < other.end && self.end > other.start
    }
}

impl fmt::Display for MemRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#010x}-{:#010x} {} {}", self.start, self.end, self.prot, self.label)
    }
}

/// Virtual address space — tracks mapped regions.
#[derive(Default)]
pub struct AddressSpace {
    regions: BTreeMap<usize, MemRegion>, // keyed by start address
}

impl AddressSpace {
    pub fn new() -> Self { Self::default() }

    pub fn map(&mut self, region: MemRegion) -> Result<(), String> {
        for existing in self.regions.values() {
            if existing.overlaps(&region) {
                return Err(format!("overlap with existing region {existing}"));
            }
        }
        self.regions.insert(region.start, region);
        Ok(())
    }

    pub fn unmap(&mut self, start: usize) -> Option<MemRegion> {
        self.regions.remove(&start)
    }

    pub fn find(&self, addr: usize) -> Option<&MemRegion> {
        self.regions.values().find(|r| r.contains(addr))
    }

    pub fn total_mapped(&self) -> usize {
        self.regions.values().map(|r| r.size()).sum()
    }

    pub fn region_count(&self) -> usize { self.regions.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_region() {
        // Arrange
        let mut space = AddressSpace::new();
        let region = MemRegion::new(0x1000, 0x2000, Protection::ReadWrite, "heap");
        // Act
        space.map(region).unwrap();
        // Assert
        assert_eq!(space.region_count(), 1);
    }

    #[test]
    fn test_overlapping_map_fails() {
        // Arrange
        let mut space = AddressSpace::new();
        space.map(MemRegion::new(0x1000, 0x3000, Protection::Read, "a")).unwrap();
        let overlap = MemRegion::new(0x2000, 0x4000, Protection::Read, "b");
        // Act
        let result = space.map(overlap);
        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_find_by_address() {
        // Arrange
        let mut space = AddressSpace::new();
        space.map(MemRegion::new(0x1000, 0x2000, Protection::Read, "text")).unwrap();
        // Act
        let found = space.find(0x1500);
        // Assert
        assert!(found.is_some());
        assert_eq!(found.unwrap().label, "text");
    }

    #[test]
    fn test_find_outside_region_is_none() {
        // Arrange
        let mut space = AddressSpace::new();
        space.map(MemRegion::new(0x1000, 0x2000, Protection::Read, "text")).unwrap();
        // Act / Assert
        assert!(space.find(0x3000).is_none());
    }

    #[test]
    fn test_total_mapped_size() {
        // Arrange
        let mut space = AddressSpace::new();
        space.map(MemRegion::new(0x0000, 0x1000, Protection::Read, "a")).unwrap();
        space.map(MemRegion::new(0x2000, 0x4000, Protection::ReadWrite, "b")).unwrap();
        // Assert
        assert_eq!(space.total_mapped(), 0x1000 + 0x2000);
    }
}
