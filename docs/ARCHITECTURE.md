# SSHChic Architecture

This document provides a detailed overview of SSHChic's architecture, design decisions, and implementation details.

## Table of Contents

- [Overview](#overview)
- [Architecture Diagrams](#architecture-diagrams)
- [Component Details](#component-details)
- [Data Flow](#data-flow)
- [Concurrency Model](#concurrency-model)
- [Performance Characteristics](#performance-characteristics)
- [Security Considerations](#security-considerations)
- [Design Decisions](#design-decisions)

## Overview

SSHChic is a single-binary, multi-threaded CLI application written in Rust. It generates ED25519 SSH key pairs in parallel across all available CPU cores and tests them against a user-provided regex pattern.

### Key Characteristics

- **Language**: Rust (Edition 2021)
- **Architecture**: Multi-threaded producer-consumer pattern
- **Concurrency**: Lock-free using atomic operations
- **Code Size**: ~250 lines (single file)
- **Performance**: Thousands of keys/second

## Architecture Diagrams

### High-Level System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         SSHChic                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌───────────┐   ┌──────────┐   ┌─────────────────────┐   │
│  │    CLI    │──▶│  Regex   │──▶│  Thread Manager     │   │
│  │  Parser   │   │ Compiler │   │  (Main Thread)      │   │
│  └───────────┘   └──────────┘   └──────────┬──────────┘   │
│                                             │              │
│                         ┌───────────────────┴─────┐        │
│                         │                         │        │
│                  ┌──────▼──────┐           ┌──────▼──────┐ │
│                  │   Worker    │    ...    │   Worker    │ │
│                  │  Thread 1   │           │  Thread N   │ │
│                  └──────┬──────┘           └──────┬──────┘ │
│                         │                         │        │
│                  ┌──────▼─────────────────────────▼──────┐ │
│                  │     Atomic Counter (COUNTER)          │ │
│                  └───────────────────────────────────────┘ │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Monitor Loop (Main Thread - 250ms interval)        │  │
│  │  - Read counter value                               │  │
│  │  - Calculate key generation rate                    │  │
│  │  - Display progress                                 │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Thread Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                            Main Thread                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1. Parse CLI Arguments (clap)                                     │
│  2. Compile Regex Pattern                                          │
│  3. Setup Ctrl+C Handler                                           │
│  4. Create Atomic Shared State:                                    │
│     - running: Arc<AtomicBool>                                     │
│     - COUNTER: AtomicI64                                           │
│  5. Spawn Worker Threads (N = num_cpus)                            │
│  6. Enter Monitor Loop                                             │
│                                                                     │
└──────────────┬──────────────────────────────────────────────────────┘
               │
               │ spawns
               ▼
    ┌──────────────────────┐
    │   Worker Thread 1    │
    ├──────────────────────┤
    │ while running.load() │
    │   ├─ COUNTER++       │
    │   ├─ generate_key()  │
    │   ├─ test_regex()    │
    │   └─ if match:       │
    │      ├─ print()      │
    │      └─ save_files() │
    └──────────────────────┘

    ┌──────────────────────┐
    │   Worker Thread 2    │
    │         ...          │
    └──────────────────────┘

    ┌──────────────────────┐
    │   Worker Thread N    │
    │         ...          │
    └──────────────────────┘

         │ all threads
         │ access
         ▼
    ┌──────────────────────┐
    │  Shared Atomic State │
    ├──────────────────────┤
    │ running: AtomicBool  │
    │ COUNTER: AtomicI64   │
    └──────────────────────┘
```

### Data Flow Diagram

```
┌─────────────┐
│ User Input  │
│ (CLI args)  │
└──────┬──────┘
       │
       ▼
┌─────────────────────┐
│  Regex Compilation  │
│  (?i)?pattern       │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────────────────────┐
│        Worker Thread Loop           │
├─────────────────────────────────────┤
│                                     │
│  ┌──────────────────────┐          │
│  │ Generate Random Seed │          │
│  └──────────┬───────────┘          │
│             ▼                       │
│  ┌──────────────────────┐          │
│  │  ED25519 Key Gen     │          │
│  │  (signing_key,       │          │
│  │   verifying_key)     │          │
│  └──────────┬───────────┘          │
│             ▼                       │
│  ┌──────────────────────┐          │
│  │  Convert to SSH      │          │
│  │  Format or           │◀─────┐   │
│  │  Fingerprint         │      │   │
│  └──────────┬───────────┘      │   │
│             ▼                  │   │
│  ┌──────────────────────┐     │   │
│  │  Regex Match Test    │     │   │
│  └──────────┬───────────┘     │   │
│             │                 │   │
│      ┌──────┴──────┐          │   │
│      │             │          │   │
│   No Match      Match         │   │
│      │             │          │   │
│      │             ▼          │   │
│      │  ┌─────────────────┐  │   │
│      │  │ Display Result  │  │   │
│      │  └─────────────────┘  │   │
│      │             │          │   │
│      │             ▼          │   │
│      │  ┌─────────────────┐  │   │
│      │  │  Save to Files  │  │   │
│      │  │  (if !streaming)│  │   │
│      │  └─────────────────┘  │   │
│      │             │          │   │
│      └─────────────┴──────────┘   │
│             │                      │
│             ▼                      │
│      COUNTER.fetch_add(1) ─────────┘
│                                    │
└────────────────────────────────────┘
```

### Key Generation Flow

```
┌──────────────────────────────────────────────────────────────┐
│                   generate_key_pair()                        │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  thread_rng()                                                │
│       │                                                      │
│       ▼                                                      │
│  ┌────────────────┐                                          │
│  │ Random Bytes   │ (32 bytes seed)                         │
│  │ [CSPRNG]       │                                          │
│  └────────┬───────┘                                          │
│           │                                                  │
│           ▼                                                  │
│  ┌────────────────┐                                          │
│  │ SigningKey     │ (ED25519 private key)                   │
│  │ (32 bytes)     │                                          │
│  └────────┬───────┘                                          │
│           │                                                  │
│           ├──────────────────┐                               │
│           │                  │                               │
│           ▼                  ▼                               │
│  ┌────────────────┐  ┌──────────────┐                       │
│  │ VerifyingKey   │  │ For Signing  │                       │
│  │ (public key)   │  │ Operations   │                       │
│  └────────┬───────┘  └──────────────┘                       │
│           │                                                  │
│           ├────────────────────┬──────────────────┐          │
│           │                    │                  │          │
│           ▼                    ▼                  ▼          │
│  ┌─────────────────┐  ┌──────────────┐  ┌────────────────┐ │
│  │ OpenSSH Format  │  │ Fingerprint  │  │ Regex Matching │ │
│  │ ssh-ed25519 ... │  │ SHA256:...   │  │                │ │
│  └─────────────────┘  └──────────────┘  └────────────────┘ │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. CLI Parser (clap)

**Responsibility**: Parse and validate command-line arguments

**Implementation**:
```rust
#[derive(Parser, Clone)]
struct Args {
    regex: String,
    insensitive: bool,
    streaming: bool,
    fingerprint: bool,
}
```

**Features**:
- Automatic help generation
- Type-safe argument parsing
- Error handling for invalid input

### 2. Regex Compiler

**Responsibility**: Compile user pattern into executable regex

**Implementation**:
```rust
let regex_str = if args.insensitive {
    format!("(?i){}", args.regex)
} else {
    args.regex
};
let regex = Regex::new(&regex_str)?;
```

**Optimizations**:
- Compiled once, shared across threads (via `Arc<Regex>`)
- Cloned cheaply (internal Arc)

### 3. Key Generator

**Responsibility**: Generate cryptographically secure ED25519 key pairs

**Functions**:
- `generate_key_pair()` - Creates signing and verifying keys
- `get_authorized_key()` - Converts to OpenSSH format
- `get_fingerprint()` - Computes SHA256 fingerprint

**Security**:
- Uses `rand::thread_rng()` (ChaCha CSPRNG)
- Each thread has independent RNG state

### 4. Pattern Matcher

**Responsibility**: Test generated keys against regex pattern

**Implementation**:
```rust
let matched = if args.fingerprint {
    regex.is_match(&get_fingerprint(&verifying_key))
} else {
    regex.is_match(&get_authorized_key(&verifying_key))
};
```

**Performance**:
- Fingerprint matching is faster (shorter string)
- Public key matching is more common use case

### 5. File Writer

**Responsibility**: Save matching keys to disk

**Files Generated**:
- `id_ed25519` - Private key (OpenSSH format, 600 permissions recommended)
- `id_ed25519.pub` - Public key (OpenSSH format)

**Behavior**:
- Only in non-streaming mode
- Overwrites existing files without warning

### 6. Monitor Loop

**Responsibility**: Display real-time progress and statistics

**Metrics**:
- Keys processed (human-readable format)
- Generation rate (kKeys/s)
- Exponential moving average (5s window)

**Update Frequency**: 250ms

## Concurrency Model

### Lock-Free Architecture

SSHChic uses atomic operations instead of mutexes for better performance:

```rust
static COUNTER: AtomicI64 = AtomicI64::new(0);
let running = Arc<AtomicBool>::new(true);
```

**Advantages**:
- No lock contention
- Better CPU utilization
- Simpler code (no deadlock risk)

**Atomic Operations**:
- `fetch_add(1, Ordering::SeqCst)` - Increment counter
- `load(Ordering::SeqCst)` - Read running flag
- `store(false, Ordering::SeqCst)` - Stop threads

### Memory Ordering

We use `SeqCst` (Sequential Consistency) for simplicity and correctness:
- Ensures total order of operations
- Prevents unexpected reordering
- Slight performance cost acceptable for this use case

### Thread Lifecycle

```
Main Thread                   Worker Thread
    |                              |
    ├─ Parse args                  |
    ├─ Compile regex               |
    ├─ Create atomic state         |
    ├─ spawn() ─────────────────▶  |
    |                              ├─ Clone regex/args/running
    |                              ├─ Enter loop
    |                              │  while running.load()
    |                              │    ├─ COUNTER++
    |                              │    ├─ Generate key
    |                              │    ├─ Test regex
    |                              │    └─ On match: save & exit
    |                              │
    ├─ Enter monitor loop          │
    │  while running.load()        │
    │    ├─ Read COUNTER            │
    │    ├─ Calculate rate          │
    │    └─ Display progress        │
    |                              |
    ├─ Ctrl+C ───────────────────▶ |
    ├─ running.store(false)        ├─ Exit loop
    |                              |
    ├─ join() ◀───────────────────  Exit thread
    |
    └─ Exit
```

## Data Flow

### Input Flow

1. **CLI Arguments** → Parsed by clap
2. **Regex Pattern** → Compiled to `Regex`
3. **Flags** → Converted to `Args` struct

### Processing Flow

1. **Random Seed** → Thread-local RNG
2. **Key Generation** → ED25519 algorithm
3. **Format Conversion** → OpenSSH or fingerprint
4. **Pattern Matching** → Regex test
5. **Output** → Display and/or file save

### Control Flow

- **Graceful Shutdown**: Ctrl+C → `running.store(false)` → threads exit
- **Match Found**: Display → (if !streaming) save files and exit
- **Error Handling**: Invalid regex → error message and exit

## Performance Characteristics

### Throughput

**Typical Performance** (8-core CPU, simple regex):
- ~10,000 - 50,000 keys/second
- Scales linearly with CPU cores

**Factors Affecting Performance**:
1. **CPU Cores**: More cores = higher throughput
2. **Regex Complexity**: Complex patterns slow matching
3. **Match Mode**: Fingerprint slightly faster than public key
4. **CPU Clock Speed**: Higher clock = faster crypto

### Bottlenecks

1. **ED25519 Key Generation**: CPU-bound cryptographic operation
2. **Regex Matching**: Depends on pattern complexity
3. **String Formatting**: Converting keys to SSH format

### Memory Usage

**Per-Thread Memory**:
- RNG state: ~200 bytes
- Stack: ~2 MB (default)
- Key data: ~100 bytes

**Total Memory** (8 threads): ~20 MB typical

### Optimizations

1. **Multi-threading**: Utilizes all CPU cores
2. **Lock-free atomics**: No mutex contention
3. **Regex cloning**: Cheap Arc-based cloning
4. **Minimal allocations**: Reuses thread-local RNG

## Security Considerations

### Cryptographic Security

1. **Random Number Generation**:
   - Uses `rand::thread_rng()` (ChaCha20-based CSPRNG)
   - Cryptographically secure for key generation
   - Independent seed per thread

2. **ED25519 Algorithm**:
   - Industry-standard elliptic curve cryptography
   - Implemented by `ed25519-dalek` (audited library)
   - Resistant to timing attacks

3. **Key Storage**:
   - Private keys saved to disk in OpenSSH format
   - **User responsibility** to set proper permissions (chmod 600)
   - No password protection (user should add via ssh-keygen)

### Potential Security Issues

1. **File Overwrite**: Existing `id_ed25519` files are overwritten without warning
   - **Mitigation**: Document clearly, consider adding confirmation

2. **Pattern Correlation**: Vanity keys may reduce effective entropy
   - **Impact**: Minimal for short patterns (e.g., 3-4 chars)
   - **Risk**: Longer patterns reduce search space

3. **Regex DoS**: Catastrophic backtracking in complex regex
   - **Mitigation**: User-provided patterns, timeout not implemented
   - **Risk**: Low (affects only user's own system)

## Design Decisions

### Why Single-File Architecture?

**Pros**:
- Simple to understand and navigate
- Fast compilation
- Easy to audit

**Cons**:
- Harder to test individual components
- Limited code organization

**Decision**: Prioritized simplicity for this tool's scope

### Why Lock-Free Atomics?

**Alternatives Considered**:
- Mutex-protected counter
- Channel-based communication

**Chosen**: Atomics
- Simplest implementation
- Best performance
- Sufficient for this use case

### Why ED25519 Only?

**Alternatives**: RSA, ECDSA

**Chosen**: ED25519
- Modern, secure algorithm
- Fast key generation
- Small key size
- Widely supported

**Future**: Could add other algorithms via feature flags

### Why No Configuration File?

**Decision**: CLI-only for simplicity
- Tools like this are typically one-off use
- Configuration adds complexity
- Environment variables could be added if needed

### Why Overwrite Files?

**Alternatives**:
- Prompt for confirmation
- Generate unique filenames
- Write to stdout only

**Chosen**: Overwrite
- Standard SSH key naming convention
- Matches ssh-keygen behavior
- Documented in help text

**Future**: Could add `-o/--output` flag for custom names

## Extension Points

### Adding New Features

1. **Additional Algorithms**:
   - Add new key generation functions
   - Update CLI with algorithm selection flag
   - Update file naming scheme

2. **Custom Output Formats**:
   - Implement format conversion functions
   - Add format selection flag
   - Update file writer

3. **Configuration File**:
   - Add config parsing (e.g., TOML via serde)
   - Define config schema
   - Merge with CLI args (CLI takes precedence)

4. **Progress Bar**:
   - Replace monitor loop output
   - Use `indicatif` crate
   - Show estimated time remaining

### Modularization Path

If the project grows, suggested module structure:

```
src/
├── main.rs           # CLI entry point
├── lib.rs            # Library exports
├── cli.rs            # Argument parsing
├── crypto/
│   ├── mod.rs
│   ├── ed25519.rs    # ED25519 operations
│   ├── rsa.rs        # (Future) RSA operations
│   └── fingerprint.rs
├── matcher.rs        # Regex matching logic
├── monitor.rs        # Progress monitoring
└── output.rs         # File writing
```

## Diagrams Summary

This document includes:

1. **High-Level System Architecture** - Overall component structure
2. **Thread Architecture** - Concurrency model
3. **Data Flow Diagram** - How data moves through the system
4. **Key Generation Flow** - Cryptographic operations
5. **Thread Lifecycle** - Thread creation and termination

For implementation details, see the inline rustdoc comments in `src/main.rs`.

---

*Last Updated: 2025*
*Document Version: 1.0*
