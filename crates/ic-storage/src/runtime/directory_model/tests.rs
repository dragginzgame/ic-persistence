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

// Private fixture helpers; excluded from production compilation.
impl Directory {
    fn apply(&mut self, keys: Vec<StableKey>, growth: &[(AllocationId, u64)]) -> Result<(), Error> {
        let image = self.candidate_image(keys, growth, self.image.next_page)?;
        let candidate = Self::validate(image, self.budget, 1)?;
        candidate.image.encode(self.budget)?;
        *self = candidate;
        Ok(())
    }

    fn page_span(&self, id: AllocationId, logical: u64) -> Option<(u64, u64)> {
        let index = *self.by_id.get(&id)?;
        let ends = &self.logical_ends[index];
        let slot = ends.partition_point(|&end| end <= logical);
        let extent = self.image.entries[index].extents.get(slot)?;
        let start = if slot == 0 { 0 } else { ends[slot - 1] };
        Some((extent.start + logical - start, ends[slot] - logical))
    }

    pub(super) fn page(&self, id: AllocationId, logical: u64) -> Option<u64> {
        self.page_span(id, logical).map(|(page, _)| page)
    }
}
