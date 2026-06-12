Contrax Server
Contrax Server is a configuration-driven application runtime engine built in Rust. Instead of hardcoding network endpoints and business logic routines into a compiled binary, the server parses a declarative JSON manifest at startup to dynamically assemble its routing matrix and asynchronous execution pipelines.

By separating the network transport layer from core application logic, Contrax is structurally protocol-agnostic—capable of orchestrating HTTP requests, persistent WebSocket connections, or real-time UDP datagram streams through a single, unified execution loop.

🏗️ System Architecture & Data Flow
The engine is engineered around three structural layers designed to isolate execution state, enforce type constraints across dynamic boundaries, and minimize allocation overhead.

                  ┌────────────────────────────────────────┐
                  │          Inbound Network Frame         │
                  │        (HTTP / WebSocket / UDP)        │
                  └───────────────────┬────────────────────┘
                                      │
                                      ▼
                  ┌────────────────────────────────────────┐
                  │       DynamicContext Allocation        │
                  │    - Shared State (Arc<RwLock<>>)      │
                  └───────────────────┬────────────────────┘
                                      │
                                      ▼
                  ┌────────────────────────────────────────┐
                  │     Pipeline Processing Execution      │
                  │     Vec<Arc<dyn DynamicContract>>       │
                  └───────────┬────────────────────┬───────┘
                              │                    │
            [Step 1: Extractor Wrapper]  [Step 2: Concrete Logic Execution]
                              │                    │
                              ▼                    ▼
                  ┌──────────────────────┐  ┌──────────────────────┐
                  │  Argument Mapping    │  │  Struct Validation   │
                  │  & JSON Deserialization│  │  & Mutable Commit    │
                  └──────────────────────┘  └──────────────────────┘
1. The Dynamic Context (Data Isolation Layer)
Upon intercepting a network frame, the server instantiates a universal DynamicContext. This structure acts as the single source of truth for the duration of a request pipeline. It encapsulates an isolated, thread-safe memory registry wrapped in an atomic reader-writer lock (Arc<RwLock<serde_json::Map<String, Value>>>). This ensures that concurrent tasks or sequential pipeline steps can safely read from and mutate state parameters without data races.

2. The Contract Blueprint (Type Erasure & Abstraction)
Execution steps—such as input validation, request decoration, or domain-specific business logic—are decoupled into individual components implementing the InnerContract blueprint.

To manage these heterogeneous blocks within a single sequential pipeline vector, the runtime utilizes type erasure via dynamic dispatch (dyn DynamicContract). This allows the server to compile diverse logical routines into a uniform Vec<Arc<dyn DynamicContract>> at boot time, executing them sequentially without needing to know their concrete underlying structures at compilation.

3. The Marshalling Wrapper (Data Pipeline Gateway)
Because core contracts require strictly-typed structures to execute safely, the runtime bridges the gap between raw manifest arguments and typed logic using a generic container wrapper (ManifestContractWrapper<T: InnerContract>).

When a pipeline step executes, the wrapper performs automated data marshalling:

It resolves declarative path pointers against the DynamicContext.

It evaluates and maps raw arguments into an intermediate JSON structure.

It deserializes the arguments directly into the contract's explicit Input associated type using bounded generics.

It triggers structural constraints verification and dispatches the data to the target execution block, returning the output back to the context registry.

⚡ Concurrency & Memory Model
Contrax Server is engineered to optimize performance across multi-threaded asynchronous runtimes like Tokio:

Heap-Allocated Futures: Because asynchronous functions in Rust compute anonymous, unnamable types at compile time, traits cannot natively yield standard futures. Contrax resolves this by utilizing BoxFuture<'static, Result<T, E>> type boundaries, enabling seamless dynamic dispatch across thread lines.

Thread-Safety Constraints: Every abstract contract layer strictly enforces Send + Sync markers. This guarantees that the entire routing matrix can be safely distributed and executed across a shared multi-threaded CPU worker pool.

Failure Boundaries: The execution loop operates via a strict type-state boundary. Validation faults or internal thread panics originating inside an isolated contract step are caught cleanly at the wrapper margin. This prevents cascading crashes and allows the server to terminate the current pipeline while keeping adjacent connections active.

🔮 Ahead-of-Time WASM (CWASM) Roadmap
The runtime layout is architected to decouple plugin development entirely from the core engine binary using WebAssembly.

By leveraging an Ahead-of-Time (AOT) compilation paradigm via wasmtime's .cwasm format, the server will ingest pre-compiled native machine artifacts generated from any language supporting a WebAssembly target toolchain (Rust, C++, Go, Zig). The engine will stream payloads directly into the WASM instance's isolated linear memory spaces, achieving total cross-language plugin hot-swapping at near-native execution velocity while preserving absolute sandbox security.
