# PostgreSQL-Inspired Slotted Page in Rust

## Goal

Build a standalone, in-memory implementation of a PostgreSQL-inspired relational
heap page. The page should store variable-length serialized table rows inside one fixed-size byte array and
support:

- insertion;
- lookup by tuple identifier (TID);
- deletion;
- compaction;
- a deliberately simplified update operation;
- validation of the page's internal invariants.

Model a regular relational table such as:

```sql
CREATE TABLE users (
    id          INTEGER NOT NULL,
    username    TEXT NOT NULL,
    active      BOOLEAN NOT NULL,
    display_name TEXT
);
```

Each inserted row is serialized into a heap tuple before the page stores it.
The slotted page does not store nonrelational documents or understand SQL; it
stores the binary representation of relational rows.

This is a learning project, not a byte-for-byte PostgreSQL-compatible page
implementation. The important goal is to understand why a database page is laid
out this way and what must remain true after every operation.

Do not start with files, WAL, a buffer pool, SQL, MVCC, unsafe Rust, or raw
pointers. First make one page work entirely in memory using safe Rust.

## The central idea

A page is one fixed-size array of bytes:

```rust
pub const PAGE_SIZE: usize = 8192;

pub struct Page {
    data: [u8; PAGE_SIZE],
}
```

PostgreSQL normally uses 8 KiB pages, although it can be compiled with a
different page size. Use 8 KiB if the goal is to resemble PostgreSQL. Using 4
KiB would still teach the same design, but it would not match the normal
PostgreSQL default.

The Rust struct should initially contain only the byte array. The header, line
pointers, free space, and tuple bytes are all encoded *inside* that array.

The page manager should not know that column 0 is an integer or column 1 is
text. Its input is already-serialized tuple bytes. A separate tuple codec uses a
table schema to turn relational values into those bytes and back again:

```text
Row values + table schema -> TupleCodec::encode -> heap tuple bytes
heap tuple bytes + schema -> TupleCodec::decode -> Row values
```

Do not store native Rust pointers in the page. A pointer is meaningful only in
one process at one moment. A page must still make sense after being written to
disk and loaded at a different address. Store integer byte offsets instead.

## Mental model of the layout

```text
low addresses                                                  high addresses

+-------------+------------------+----------------------+------------------+
| Page header | line pointers -> | contiguous free space| <- tuple bytes   |
+-------------+------------------+----------------------+------------------+
0             pd_lower                                  pd_upper       8192
```

Two regions grow toward each other:

- The line-pointer array grows upward from the end of the header.
- Tuple bytes grow downward from the end of the page.
- The bytes in `[pd_lower, pd_upper)` are contiguous free space.
- The page is full when the two growing regions can no longer fit the new tuple
  and, when required, a new line pointer.

PostgreSQL calls the small entries in the slot array **line pointers** or item
identifiers. A line pointer identifies a tuple's current byte range within the
page.

## PostgreSQL concepts to imitate

### Page header

A real PostgreSQL page header is 24 bytes and includes more fields than this
exercise needs, including an LSN, checksum, flags, layout version, and pruning
information. Start with a smaller logical subset:

```text
pd_lower: first byte after the line-pointer array
pd_upper: first byte of the packed tuple-data region
pd_special: beginning of special space
```

For an ordinary heap page, special space is normally unused, so initialize:

```text
pd_lower   = HEADER_SIZE
pd_upper   = PAGE_SIZE
pd_special = PAGE_SIZE
```

You may reserve 24 bytes for `HEADER_SIZE` even before you implement all of the
real fields. This keeps the physical layout closer to PostgreSQL.

The essential invariant is:

```text
HEADER_SIZE <= pd_lower <= pd_upper <= pd_special <= PAGE_SIZE
```

### Line pointers

PostgreSQL uses 4-byte `ItemIdData` entries. Each entry encodes a tuple offset,
length, and state flags using packed bits. For the first version, use a simpler
4-byte encoding:

```rust
struct LinePointer {
    offset: u16,
    length_and_flags: u16,
}
```

You can initially simplify this further conceptually to:

```rust
struct LinePointer {
    offset: u16,
    length: u16,
}
```

but still encode its fields explicitly into four page bytes. Do not cast a
`LinePointer` struct directly into bytes; struct padding and native endianness
should not define an on-page format.

Eventually support states inspired by PostgreSQL:

