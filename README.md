Contrax Server
Contrax Server is a lightweight, configuration-driven application server built in Rust. Instead of hardcoding API routes and business logic directly into the application source code, this engine parses a declarative JSON manifest at startup to dynamically assemble network endpoints and asynchronous execution pipelines.

By separating the transport network layer from the core application logic, the runtime provides a decoupled architecture capable of orchestrating web APIs, persistent socket connections, or real-time network traffic through a single, stateless binary.

🛠️ Core Architectural Components
The engine is split into three distinct structural layers designed to safely manage data flow and multi-threaded execution:

1. The Dynamic Context (Data Layer)
When a network frame hits the server, it is immediately translated into a universal DynamicContext. This context contains an isolated, thread-safe memory space (Arc<RwLock<...>>) that acts as a shared state pool for the duration of that specific request pipeline.

2. The Contract Blueprint (Abstraction Layer)
Every execution step—whether it is a data validation check, an authentication layer, or a database handler—is modeled as a "Contract." The server relies on an explicit trait architecture (DynamicContract) to enforce strict type-erasure (Dynamic Dispatch). This allows heterogenous blocks of logic to be stored, managed, and executed sequentially within a single asynchronous runtime vector.

3. The Marshalling Wrapper (Execution Layer)
Because contracts require strongly-typed data structures to run safely, the engine utilizes a generic container wrapper (ManifestContractWrapper<T: InnerContract>). This wrapper acts as an automated data pipeline: it extracts un-typed JSON arguments mapped by the manifest, deserializes them directly into the contract's explicit target Input struct, runs structural validation checks, and securely commits the logic back to the shared memory pool.

🧠 Systems Engineering Showcase
This repository serves as a portfolio piece demonstrating intermediate-to-advanced systems programming patterns in Rust, specifically focused on building production-grade infrastructure:

Asynchronous Lifetimes & Future Boxing: Leverages BoxFuture<'static, ...> heap allocations to overcome trait limitations with asynchronous functions, ensuring clean compatibility with the multi-threaded Tokio runtime.

Thread-Safe Concurrency Boundaries: Enforces strict Send + Sync marker traits across all dynamic steps, guaranteeing that the server can safely distribute incoming connections across multiple CPU cores without data races or memory corruption.

Memory and Fault Isolation: Built using a type-state pattern that ensures validation failures or unexpected thread panics are safely caught within the execution boundary of an isolated contract node, preventing cascades that could crash the entire server.

Protocol-Agnostic Design: The internal execution loop functions completely independently of the network protocol frame. The engine is natively architected to process HTTP requests via Actix-web, persistent WebSockets, or high-frequency game-server UDP datagrams through the exact same underlying logic pipelines.

🚀 Future Roadmap: Ahead-of-Time WASM (CWASM)
The architecture is actively scaling toward decoupling contract development entirely from the main codebase via WebAssembly.

By migrating to an Ahead-of-Time (AOT) compilation model using wasmtime's .cwasm format, external developers will be able to author execution pipelines in any systems language (Rust, C++, Go, Zig), compile them to native architecture-specific instructions, and drop them into a running server. This achieves total runtime plugin extensibility and native-level execution speed, all while maintaining WebAssembly’s strict sandbox isolation boundaries.
