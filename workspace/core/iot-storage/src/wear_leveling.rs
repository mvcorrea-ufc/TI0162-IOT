//! # Wear Leveling Implementation
//!
//! Advanced wear leveling algorithms for flash storage to maximize
//! device lifetime and ensure uniform wear across all sectors.

use heapless::Vec;
use crate::{
    traits::{WearLeveling, StorageError, StorageResult},
};

/// Wear leveling statistics for a sector
#[derive(Debug, Clone, Copy)]
pub struct SectorWearInfo {
    /// Sector index
    pub sector_index: usize,
    /// Number of erase cycles
    pub erase_cycles: u32,
    /// Current wear level (0-100)
    pub wear_level: u8,
    /// Whether sector is marked as bad
    pub is_bad: bool,
    /// Last access timestamp
    pub last_access: u64,
}

/// Wear leveling manager
pub struct WearLevelingManager {
    /// Sector wear information
    sectors: Vec<SectorWearInfo, 64>,
    /// Maximum erase cycles per sector
    max_erase_cycles: u32,
    /// Wear leveling threshold
    wear_threshold: u8,
    /// Bad block count
    bad_block_count: usize,
}

impl WearLevelingManager {
    /// Create new wear leveling manager
    pub fn new(sector_count: usize, max_erase_cycles: u32) -> StorageResult<Self> {
        let mut sectors = Vec::new();
        
        for i in 0..sector_count {
            let sector_info = SectorWearInfo {
                sector_index: i,
                erase_cycles: 0,
                wear_level: 0,
                is_bad: false,
                last_access: 0,
            };
            sectors.push(sector_info).map_err(|_| StorageError::CapacityExceeded)?;
        }
        
        Ok(Self {
            sectors,
            max_erase_cycles,
            wear_threshold: 80,
            bad_block_count: 0,
        })
    }

    /// Update sector erase count
    pub fn update_erase_count(&mut self, sector: usize) -> StorageResult<()> {
        if sector >= self.sectors.len() {
            return Err(StorageError::InvalidValue);
        }
        
        let erase_cycles = {
            let sector_info = &mut self.sectors[sector];
            sector_info.erase_cycles += 1;
            sector_info.last_access = 0; // Would use actual timestamp
            sector_info.erase_cycles
        };
        
        // Calculate wear level after releasing the mutable borrow
        let wear_level = self.calculate_wear_level(erase_cycles);
        self.sectors[sector].wear_level = wear_level;
        
        // Check if sector should be marked as bad
        if erase_cycles >= self.max_erase_cycles && !self.sectors[sector].is_bad {
            self.mark_sector_bad(sector)?;
        }
        
        Ok(())
    }

    /// Calculate wear level from erase cycles
    fn calculate_wear_level(&self, erase_cycles: u32) -> u8 {
        ((erase_cycles * 100) / self.max_erase_cycles).min(100) as u8
    }

    /// Mark sector as bad
    fn mark_sector_bad(&mut self, sector: usize) -> StorageResult<()> {
        if sector >= self.sectors.len() {
            return Err(StorageError::InvalidValue);
        }
        
        if !self.sectors[sector].is_bad {
            self.sectors[sector].is_bad = true;
            self.bad_block_count += 1;
        }
        
        Ok(())
    }

    /// Find sectors that need wear leveling
    pub fn find_wear_leveling_candidates(&self) -> Option<(usize, usize)> {
        let mut min_wear_sector = 0;
        let mut max_wear_sector = 0;
        let mut min_cycles = u32::MAX;
        let mut max_cycles = 0;
        
        // Find sectors with minimum and maximum wear
        for (index, sector) in self.sectors.iter().enumerate() {
            if !sector.is_bad {
                if sector.erase_cycles < min_cycles {
                    min_cycles = sector.erase_cycles;
                    min_wear_sector = index;
                }
                if sector.erase_cycles > max_cycles {
                    max_cycles = sector.erase_cycles;
                    max_wear_sector = index;
                }
            }
        }
        
        // Check if wear leveling is needed
        let wear_difference = max_cycles.saturating_sub(min_cycles);
        let threshold = self.max_erase_cycles / 20; // 5% threshold
        
        if wear_difference > threshold {
            Some((max_wear_sector, min_wear_sector))
        } else {
            None
        }
    }