- `Unused`: no tuple currently belongs to this line pointer;
- `Normal`: points to a live tuple byte range;
- `Dead`: known not to contain a useful tuple;
- `Redirect`: optional stretch goal related to HOT updates.

For the core project, `Unused` and `Normal` are enough.

### Tuple identifier (TID)

PostgreSQL identifies a physical heap tuple using roughly:

```text
(block number, offset number)
```

The block number selects a page. The offset number selects a line pointer on
that page. It is not a raw byte offset into the tuple-data region.

Use:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tid {
    pub block_number: u32,
    pub offset_number: u16,
}
```

PostgreSQL offset numbers are conventionally one-based. You can preserve that
behavior publicly while translating to a zero-based array index internally:

```text
line_pointer_index = offset_number - 1
```

Because this project initially owns only one page, its `block_number` can be
provided when constructing the page or when constructing the returned TID.

## Proposed public API

```rust
pub const PAGE_SIZE: usize = 8192;
pub type BlockNumber = u32;
pub type OffsetNumber = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tid {
    pub block_number: BlockNumber,
    pub offset_number: OffsetNumber,
}

pub struct Page {
    block_number: BlockNumber,
    data: [u8; PAGE_SIZE],
}

impl Page {
    pub fn new(block_number: BlockNumber) -> Self;
    pub fn insert(&mut self, tuple: &[u8]) -> Result<Tid, PageError>;
    pub fn get(&self, tid: Tid) -> Result<Option<&[u8]>, PageError>;
    pub fn delete(&mut self, tid: Tid) -> Result<bool, PageError>;
    pub fn update(&mut self, tid: Tid, tuple: &[u8]) -> Result<Tid, PageError>;
    pub fn compact(&mut self) -> Result<(), PageError>;
    pub fn contiguous_free_space(&self) -> usize;
    pub fn validate(&self) -> Result<(), PageError>;
    pub fn as_bytes(&self) -> &[u8; PAGE_SIZE];
    pub fn from_bytes(
        block_number: BlockNumber,
        data: [u8; PAGE_SIZE],
    ) -> Result<Self, PageError>;
}
```

Keep the relational layer separate:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Int32(i32),
    Bool(bool),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Row(pub Vec<Value>);

pub enum DataType {
    Int32,
    Bool,
    Text,
}

pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

pub struct Schema {
    pub columns: Vec<Column>,
}

pub struct TupleCodec;

impl TupleCodec {
    pub fn encode(schema: &Schema, row: &Row) -> Result<Vec<u8>, TupleError>;
    pub fn decode(schema: &Schema, bytes: &[u8]) -> Result<Row, TupleError>;
}
```

You do not have to write this entire API before beginning. It is the destination,
not checkpoint one.

## Suggested file/module layout

Keep the project small:

```text
src/
  lib.rs       public types and Page API
  codec.rs     read/write u16 values and line pointers
  page.rs      page operations and invariants
  tuple.rs     relational values and heap-tuple encoding
tests/
  page.rs      behavior tests
  tuple.rs     row round-trip tests
```

One file is also completely reasonable for the first flight session. Split it
only when the code becomes harder to navigate.

## Relational row encoding

Treat this as a layer above the slotted page. The page stores opaque tuple bytes;
the tuple codec understands columns and types.

For the first version, define this educational heap-tuple format:

```text
+----------------+-------------+------------------+-------------------+
| tuple header   | null bitmap | fixed-size data  | variable-size data|
+----------------+-------------+------------------+-------------------+
```

A simpler implementation can use:

```text
attribute count: u16
null bitmap:     ceil(attribute_count / 8) bytes
attribute data:  encoded in schema order
```

Encoding rules:

- `Int32`: exactly four little-endian bytes;
- `Bool`: one byte, `0` or `1`;
- `Text`: four-byte length followed by UTF-8 bytes;
- `NULL`: set the corresponding null-bitmap bit and store no value bytes.

For this row:

```text
(42, "joe", true, NULL)
```

the codec conceptually writes:

```text
[attribute count]
[null bitmap: column 3 is null]
[42 as i32]
[text length 3][j][o][e]
[true as 1]
[no bytes for display_name]
```

This is intentionally easier than PostgreSQL's real heap-tuple representation.
PostgreSQL has `HeapTupleHeaderData`, transaction metadata such as `xmin` and
`xmax`, a CTID, flags, an optional null bitmap, alignment rules, and encoded
attribute data. The educational format lets you understand the boundary before
reproducing those details.

Important separation:

