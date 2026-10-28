//! Process-local backing claims, retained by every issued runtime handle.
use super::RuntimeConstructionError;
use ic_platform::stable_structures::{Memory, VectorMemory};
use std::{collections::BTreeSet, rc::Rc, sync::Arc};

///
/// BackingIdentity
///
/// Identity of a physical backing allocation, shared by all of its wrappers.
/// This is host custody metadata, not a virtual memory ID or durable record.
///

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BackingIdentity(usize);

impl BackingIdentity {
    /// Identify an Rc-owned backing. Its allocation must remain alive through
    /// every runtime/handle using this identity. Identify the underlying backing,
    /// not a newly allocated wrapper or an address inside a growable byte buffer.
    #[must_use]
    pub fn of_rc<T: ?Sized>(backing: &Rc<T>) -> Self {
        Self(Rc::as_ptr(backing).cast::<()>() as usize)
    }

    /// Identify an Arc-owned backing under the same contract as [`Self::of_rc`].
    #[must_use]
    pub fn of_arc<T: ?Sized>(backing: &Arc<T>) -> Self {
        Self(Arc::as_ptr(backing).cast::<()>() as usize)
    }
}

///
/// MemoryBacking
///
/// Host boundary for a backing whose aliases can be identified consistently.
///
/// All wrappers and clones over the same physical backing must return the same
/// stable identity; independent backings must have different identities. Keep
/// its identity allocation alive for the backing's complete runtime/handle
/// lifetime. A wrapper delegates to its underlying backing's implementation.
/// Obtaining the identity must not read, grow or write backing memory or create
/// a manager/runtime; use a retained heap identity, not persisted contents.
/// Implementing Memory alone does not establish this custody contract. Raw IO
/// and unmanaged managers over a claimed backing remain prohibited; this is not
/// an isolation boundary against arbitrary linked code or a dishonest backing.
///
pub trait MemoryBacking: Memory {
    /// Return the physical backing identity under the trait's host contract.
    fn backing_identity(&self) -> BackingIdentity;
}

impl MemoryBacking for VectorMemory {
    fn backing_identity(&self) -> BackingIdentity {
        BackingIdentity::of_rc(self)
    }
}

impl<M: MemoryBacking> MemoryBacking for Rc<M> {
    fn backing_identity(&self) -> BackingIdentity {
        self.as_ref().backing_identity()
    }
}

#[cfg(target_arch = "wasm32")]
impl MemoryBacking for ic_platform::stable_structures::DefaultMemoryImpl {
    fn backing_identity(&self) -> BackingIdentity {
        // Every default handle addresses the same canister stable memory.
        BackingIdentity(0)
    }
}

#[cfg(not(target_arch = "wasm32"))]
static CLAIMS: std::sync::Mutex<BTreeSet<BackingIdentity>> = std::sync::Mutex::new(BTreeSet::new());

#[cfg(target_arch = "wasm32")]
thread_local! {
    static CLAIMS: std::cell::RefCell<BTreeSet<BackingIdentity>> = const {
        std::cell::RefCell::new(BTreeSet::new())
    };
}

fn with_claims<T>(operation: impl FnOnce(&mut BTreeSet<BackingIdentity>) -> T) -> T {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut claims = CLAIMS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        operation(&mut claims)
    }
    #[cfg(target_arch = "wasm32")]
    CLAIMS.with(|claims| operation(&mut claims.borrow_mut()))
}

pub(super) struct BackingClaim(BackingIdentity);

impl BackingClaim {
    pub fn acquire(memory: &impl MemoryBacking) -> Result<Self, RuntimeConstructionError> {
        let identity = memory.backing_identity();
        if with_claims(|claims| claims.insert(identity)) {
            Ok(Self(identity))
        } else {
            Err(RuntimeConstructionError::BackingAlreadyClaimed)
        }
    }
}

