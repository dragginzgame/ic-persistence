//! Owned named-allocation directory and durable extent validation.
pub(super) mod leases;
mod persistence;

use crate::{StableKey, hash};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const FRAME: &[u8] = b"ICDIRR01";

///
/// StorageConfig
///
/// Resource ceilings checked when creating, opening and extending the directory.
/// Physical page accounting includes retained metadata and abandoned tails.
///

#[derive(Clone, Copy, Debug)]
pub struct StorageConfig {
    /// Maximum retained named allocations, including currently omitted names.
    pub allocations: usize,
    /// Maximum total data extents across every allocation.
    pub extents: usize,
    /// Maximum physical backing extent, in 64 KiB Wasm pages.
    pub pages: u64,
    /// Encoding and decoding budgets for each directory snapshot.
    pub wire: ic_wire::Limits,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            allocations: 4_096,
            extents: 16_384,
            pages: 1_048_576,
            wire: ic_wire::Limits {
                bytes: 4 * 1024 * 1024,
                allocation: 16 * 1024 * 1024,
                nodes: 262_144,
                depth: 16,
            },
        }
    }
}

///
/// AllocationId
///
/// Opaque durable identity assigned by the directory. It grants no access.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct AllocationId(u64);
impl AllocationId {
    /// Opaque diagnostic identity; callers cannot choose an allocation ID.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Extent {
    start: u64,
    pub pages: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Entry {
    id: AllocationId,
    key: StableKey,
    // Logical page order is explicit; physical ranges need not be contiguous.
    extents: Vec<Extent>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Image {
    generation: u64,
    next_id: u64,
    // High-water mark, including space reserved for directory persistence.
    next_page: u64,
    // Reserved snapshot arenas, including the active arena. Never application data.
    metadata: Vec<Extent>,
    entries: Vec<Entry>,
}

impl Image {
    fn encode(&self, budget: StorageConfig) -> Result<Vec<u8>, Error> {
        let mut limits = budget.wire;
        limits.bytes = limits.bytes.checked_sub(FRAME.len()).ok_or(Error::Budget)?;
        let payload = ic_wire::native::encode(self, limits)?;
        let mut bytes = Vec::with_capacity(FRAME.len() + payload.len());
        bytes.extend_from_slice(FRAME);
        bytes.extend_from_slice(&payload);
        Ok(bytes)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Logical page count overflowed before any backing access.
    #[error("allocation page count overflow")]
    ArithmeticOverflow,
    #[error("directory budget exceeded")]
    Budget,
    #[error("invalid directory: {0}")]
    Invalid(&'static str),
    #[error("allocation already has a live lease")]
    AlreadyOpen,
    #[error("access exceeds allocation bounds")]
    Bounds,
    #[error("backing already has a live owner")]
    AlreadyClaimed,
    #[error("allocation does not exist")]
    Missing,
    #[error("prepared directory belongs to a stale generation")]
    Stale,
    #[error("backing growth refused")]
    Growth,
    #[error("directory owner is poisoned after interrupted backing IO")]
    Poisoned,
    #[error("Wire directory record: {0}")]
    Wire(#[from] ic_wire::Error),
}

#[derive(Clone, Debug)]
struct Directory {
    image: Image,
    by_key: BTreeMap<StableKey, AllocationId>,
    by_id: BTreeMap<AllocationId, usize>,
    // Derived only from validated extents; never persisted as a second authority.
    logical_ends: Vec<Vec<u64>>,
    budget: StorageConfig,
}

impl Directory {
    fn empty(budget: StorageConfig, data_start: u64) -> Result<Self, Error> {
        Self::validate(
            Image {
                generation: 0,
                next_id: 1,
                next_page: data_start,
                metadata: vec![],
                entries: vec![],
            },
            budget,
            data_start,
        )
    }

    fn validate(image: Image, budget: StorageConfig, data_start: u64) -> Result<Self, Error> {
        if image.entries.len() > budget.allocations || image.next_page > budget.pages {
            return Err(Error::Budget);
        }
        if data_start == 0 || image.next_page < data_start || image.next_id == 0 {
            return Err(Error::Invalid("root boundary or identity watermark"));
        }
        let mut by_key = BTreeMap::new();
        let mut by_id = BTreeMap::new();
        if image.metadata.len() > 128 {
            return Err(Error::Budget);
        }
        let mut ranges = Vec::new();
        for extent in &image.metadata {
            let end = extent
                .start
                .checked_add(extent.pages)
                .ok_or(Error::Budget)?;
            if extent.pages == 0 || extent.start < data_start || end > image.next_page {
                return Err(Error::Invalid("metadata arena outside backing"));
            }
            ranges.push((extent.start, end));
        }
        let mut data_extents = 0;
        let mut previous = None;
        for (index, entry) in image.entries.iter().enumerate() {
            if entry.id.0 == 0
                || entry.id.0 >= image.next_id
                || previous.is_some_and(|id| id >= entry.id)
            {
                return Err(Error::Invalid("noncanonical allocation identities"));
            }
            previous = Some(entry.id);
            if by_key.insert(entry.key.clone(), entry.id).is_some() {
                return Err(Error::Invalid("duplicate stable key"));
            }
            by_id.insert(entry.id, index);
            if entry.extents.len() > budget.extents.saturating_sub(data_extents) {
                return Err(Error::Budget);
            }
            data_extents += entry.extents.len();
            for extent in &entry.extents {
                let end = extent
                    .start
                    .checked_add(extent.pages)
                    .ok_or(Error::Invalid("extent overflow"))?;
                if extent.pages == 0 || extent.start < data_start || end > image.next_page {
                    return Err(Error::Invalid("extent outside data region"));
                }
                ranges.push((extent.start, end));
            }
        }
        ranges.sort_unstable();
        if ranges.windows(2).any(|pair| pair[0].1 > pair[1].0) {
            return Err(Error::Invalid("overlapping physical extents"));
        }
        let logical_ends = image
            .entries
            .iter()
            .map(|entry| {
                let mut total = 0;
                entry
                    .extents
                    .iter()
                    .map(|extent| {
                        // Disjoint ranges fit within the admitted physical watermark.
                        total += extent.pages;
                        total
                    })
                    .collect()
            })
            .collect();
        Ok(Self {
            image,
            by_key,
            by_id,
            logical_ends,
            budget,
        })
    }

    fn decode(bytes: &[u8], budget: StorageConfig, data_start: u64) -> Result<Self, Error> {
        if bytes.len() > budget.wire.bytes {
            return Err(Error::Budget);
        }
        let payload = bytes
            .strip_prefix(FRAME)
            .ok_or(Error::Invalid("directory frame"))?;
        let image = ic_wire::native::decode(payload, budget.wire)?;
        Self::validate(image, budget, data_start)
    }

    // Extend the validated base directly; no second indexed Directory is needed
    // merely to account for a native write's uncommitted physical tail.
    fn candidate_image(
        &self,
        keys: Vec<StableKey>,
        growth: &[(AllocationId, u64)],
        physical_end: u64,
    ) -> Result<Image, Error> {
        let mut image = self.image.clone();
        image.next_page = image.next_page.max(physical_end);
        let keys: BTreeSet<_> = keys.into_iter().collect();
        for key in keys {
            if self.by_key.contains_key(&key) {
                continue;
            }
            if image.entries.len() >= self.budget.allocations {
                return Err(Error::Budget);
            }
            let id = AllocationId(image.next_id);
            image.next_id = image.next_id.checked_add(1).ok_or(Error::Budget)?;
            image.entries.push(Entry {
                id,
                key,
                extents: vec![],
            });
        }
        for &(id, pages) in growth {
            // Growth applies only to bindings already present in the base.
            let index = *self.by_id.get(&id).ok_or(Error::Missing)?;
            if pages == 0 {
                continue;
            }
            let start = image.next_page;
            image.next_page = start.checked_add(pages).ok_or(Error::Budget)?;
            if let Some(last) = image.entries[index].extents.last_mut()
                && last.start + last.pages == start
            {
                last.pages += pages;
            } else {
                image.entries[index].extents.push(Extent { start, pages });
            }
        }
        image.generation = image.generation.checked_add(1).ok_or(Error::Budget)?;
        Ok(image)
    }

    // Model-only application; production validates once after reserving metadata.
    #[cfg(test)]
    fn apply(&mut self, keys: Vec<StableKey>, growth: &[(AllocationId, u64)]) -> Result<(), Error> {
        let image = self.candidate_image(keys, growth, self.image.next_page)?;
        let candidate = Self::validate(image, self.budget, 1)?;
        candidate.image.encode(self.budget)?;
        *self = candidate;
        Ok(())
    }

    #[cfg(test)]
    fn page_span(&self, id: AllocationId, logical: u64) -> Option<(u64, u64)> {
        let index = *self.by_id.get(&id)?;
        let ends = &self.logical_ends[index];
        let slot = ends.partition_point(|&end| end <= logical);
        let extent = self.image.entries[index].extents.get(slot)?;
        let start = if slot == 0 { 0 } else { ends[slot - 1] };
        Some((extent.start + logical - start, ends[slot] - logical))
    }

    #[cfg(test)]
    fn page(&self, id: AllocationId, logical: u64) -> Option<u64> {
        self.page_span(id, logical).map(|(page, _)| page)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(i: usize) -> StableKey {
        StableKey::parse(format!("test.store{i}.v1")).unwrap()
    }

    #[test]
    fn identities_survive_reorder_omission_and_reopen_beyond_256() {
        let budget = StorageConfig::default();
        let mut d = Directory::empty(budget, 1).unwrap();
        d.apply((0..1_024).rev().map(key).collect(), &[]).unwrap();
        let original = d.by_key.clone();
        d.apply(vec![key(0), key(2_000)], &[]).unwrap();
        let recovered = Directory::decode(&d.image.encode(budget).unwrap(), budget, 1).unwrap();
        for (key, id) in original {
            assert_eq!(recovered.by_key[&key], id);
        }
        assert_eq!(recovered.by_key[&key(2_000)], AllocationId(1_025));
        assert_eq!(recovered.image.entries.len(), 1_025);
    }

    #[test]
    fn interleaved_growth_maps_disjoint_ranges() {
        let mut d = Directory::empty(StorageConfig::default(), 2).unwrap();
        d.apply(vec![key(0), key(1)], &[]).unwrap();
        let a = d.by_key[&key(0)];
        let b = d.by_key[&key(1)];
        d.apply(vec![], &[(a, 2), (b, 3), (a, 1)]).unwrap();
        assert_eq!(
            (d.page(a, 0), d.page(a, 1), d.page(a, 2), d.page(a, 3)),
            (Some(2), Some(3), Some(7), None)
        );
        assert_eq!((d.page(b, 0), d.page(b, 2)), (Some(4), Some(6)));
    }

    #[test]
    fn rejected_budgets_and_overflows_preserve_the_base() {
        let budget = StorageConfig {
            allocations: 1,
            extents: 1,
            pages: 3,
            ..StorageConfig::default()
        };
        let mut d = Directory::empty(budget, 1).unwrap();
        d.apply(vec![key(0)], &[]).unwrap();
        let original = d.image.clone();
        let id = d.by_key[&key(0)];
        assert!(d.apply(vec![key(1)], &[]).is_err());
        assert!(d.apply(vec![], &[(id, 3)]).is_err());
        assert!(d.apply(vec![], &[(id, u64::MAX)]).is_err());
        assert!(d.apply(vec![], &[(id, 1), (id, 2)]).is_err());
        assert_eq!(d.image, original);
    }

    #[test]
    fn mutation_enforces_owner_budget_and_identity_exhaustion_is_fallible() {
        let mut strict = Directory::empty(
            StorageConfig {
                allocations: 1,
                ..StorageConfig::default()
            },
            1,
        )
        .unwrap();
        let relaxed = Directory::empty(StorageConfig::default(), 1).unwrap();
        let before = strict.image.clone();
        assert!(matches!(
            strict.apply(vec![key(0), key(1)], &[]),
            Err(Error::Budget)
        ));
        assert_eq!(strict.image, before);

        let mut image = relaxed.image;
        image.next_id = u64::MAX - 1;
        let mut d = Directory::validate(image, StorageConfig::default(), 1).unwrap();
        d.apply(vec![key(0)], &[]).unwrap();
        assert_eq!(d.by_key[&key(0)], AllocationId(u64::MAX - 1));
        let before = d.image.clone();
        assert!(matches!(d.apply(vec![key(1)], &[]), Err(Error::Budget)));
        assert_eq!(d.image, before);
    }

    #[test]
    fn recovery_rejects_overlap_duplicate_keys_bounds_and_bad_wire() {
        let budget = StorageConfig::default();
        let mut d = Directory::empty(budget, 1).unwrap();
        d.apply(vec![key(0), key(1)], &[]).unwrap();
        let ids: Vec<_> = d.by_id.keys().copied().collect();
        d.apply(vec![], &[(ids[0], 2), (ids[1], 2)]).unwrap();
        for mutation in 0..5 {
            let mut image = d.image.clone();
            match mutation {
                0 => image.entries[1].extents[0].start = 2,
                1 => image.entries[1].key = image.entries[0].key.clone(),
                2 => image.entries[0].extents[0].start = 0,
                3 => image.entries[0].extents[0].pages = u64::MAX,
                _ => image.entries[1].id = image.entries[0].id,
            }
            let payload = ic_wire::native::encode(&image, budget.wire).unwrap();
            let bytes = [FRAME, &payload].concat();
            assert!(Directory::decode(&bytes, budget, 1).is_err());
        }
        let bytes = d.image.encode(budget).unwrap();
        for end in 0..bytes.len() {
            assert!(Directory::decode(&bytes[..end], budget, 1).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(Directory::decode(&trailing, budget, 1).is_err());
        assert!(
            Directory::decode(
                &bytes,
                StorageConfig {
                    wire: ic_wire::Limits {
                        bytes: 8,
                        ..budget.wire
                    },
                    ..budget
                },
                1
            )
            .is_err()
        );
    }
}