```rust
let tuple = TupleCodec::encode(&users_schema, &row)?;
let tid = page.insert(&tuple)?;

let tuple = page.get(tid)?.ok_or(...)?;
let row = TupleCodec::decode(&users_schema, tuple)?;
```

The schema is not repeated inside every page. A catalog would normally retain
table metadata. For this standalone project, construct the schema in the test
or example program and pass it to the codec.

Tuple-codec tests:

- Round-trip an all-non-null row.
- Round-trip null values in several positions.
- Reject the wrong number of values.
- Reject `Null` in a non-nullable column.
- Reject a value whose type disagrees with its column.
- Reject truncated or malformed encoded text.
- Encode two rows of different sizes to demonstrate why the slotted page is
  needed.

## Milestone 1: initialize and inspect an empty page

Implement only:

- `Page::new`;
- header getters and setters;
- `contiguous_free_space`;
- `validate` for the header ordering.

Decide on a byte order, such as little-endian, and always encode/decode integers
explicitly:

```rust
u16::to_le_bytes(value)
u16::from_le_bytes(bytes)
```

Questions to answer before continuing:

1. Why are `pd_lower` and `pd_upper` offsets rather than pointers?
2. Why does an empty page have `pd_lower = HEADER_SIZE`?
3. Why does an empty page have `pd_upper = PAGE_SIZE`?
4. What invalid page states can already be detected?

Tests:

- An empty page validates.
- Its line-pointer count is zero.
- Its contiguous free space is `PAGE_SIZE - HEADER_SIZE`.
- Loading a page with `pd_lower > pd_upper` fails validation.

## Milestone 2: encode and decode line pointers

Choose where line pointer `i` lives:

```text
HEADER_SIZE + i * LINE_POINTER_SIZE
```

Implement private helpers:

```rust
fn line_pointer_count(&self) -> usize;
fn read_line_pointer(&self, index: usize) -> Result<LinePointer, PageError>;
fn write_line_pointer(
    &mut self,
    index: usize,
    item: LinePointer,
) -> Result<(), PageError>;
```

The line-pointer count can be derived rather than stored separately:

```text
(pd_lower - HEADER_SIZE) / LINE_POINTER_SIZE
```

Questions:

1. Why is deriving the count useful?
2. What should happen if `pd_lower - HEADER_SIZE` is not divisible by four?
3. How will a line pointer represent `Unused` without making empty tuples
   impossible?
4. Which ranges must be validated before slicing the byte array?

Do not implement insertion until malformed line pointers produce errors rather
than panics.

## Milestone 3: insert and retrieve tuples

Insertion algorithm:

1. Reject a tuple that cannot be represented by the chosen length field.
2. Search existing line pointers for an `Unused` entry.
3. Decide whether insertion needs a new four-byte line pointer.
4. Calculate the new tuple start with checked arithmetic:

   ```text
   tuple_start = pd_upper - tuple_length
   ```

5. Verify that the tuple and optional new line pointer do not overlap:

   ```text
   new_pd_lower <= tuple_start
   ```

6. Only after all checks succeed, copy the tuple bytes.
7. Write or reuse the line pointer.
8. Update `pd_upper` and, if applicable, `pd_lower`.
9. Return `(block_number, offset_number)`.

Important property: a failed insertion must leave every page byte unchanged.
Do all calculations and validation before committing mutations.

Lookup algorithm:

1. Confirm that the TID's block number matches the page.
2. Translate its one-based offset number to a line-pointer index.
3. Confirm that the line pointer exists and is `Normal`.
4. Validate `offset + length` using checked arithmetic.
5. Confirm that the tuple range lies inside the tuple-data area.
6. Return a borrowed slice of the page bytes.

Tests:

- Insert and retrieve one tuple.
- Insert tuples of different lengths.
- Returned TIDs have stable and distinct offset numbers.
- An unknown or wrong-page TID does not panic.
- An oversized tuple fails without modifying the page.
- Insert until the page reports insufficient space.
- Optionally support a zero-length tuple and ensure it is not confused with an
  unused line pointer.

## Milestone 4: deletion and fragmentation

For the first implementation, deletion should mark the selected line pointer
`Unused`. Do not immediately move other tuples and do not shrink the
line-pointer array.

This creates an important distinction:

- **contiguous free space** is `[pd_lower, pd_upper)`;
- **reclaimable/fragmented space** includes bytes occupied by deleted tuples;
- total potential free space may therefore be larger than immediately usable
  contiguous space.