impl Drop for BackingClaim {
    fn drop(&mut self) {
        with_claims(|claims| claims.remove(&self.0));
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
impl<M: MemoryBacking> MemoryBacking for ic_testkit_native::memory::MeteredMemory<M> {
    fn backing_identity(&self) -> BackingIdentity {
        self.backing().backing_identity()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::{MemoryManagerConfig, MemoryRuntime, testing::fixture_runtime};
    use std::{cell::Cell, panic::AssertUnwindSafe};

    #[derive(Clone, Copy, Debug)]
    enum Failure {
        None,
        Grow,
        Initialize,
        Capture,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Destruction {
        strong_refs: usize,
        claimed_after_free: bool,
    }

    struct TrackedBacking {
        memory: Option<VectorMemory>,
        failure: Failure,
        destruction: Rc<Cell<Option<Destruction>>>,
    }

    impl TrackedBacking {
        fn new(failure: Failure) -> Self {
            Self {
                memory: Some(VectorMemory::default()),
                failure,
                destruction: Rc::default(),
            }
        }

        const fn memory(&self) -> &VectorMemory {
            self.memory.as_ref().unwrap()
        }
    }

    impl MemoryBacking for TrackedBacking {
        fn backing_identity(&self) -> BackingIdentity {
            self.memory().backing_identity()
        }
    }

    impl Memory for TrackedBacking {
        fn size(&self) -> u64 {
            self.memory().size()
        }

        fn grow(&self, pages: u64) -> i64 {
            if matches!(self.failure, Failure::Grow) {
                -1
            } else {
                self.memory().grow(pages)
            }
        }

        fn read(&self, offset: u64, dst: &mut [u8]) {
            assert!(
                !matches!(self.failure, Failure::Capture)
                    || dst.len() != super::super::layout::METADATA_BYTES,
                "injected authority capture failure"
            );
            self.memory().read(offset, dst);
        }

        fn write(&self, offset: u64, src: &[u8]) {
            assert!(
                !matches!(self.failure, Failure::Initialize),
                "injected manager initialization failure"
            );
            self.memory().write(offset, src);
        }
    }

    impl Drop for TrackedBacking {
        fn drop(&mut self) {
            let memory = self.memory.take().unwrap();
            let identity = memory.backing_identity();
            let strong_refs = Rc::strong_count(&memory);
            // Hold the registry lock across freeing the final allocation and
            // observing its former identity. Other tests cannot reuse that
            // address and install a legitimate new claim before this check.
            let claimed_after_free = with_claims(|claims| {
                drop(memory);
                claims.contains(&identity)
            });
            // Record instead of asserting in Drop: construction cases unwind.
            self.destruction.set(Some(Destruction {
                strong_refs,
                claimed_after_free,
            }));
        }
    }

    #[test]
    fn final_runtime_or_detached_handle_releases_claim_before_identity_allocation() {
        for detach_handle in [false, true] {
            let backing = TrackedBacking::new(Failure::None);
            let destruction = Rc::clone(&backing.destruction);
            let identity = backing.backing_identity();
            let runtime = fixture_runtime(
                backing,
                "drop",
                "drop.",
                &["drop.rows.v1"],
                MemoryManagerConfig::new(8).unwrap(),
            )
            .unwrap();
            let handle = detach_handle.then(|| runtime.open_memory("drop.rows.v1").unwrap());
            drop(runtime);
            if handle.is_some() {
                assert_eq!(destruction.get(), None);
                assert!(with_claims(|claims| claims.contains(&identity)));
            }
            drop(handle);
            assert_eq!(
                destruction.get(),
                Some(Destruction {
                    strong_refs: 1,
                    claimed_after_free: false,
                })
            );
        }
    }

    #[test]
    fn construction_refusal_and_unwind_release_claim_before_identity_allocation() {
        for failure in [Failure::Grow, Failure::Initialize, Failure::Capture] {
            let backing = TrackedBacking::new(failure);
            let destruction = Rc::clone(&backing.destruction);
            let result = std::panic::catch_unwind(AssertUnwindSafe(|| MemoryRuntime::new(backing)));
            if matches!(failure, Failure::Grow) {
                assert!(matches!(
                    result,
                    Ok(Err(RuntimeConstructionError::Growth(_)))
                ));
            } else {
                assert!(result.is_err(), "expected injected {failure:?} failure");
            }
            assert_eq!(
                destruction.get(),
                Some(Destruction {
                    strong_refs: 1,
                    claimed_after_free: false,
                }),
                "failure during {failure:?} retained a dead backing identity"
            );
        }
    }

    #[test]
    fn independent_backings_construct_and_drop_concurrently() {
        let start = std::sync::Barrier::new(4);
        std::thread::scope(|scope| {
            for _ in 0..4 {
                let start = &start;
                scope.spawn(move || {
                    start.wait();
                    for _ in 0..2000 {
                        drop(MemoryRuntime::new(VectorMemory::default()).unwrap());
                    }
                });
            }
        });
    }
}