    /// Get sectors sorted by wear level
    pub fn get_sectors_by_wear(&self) -> Vec<usize, 64> {
        let mut sector_indices: Vec<usize, 64> = (0..self.sectors.len()).collect();
        
        // Sort by erase cycles (ascending)
        sector_indices.sort_by(|&a, &b| {
            self.sectors[a].erase_cycles.cmp(&self.sectors[b].erase_cycles)
        });
        
        sector_indices
    }

    /// Get available sectors (not bad)
    pub fn get_available_sectors(&self) -> Vec<usize, 64> {
        self.sectors.iter()
            .enumerate()
            .filter(|(_, sector)| !sector.is_bad)
            .map(|(index, _)| index)
            .collect()
    }

    /// Get sector with minimum wear (for new allocations)
    pub fn get_least_worn_sector(&self) -> Option<usize> {
        self.sectors.iter()
            .enumerate()
            .filter(|(_, sector)| !sector.is_bad)
            .min_by_key(|(_, sector)| sector.erase_cycles)
            .map(|(index, _)| index)
    }

    /// Estimate remaining lifetime for sector
    pub fn get_sector_remaining_lifetime(&self, sector: usize) -> u8 {
        if sector >= self.sectors.len() {
            return 0;
        }
        
        let sector_info = &self.sectors[sector];
        if sector_info.is_bad {
            return 0;
        }
        
        let remaining_cycles = self.max_erase_cycles.saturating_sub(sector_info.erase_cycles);
        ((remaining_cycles * 100) / self.max_erase_cycles).min(100) as u8
    }
}

impl WearLeveling for WearLevelingManager {
    fn get_wear_level(&self, region: usize) -> u8 {
        if region < self.sectors.len() {
            self.sectors[region].wear_level
        } else {
            0
        }
    }

    fn get_average_wear_level(&self) -> u8 {
        if self.sectors.is_empty() {
            return 0;
        }
        
        let total_wear: u32 = self.sectors.iter()
            .filter(|sector| !sector.is_bad)
            .map(|sector| sector.wear_level as u32)
            .sum();
        
        let available_sectors = self.sectors.iter().filter(|sector| !sector.is_bad).count();
        
        if available_sectors > 0 {
            (total_wear / available_sectors as u32) as u8
        } else {
            100 // All sectors are bad
        }
    }

    fn level_wear(&mut self) -> StorageResult<()> {
        if let Some((high_wear, low_wear)) = self.find_wear_leveling_candidates() {
            // In a real implementation, this would involve moving data
            // from high-wear sector to low-wear sector
            
            // Simulate the operation by updating access times
            self.sectors[high_wear].last_access = 1; // Simulated timestamp
            self.sectors[low_wear].last_access = 1; // Simulated timestamp
            
            Ok(())
        } else {
            // No wear leveling needed
            Ok(())
        }
    }

    fn get_bad_block_count(&self) -> usize {
        self.bad_block_count
    }

    fn mark_bad_block(&mut self, block: usize) -> StorageResult<()> {
        self.mark_sector_bad(block)
    }

    fn get_remaining_lifetime(&self) -> u8 {
        if self.sectors.is_empty() {
            return 0;
        }
        
        // Use the sector with minimum remaining lifetime
        self.sectors.iter()
            .enumerate()
            .filter(|(_, sector)| !sector.is_bad)
            .map(|(index, _)| self.get_sector_remaining_lifetime(index))
            .min()
            .unwrap_or(0)
    }
}