After deletion:

- `get(deleted_tid)` returns `None`;
- other TIDs still retrieve the same tuples;
- the deleted tuple's old bytes may remain unchanged;
- those stale bytes are not logically accessible through the page API.

Questions:

1. Why is leaving stale bytes behind logically safe but potentially a security
   concern?
2. Why might immediate compaction make every deletion unnecessarily expensive?
3. Should insertion reuse an unused line pointer? What happens to an old TID
   if it does?

For this exercise, reuse unused line pointers, but document the consequence:
the old TID can later identify a new tuple. Production systems coordinate TID
reuse with transaction visibility and vacuum rules; this project does not.

## Milestone 5: compact the page

Compaction repacks all tuples referenced by `Normal` line pointers against the
high end of the page.

Required behavior:

- Live tuple bytes may move.
- Every moved tuple's line-pointer offset must be updated.
- Line-pointer positions must not change.
- Therefore, all live TIDs remain stable.
- `pd_upper` moves upward to expose one contiguous free region.
- Deleted tuple bytes no longer occupy the packed tuple area.

One safe approach:

1. Read every live tuple into temporary owned byte vectors.
2. Clear or overwrite the tuple region as desired.
3. Pack the live tuples again from the end of the page downward.
4. Rewrite their line-pointer offsets without changing line-pointer indices.
5. Set the new `pd_upper`.
6. Validate the completed page.

This uses temporary memory and is not the cleverest implementation, but it is
easy to reason about. Optimize to in-place movement only after correctness is
obvious.

Tests:

- Insert A, B, and C; delete B; compact; still retrieve A and C using their
  original TIDs.
- Free space becomes contiguous after compaction.
- Tuple ranges do not overlap.
- Compaction is idempotent.
- Compacting an empty page succeeds.

## Milestone 6: updates

Do not begin by trying to reproduce PostgreSQL MVCC updates. First decide which
simplified semantics you are implementing.

### Version A: physical in-place page update

- If the replacement has the same or a smaller length, overwrite the bytes and
  adjust the line pointer's length.
- If it is larger, insert it elsewhere, update the original line pointer to the
  new location, and reclaim the old bytes during compaction.
- Preserve the same TID.

This teaches physical page management, but it is **not how a normal PostgreSQL
MVCC update should be understood**.

### Version B: PostgreSQL-inspired logical update

PostgreSQL generally creates a new tuple version rather than overwriting the old
tuple version in place. The old tuple header records deletion/update transaction
information, and its `ctid` can point to the newer version. A HOT update may keep
the new version on the same page when indexed columns are unchanged and space is
available.

For a small simulation:

```rust
pub struct TupleHeader {
    xmin: u32,
    xmax: u32,
    ctid: Tid,
}
```

Then an update can:

1. Insert a new tuple version, producing a new TID.
2. Set the old version's `xmax`.
3. Set the old version's `ctid` to the new TID.
4. Leave visibility decisions to a later MVCC exercise.

This is a stretch goal. Finish deletion and compaction first.

## Milestone 7: treat page bytes as untrusted

`Page::from_bytes` should validate an existing byte array before accepting it.
At minimum, check:

- header boundaries are ordered and within the page;
- the line-pointer area has an integral number of entries;
- every normal line pointer has a representable, in-bounds range;
- every normal tuple lies at or above `pd_upper` and below `pd_special`;
- no two normal tuple ranges overlap;
- no normal tuple overlaps the header or line-pointer area;
- encoded state flags are recognized.

Fuzz-like test without external dependencies:

1. Create a valid page.
2. Copy its bytes.
3. Corrupt selected header or line-pointer bytes.
4. Verify that loading returns `Err` instead of panicking.

This is excellent systems-interview material: data read from storage or the
network must not be trusted merely because your program originally wrote it.

