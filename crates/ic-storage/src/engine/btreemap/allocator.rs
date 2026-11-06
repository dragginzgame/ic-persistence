// Adapted from ic-stable-structures 0.7.2, DFINITY Foundation.
// SPDX-License-Identifier: Apache-2.0
// See the package LICENSE-APACHE and docs/storage/icss-source.json.
use crate::engine::{
    Memory, read_struct,
    types::{Address, Bytes, NULL},
    write_struct,
};

const ALLOCATOR_LAYOUT_VERSION: u8 = 1;
const CHUNK_LAYOUT_VERSION: u8 = 1;

const ALLOCATOR_MAGIC: &[u8; 3] = b"BTA"; // btree allocator
const CHUNK_MAGIC: &[u8; 3] = b"CHK"; // btree allocator
pub struct Allocator<M: Memory> {
    // The address in memory where the `AllocatorHeader` is stored.
    header_addr: Address,

    // The size of the chunk to allocate in bytes.
    allocation_size: Bytes,

    // The number of chunks currently allocated.
    num_allocated_chunks: u64,

    // A linked list of unallocated chunks.
    free_list_head: Address,

    memory: M,
}

#[repr(C, packed)]
#[derive(PartialEq, Debug)]
struct AllocatorHeader {
    magic: [u8; 3],
    version: u8,
    // Empty space to memory-align the following fields.
    _alignment: [u8; 4],
    allocation_size: Bytes,
    num_allocated_chunks: u64,
    free_list_head: Address,
    // Additional space reserved to add new fields without breaking backward-compatibility.
    _buffer: [u8; 16],
}

impl AllocatorHeader {
    fn size() -> Bytes {
        Bytes::from(core::mem::size_of::<Self>() as u64)
    }
}

impl<M: Memory> Allocator<M> {
    pub fn new(memory: M, addr: Address, allocation_size: Bytes) -> Self {
        let mut allocator = Self {
            header_addr: addr,
            allocation_size,
            num_allocated_chunks: 0,
            free_list_head: NULL, // Will be set in `allocator.clear()` below.
            memory,
        };

        allocator.clear();
        allocator
    }
    pub fn clear(&mut self) {
        // Create the initial memory chunk and save it directly after the allocator's header.
        self.free_list_head = self.header_addr + AllocatorHeader::size();
        let chunk = ChunkHeader::null();
        chunk.save(self.free_list_head, &self.memory);

        self.num_allocated_chunks = 0;

        self.save();
    }
    pub fn load(memory: M, addr: Address) -> Self {
        let header: AllocatorHeader = read_struct(addr, &memory);
        assert_eq!(&header.magic, ALLOCATOR_MAGIC, "Bad magic.");
        assert_eq!(
            header.version, ALLOCATOR_LAYOUT_VERSION,
            "Unsupported version."
        );

        Self {
            header_addr: addr,
            allocation_size: header.allocation_size,
            num_allocated_chunks: header.num_allocated_chunks,
            free_list_head: header.free_list_head,
            memory,
        }
    }
    pub fn allocate(&mut self) -> Address {
        // Get the next available chunk.
        let chunk_addr = self.free_list_head;
        let mut chunk = ChunkHeader::load(chunk_addr, &self.memory);

        // The available chunk must not be allocated.
        assert!(
            chunk.allocated == 0,
            "Attempting to allocate an already allocated chunk."
        );

        // Allocate the chunk.
        chunk.allocated = 1;
        chunk.save(chunk_addr, &self.memory);

        // Update the head of the free list.
        if chunk.next == NULL {
            // There is no next chunk. Shift everything by chunk size.
            self.free_list_head += self.chunk_size();

            // Write new chunk to that location.
            ChunkHeader::null().save(self.free_list_head, &self.memory);
        } else {
            // The next chunk becomes the new head of the list.
            self.free_list_head = chunk.next;
        }

        self.num_allocated_chunks += 1;
        self.save();

        // Return the chunk's address offset by the chunk's header.
        chunk_addr + ChunkHeader::size()
    }
    pub fn deallocate(&mut self, address: Address) {
        let chunk_addr = address - ChunkHeader::size();

        let mut chunk = ChunkHeader::load(chunk_addr, &self.memory);

        assert_eq!(chunk.allocated, 1);

        chunk.allocated = 0;
        chunk.next = self.free_list_head;

        chunk.save(chunk_addr, &self.memory);

        self.free_list_head = chunk_addr;

        self.num_allocated_chunks -= 1;
        self.save();
    }
    pub fn save(&self) {
        let header = AllocatorHeader {
            magic: *ALLOCATOR_MAGIC,
            version: ALLOCATOR_LAYOUT_VERSION,
            _alignment: [0; 4],
            num_allocated_chunks: self.num_allocated_chunks,
            allocation_size: self.allocation_size,
            free_list_head: self.free_list_head,
            _buffer: [0; 16],
        };

        write_struct(&header, self.header_addr, &self.memory);
    }

    // The full size of a chunk, which is the size of the header + the `allocation_size` that's
    // available to the user.
    fn chunk_size(&self) -> Bytes {
        self.allocation_size + ChunkHeader::size()
    }
    #[inline]
    pub const fn memory(&self) -> &M {
        &self.memory
    }
}

#[derive(Debug)]
#[repr(C, packed)]
struct ChunkHeader {
    magic: [u8; 3],
    version: u8,
    allocated: u8,
    // Empty space to memory-align the following fields.
    _alignment: [u8; 3],
    next: Address,
}

impl ChunkHeader {
    // Initializes an unallocated chunk that doesn't point to another chunk.
    const fn null() -> Self {
        Self {
            magic: *CHUNK_MAGIC,
            version: CHUNK_LAYOUT_VERSION,
            allocated: 0,
            _alignment: [0; 3],
            next: NULL,
        }
    }

    fn save<M: Memory>(&self, address: Address, memory: &M) {
        write_struct(self, address, memory);
    }

    fn load<M: Memory>(address: Address, memory: &M) -> Self {
        let header: Self = read_struct(address, memory);
        assert!(header.allocated <= 1, "Invalid chunk allocation flag");
        assert_eq!(&header.magic, CHUNK_MAGIC, "Bad magic.");
        assert_eq!(header.version, CHUNK_LAYOUT_VERSION, "Unsupported version.");

        header
    }

    fn size() -> Bytes {
        Bytes::from(core::mem::size_of::<Self>() as u64)
    }
}