## Errors worth modeling

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum PageError {
    NoSpace,
    WrongBlock,
    InvalidOffsetNumber,
    TupleTooLarge,
    CorruptHeader,
    CorruptLinePointer,
    OverlappingTuples,
}
```

The exact variants can change. Prefer explicit errors for corruption and
expected capacity failures. Avoid panics for malformed page contents.

## Core invariants checklist

Call `validate()` liberally in tests. After every successful operation:

- [ ] Header offsets are ordered and within the page.
- [ ] `pd_lower` equals the end of the line-pointer array.
- [ ] Every normal line pointer refers to an in-bounds tuple.
- [ ] No tuple overlaps the header or line-pointer array.
- [ ] No two live tuple ranges overlap.
- [ ] Every live tuple lies within `[pd_upper, pd_special)`.
- [ ] A failed operation leaves the page unchanged.
- [ ] Compaction preserves the TIDs of all live tuples.
- [ ] Lookup never slices before completing checked bounds validation.

## What is PostgreSQL-like and what is simplified?

| Area | This project | PostgreSQL |
| --- | --- | --- |
| Default page size | 8 KiB | Normally 8 KiB |
| Page representation | Fixed byte array | Fixed-size buffer page |
| Header boundaries | `pd_lower`, `pd_upper`, `pd_special` | Same core concepts |
| Slot metadata | Explicit 4-byte simplified encoding | Packed 4-byte `ItemIdData` |
| Tuple ID | `(block, one-based offset)` | `ItemPointerData`/CTID concept |
| Deletion | Mark line pointer unused | MVCC tuple state followed later by pruning/vacuum |
| Update | Optional simplified physical or versioned update | New MVCC tuple version; possibly HOT |
| Tuple contents | Opaque caller-provided bytes | Heap tuple header plus null bitmap and attributes |
| WAL/checksum/LSN | Omitted initially | Integral to production behavior |

## Suggested session plan

### Session 1: understand the bytes

- Draw the empty page and one inserted tuple on paper.
- Implement header codecs and validation.
- Implement line-pointer codecs.
- Insert and retrieve one tuple.

Stop and explain aloud exactly which byte ranges changed.

### Session 2: create fragmentation

- Insert several differently sized tuples.
- Delete the middle one.
- Display header values, line pointers, and tuple ranges.
- Implement unused-line-pointer reuse.

Stop and explain why total reclaimable space differs from contiguous free
space.

### Session 3: compact and harden

- Implement compaction using temporary buffers.
- Preserve TIDs.
- Add corruption validation.
- Add property-style loops that perform many inserts/deletes and call
  `validate()` after every operation.

### Session 4: optional PostgreSQL depth

- Add a small tuple header.
- Simulate version-creating updates.
- Read about PostgreSQL `PageHeaderData`, `ItemIdData`, `HeapTupleHeaderData`,
  CTIDs, and HOT updates.
- Write down exactly where your design intentionally differs.

## Debugging tool: print a page map

Implement a method that prints logical metadata rather than dumping all 8 KiB:

```text
block=7 lower=36 upper=8140 special=8192 free=8104
offset 1: NORMAL offset=8179 length=13
offset 2: UNUSED
offset 3: NORMAL offset=8140 length=39
```

This will be far more helpful on a flight than staring at a raw byte dump. An
optional hex dump of only the header and occupied tuple ranges is also useful.

## Questions to use when stuck

For any operation, ask in this order:

1. What are the page invariants before the operation?
2. Which byte ranges must be read?
3. Which arithmetic can overflow or underflow?
4. Can I calculate the complete result before mutating anything?
5. Exactly which byte ranges and header fields should change?
6. What must remain stable, especially TIDs?
7. What happens when there is one byte less space than required?
8. What malformed input could cause a panic?
9. Can a test compare the page bytes before and after a failed operation?

## Things deliberately left for later

Do not add these until the basic page is correct:

- disk files and page numbers managed by a pager;
- buffer-pool eviction and dirty-page tracking;
- WAL and page LSN enforcement;
- checksums;
- concurrent readers and writers;
- latches or locks;
- transaction status and snapshot visibility;
- vacuum and pruning;
- indexes;
- TOAST-like out-of-line storage for oversized values;
- true PostgreSQL binary compatibility.

These are natural future projects. A useful eventual sequence is:

```text
slotted page -> heap file -> buffer pool -> WAL/recovery -> MVCC -> indexes
```

## Definition of done

The core project is done when:

- one 8 KiB byte array contains both metadata and variable-sized tuples;
- insertion returns stable PostgreSQL-style TIDs;
- lookup resolves a TID through its line pointer;
- deletion creates fragmentation without breaking other TIDs;
- compaction reclaims fragmentation without changing live TIDs;
- malformed page metadata returns errors rather than panicking;
- the tests demonstrate every core invariant;
- you can draw the layout and explain why each level of indirection exists.

If you reach this point, resist immediately adding more features. Write a short
summary of what surprised you and which PostgreSQL behavior you simplified.
Being able to explain those tradeoffs is as valuable as the implementation.
