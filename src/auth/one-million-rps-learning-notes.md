# Handling 1 Million Requests per Second

> **Readable learning notes derived from the supplied transcript**  
> The focus of the video is not a copy-paste production architecture. It is the engineering thought process required when performance reaches an extreme scale: **measure, locate the real bottleneck, change the architecture, and measure again.**

---

## 🧭 What This Experiment Is Trying to Answer

The experiment asks a deceptively simple question:

> **What does it actually take to handle 1,000,000 HTTP requests per second?**

At that scale, small inefficiencies stop being small.

The speaker emphasizes that:

- an algorithmic choice such as `O(n)` instead of `O(log n)` can become financially significant;
- assumptions such as “the database is always the bottleneck” may be completely wrong;
- CPU, memory, disk, networking, runtime overhead, framework overhead, database behavior, and even the load generator can independently become the limiting factor;
- resource monitoring is mandatory because guessing becomes useless at this scale;
- solutions that “work” functionally may still be unacceptable economically.

The video therefore treats performance engineering as a sequence of experiments rather than a single optimization exercise.

---

# 1. Foundations Before Benchmarking

## 1.1 CPU Core Utilization

For a single CPU core, the video describes utilization as:

```text
core utilization = (total time - idle time) / total time × 100
```

If a core was idle for 30 minutes during a one-hour window:

```text
(60 - 30) / 60 × 100 = 50%
```

So the core was busy half of the time.

### Two Common Ways CPU Usage Is Displayed

For a multi-core CPU, tools may report utilization differently.

| Reporting style | Example with 4 fully utilized cores |
|---|---:|
| Sum utilization of all cores | `400%` |
| Normalize across all cores | `100%` |

This distinction matters later when processes appear to use values such as `900%` CPU.

---

## 1.2 Why Threads Matter

A single CPU-bound thread can only execute on one CPU core at a time.

The video demonstrates this with an infinite loop:

```text
while true
```

One such thread saturates roughly one core.

If the machine has 12 cores and 12 CPU-bound threads are created, the machine can potentially keep all 12 cores busy simultaneously.

But concurrency introduces its own engineering problems:

- race conditions;
- synchronization;
- semaphores or other coordination mechanisms;
- shared-state correctness.

The important lesson is not simply “add threads.” It is:

> **Parallelism only helps when there is unused CPU capacity and the workload can safely be distributed.**

---

# 2. Understanding the Load Test

The video uses **Autocannon** to generate HTTP traffic.

A simplified command contains parameters such as:

```bash
autocannon \
  -c <connections> \
  -d <duration> \
  -p <pipeline> \
  -w <workers> \
  -m <method> \
  <url>
```

## Important Parameters

| Parameter | Meaning |
|---|---|
| `-c` | Number of open connections |
| `-d` | Test duration |
| `-p` | Number of pipelined requests per connection |
| `-w` | Number of worker threads generating load |
| `-m` | HTTP method |

The transcript uses the following mental model for concurrent in-flight requests:

```text
concurrent requests ≈ connections × pipeline
```

Example:

```text
connections = 6
pipeline    = 2

6 × 2 = 12 concurrent requests
```

Workers determine how the load generation itself is spread across CPU threads.

### Benchmarking Rule

A load test is only useful when the **load generator itself is not the bottleneck**.

This becomes important later, when a single tester can no longer open enough connections to fully exercise the server.

---

# 3. Establishing a Local Baseline

The first endpoint is intentionally trivial:

```http
GET /simple
```

Response:

```json
{
  "message": "hi"
}
```

The point is to determine the raw HTTP handling capacity before introducing database work or substantial response payloads.

## Framework Comparison

The speaker compares multiple Node.js HTTP approaches using equivalent route behavior.

| Server/framework | Approximate result shown in the transcript |
|---|---:|
| Express | ~20k RPS |
| Fastify | ~66k RPS |
| Cpeak | ~73k RPS |

The exact number changes between runs, but the important observation is that **framework overhead becomes visible at high request volumes**.

Cpeak is then used for much of the Node.js testing because the speaker considers its performance close to Fastify/raw Node while keeping the implementation small and understandable.

### Engineering Lesson

At ordinary traffic levels, a framework difference may not matter.

At tens or hundreds of thousands of requests per second, even small per-request overhead multiplies dramatically.

---

# 4. A More Realistic Route Changes Everything

The experiment then moves from a tiny JSON response to a route that does more work:

- parses path variables;
- handles query parameters;
- accepts a request body;
- performs validation/dummy computation;
- generates a much larger response;
- returns roughly **32 KB of JSON**.

The result drops dramatically.

### Single Node Process

Approximate result:

```text
~8,000 RPS
```

The process is saturating a CPU core, while the rest of the machine still has available CPU capacity.

---

# 5. Using All CPU Cores: Node Cluster Mode

The application is started in multiple Node.js processes with **PM2**, roughly one process per core.

On the 12-core local machine:

```text
1 process  → ~8k RPS
12 processes → ~36k RPS
heavier load → ~42k–50k RPS
```

No major application logic changed.

The performance improvement came mainly from making previously idle CPU cores useful.

### Key Idea

> Before buying more hardware, verify that the hardware you already have is actually being utilized.

---

# 6. Moving the Experiment to Powerful Cloud Machines

The local machine is no longer enough for the experiment, so the test moves to AWS.

The initial architecture is conceptually:

```text
                   private network

┌─────────────────┐           ┌─────────────────┐
│  Power Tester   │ ────────► │  Power Server   │
│ generates load  │           │ handles traffic │
└─────────────────┘           └────────┬────────┘
                                      │
                                      ▼
                             ┌─────────────────┐
                             │   PostgreSQL    │
                             └─────────────────┘
```

The tester and server are separated so that generating traffic does not consume the server's CPU.

## Initial Power Server

The transcript uses a `C8i.32xlarge`-class machine with approximately:

- 128 CPU cores;
- 256 GB RAM;
- 50 Gbit/s networking;
- roughly `$6/hour` in the example.

A similarly powerful machine is used as the traffic generator.

---

# 7. The Trivial Route Blows Past 1 Million RPS

The `/simple` route is tested first.

The power server runs a very large number of Node processes, and the tester generates substantial parallel traffic.

Result:

```text
~6 million requests per second
```

This proves several important things:

1. The network between the machines can support very high traffic for a tiny payload.
2. The tester can generate millions of requests per second.
3. The server can process millions of extremely cheap HTTP requests.
4. If later endpoints perform poorly, the cause must be something introduced by those endpoints.

This creates a **known-good baseline**.

---

# 8. The Large-Payload Route Hits a Network Wall

The larger PATCH route performs dramatically worse:

```text
~100k RPS
```

Initially, this looks like a CPU problem.

But CPU usage is not fully saturated.

The crucial observation comes from the amount of transferred data:

```text
~119 GB transferred in 20 seconds
≈ 5.9 GB/s
```

The machine's network capability is around:

```text
50 Gbit/s ≈ 6.25 GB/s
```

So the system is effectively saturating the network interface.

## Bottleneck Identified

```text
NOT CPU
NOT application threads
NOT database

→ NETWORK BANDWIDTH
```

### Why This Matters

If each request returns approximately 30 KB, then one million responses per second imply roughly:

```text
30 KB × 1,000,000
≈ 30 GB/s
```

That is an extraordinary amount of network traffic.

You cannot optimize your JavaScript loop enough to overcome a physical bandwidth ceiling.

---

# 9. Scale-Out Is Often More Realistic Than One Giant Server

The transcript points out that organizations operating at massive global scale generally do not route all traffic through one gigantic computer.

A more realistic model is:

```text
Users
  │
  ▼
Traffic routing / load balancing
  │
  ├── Region A servers
  ├── Region B servers
  ├── Region C servers
  └── Region D servers
```

Traffic can be distributed geographically and across multiple machines.

This changes the engineering question from:

> “How can one machine serve one million requests per second?”

into:

> “How can the system as a whole serve one million requests per second?”

---

# 10. PostgreSQL: Where Storage Becomes Expensive

The next route performs a database write.

Conceptually:

```text
HTTP request
   ↓
generate code
   ↓
INSERT INTO PostgreSQL
   ↓
return response
```

The PostgreSQL machine used in the experiment is already very large:

- dozens of CPU cores;
- hundreds of GB of RAM;
- high-performance cloud storage;
- thousands of dollars per month.

Yet database writes remain far below the desired target.

Approximate results discussed in the transcript:

```text
PostgreSQL writes: ~30k–50k RPS
PostgreSQL reads:  ~400k RPS for efficient ID lookup
```

Increasing storage IOPS raises cost substantially but does not solve the core problem because the database CPU also reaches saturation.

---

# 11. A Database Query Can Destroy the Entire Benchmark

Several versions of a database-read route are tested.

Some implementations attempt operations such as:

- counting every row;
- selecting random records inefficiently;
- scanning large portions of a table.

With roughly 10 million records, these operations become extremely expensive.

The speaker specifically highlights `COUNT(*)`-style work over a huge table as effectively `O(n)` when the database must scan records rather than simply retrieving a maintained count.

A much cheaper approach is an indexed lookup by ID.

That version reaches approximately:

```text
~400k reads/sec
```

### Core Lesson

> At large scale, query complexity is part of application complexity.

A logically correct SQL query may still be operationally disastrous.

---

# 12. Why “Just Scale PostgreSQL More” Becomes Unattractive

The experiment estimates increasingly expensive PostgreSQL configurations.

The problem is not that database scaling is impossible.

The problem is that the price-performance curve becomes unpleasant.

The transcript discusses configurations costing roughly:

```text
$14k/month
$20k–30k/month
$30k+/month
```

for progressively more aggressive database setups.

And write throughput is still far away from one million writes per second.

This leads to a change in architecture rather than simply buying a larger SQL machine.

---

# 13. Redis Changes the Storage Strategy

The next step is to move the hot path from disk-backed PostgreSQL to **Redis**, an in-memory data store.

Instead of:

```text
request → PostgreSQL → response
```

use:

```text
request → Redis → response
              │
              ▼
         synchronization queue
              │
              ▼
        PostgreSQL later
```

The important architectural change is that the synchronous request path no longer needs to wait for the slower durable database operation.

---

# 14. Asynchronous Persistence

The application writes incoming data to Redis and also records IDs in a synchronization queue.

A separate process can later:

1. read queued items;
2. batch them;
3. write them to PostgreSQL;
4. remove/process synchronized entries.

The synchronization can happen continuously in the background rather than making every request wait on disk-backed storage.

### Why Batching Helps

Instead of paying database overhead for each request individually:

```text
write
write
write
write
write
```

perform something closer to:

```text
Redis: accept quickly
Redis: accept quickly
Redis: accept quickly
Redis: accept quickly

        ↓ later

PostgreSQL: batch write
```

This separates **ingestion speed** from **durable persistence speed**.

---

# 15. Moving Existing PostgreSQL Data Into Memory

The transcript has roughly:

```text
~11 million rows
~16 GB table size
```

The server has hundreds of gigabytes of RAM, so the full working dataset can fit in memory.

A migration script copies data from PostgreSQL to Redis in batches.

After migration, Redis memory usage is around:

```text
~20 GB
```

The extra memory accounts for Redis structures and metadata in addition to the raw table content.

### Important Design Principle

You do not necessarily need to move the entire database to Redis.

A practical approach is to move only **hot data or heavily accessed tables**.

---

# 16. Single Redis Instance: Faster, But Still a Ceiling

A single Redis instance improves write performance substantially compared with PostgreSQL.

Approximate values shown:

```text
Redis write: ~100k–150k RPS
Redis read:  ~300k–400k RPS
```

But CPU remains underutilized.

Why?

Because the transcript treats a single Redis instance as effectively single-threaded for this workload, so one instance becomes the bottleneck.

The machine has many cores, but one Redis process cannot exploit all of them for request processing.

---

# 17. Redis Cluster: Scale the In-Memory Layer Horizontally

The experiment launches many Redis processes and configures them as a cluster.

The example uses:

```text
30 Redis nodes
15 masters
15 replicas
```

Conceptually:

```text
                 ┌── Redis Master 1 ── Replica 1
Application ─────┼── Redis Master 2 ── Replica 2
                 ├── Redis Master 3 ── Replica 3
                 └── ...
```

Redis hashes keys into slots so that data is distributed across nodes.

This means requests are no longer constrained by a single Redis process.

---

# 18. Removing Sequential ID Coordination

At one million operations per second, even an extra uniqueness check matters.

The earlier approach keeps a sequential ID / uniqueness structure in Redis.

That introduces additional coordination and writes.

The optimized route instead generates random UUIDs.

The transcript discusses a 122-bit random UUID space and uses the birthday-paradox idea to argue that collision risk is negligible for the experiment.

The key engineering point is:

> **Avoid centralized coordination when a sufficiently large random identifier space can remove the coordination requirement.**

That removes work from the critical request path.

---

# 19. Redis Cluster Reaches 1 Million Writes per Second

The application is run across the available CPU cores while writing to the Redis cluster.

Result:

```text
~1,000,000 requests/sec
```

The route is accepting writes into Redis while durable synchronization can be performed separately.

This is the first meaningful architecture in the video that reaches the target while still involving a data-storage workflow.

### But a New Problem Immediately Appears

At one million writes every second, memory consumption becomes enormous.

The transcript observes tens of millions of Redis records and roughly **100 GB of memory usage**.

That means the system must continuously drain or expire data.

Otherwise:

```text
high write throughput
      ↓
rapid RAM growth
      ↓
RAM exhaustion
```

Achieving the target does not eliminate system constraints—it simply reveals the next one.

---

# 20. Returning to the 30 KB Response Problem

The Redis experiment solves a storage bottleneck, but the large-response endpoint remains difficult.

The target route involves roughly:

- request parsing;
- validation;
- JSON processing;
- string/data generation;
- about 30 KB of response data;
- extremely high network throughput.

The speaker moves to a much larger network-optimized AWS instance class:

```text
C8gn.48xlarge
~192 CPU cores
~384 GB RAM
~600 Gbit/s networking
```

Two such systems—one server and one tester—represent a very expensive setup.

---

# 21. Node.js Gets Close, But Runtime Overhead Matters

On the network-optimized server, Node.js can push enormous traffic.

However, the CPU saturates before the endpoint reaches one million requests per second.

The transcript attributes part of the limitation to the architecture in which traffic reaches a parent Node process and is then distributed among child processes.

Express performs significantly worse.

Cpeak/Fastify get much closer, but still do not reach the target for this CPU-heavy, large-response route in the tested setup.

### Critical Observation

This is not a claim that Node.js is “bad.”

The experiment demonstrates that when:

- per-request CPU work is significant;
- request rates are extreme;
- response payloads are large;
- every small overhead is multiplied one million times per second;

runtime and framework overhead become first-class constraints.

---

# 22. Rewriting the Hot Route in C++

The route is rewritten using:

- **C++**;
- the **Drogon** web framework;
- **RapidJSON** for JSON parsing.

Interestingly, the initial C++ implementation is not automatically faster.

The default JSON parser used in the initial Drogon setup is described as slower than the V8 JSON path used by Node.js in this benchmark.

Only after replacing the parser and continuing to optimize the implementation does the C++ version achieve the target.

### Important Lesson

> Choosing a “faster language” does not automatically produce a faster application.

Libraries, parsing, memory allocation, serialization, logging, compression, networking, and application structure still matter.

---

# 23. C++ Hits the Large-Response Target

The optimized C++ server runs for 60 seconds.

Approximate result:

```text
Average: ~1,000,000 RPS
Peak:    ~1,200,000 RPS
```

The application is moving roughly:

```text
~38 GB/s
≈ 300 Gbit/s
```

The transcript also reports roughly:

```text
~2 TB of application-layer data in one minute
```

The server still has some idle CPU, indicating that another system limit may be preventing further utilization.

Possible suspects discussed include:

- network stack limits;
- connection-generation limits;
- cloud infrastructure limits;
- Linux tuning;
- load-balancer limits.

Again, reaching one bottleneck exposes the next bottleneck.

---

# 24. Even the Load Generator Becomes the Bottleneck

The speaker notices that the powerful server still has approximately 30% idle CPU.

The problem appears to be that a single traffic-generating machine cannot open enough connections to fully load the server.

So the architecture changes again.

Instead of one enormous tester:

```text
1 large tester → server
```

use many smaller testers:

```text
Tester 1  ─┐
Tester 2  ─┤
Tester 3  ─┤
...        ├────► Beast Server
Tester 60 ─┘
```

The final experiment launches approximately **60 smaller EC2 instances** to generate traffic simultaneously.

This is an important benchmarking principle:

> **The system under test is not the only system that must scale. Your benchmarking infrastructure must scale too.**

---

# 25. Large Distributed Stress Test

The final setup coordinates load generation across many machines.

A Bash script is used to:

- trigger Autocannon on all testers;
- collect output;
- push or retrieve logs;
- aggregate results;
- analyze totals.

The transcript later combines results using shell tools such as:

```bash
grep
wc
awk
cat
find
```

This demonstrates why shell scripting becomes valuable in infrastructure and performance work: repetitive operational tasks become programmable.

---

# 26. Final Stress-Test Results

The large distributed test runs for roughly **30 minutes**.

Aggregated result from the transcript:

```text
~2 billion requests
~60+ TB transferred
~40 timeouts
```

The speaker notes that the test used extremely large connection counts and that the server remained stable through the run.

The purpose of the long-duration test is different from a 20-second peak benchmark.

A short test answers:

> “Can it reach this throughput?”

A longer test begins to answer:

> “Can it sustain this throughput without collapsing?”

---

# 27. Benchmark Progression at a Glance

> Values below are approximate figures reported during the transcript. They are best understood as checkpoints in the experiment, not universal performance numbers.

| Stage | Main idea | Approx. throughput |
|---|---|---:|
| Express, simple local route | Basic Node HTTP | ~20k RPS |
| Fastify, simple local route | Lower framework overhead | ~66k RPS |
| Cpeak, simple local route | Near raw Node performance | ~73k RPS |
| Realistic 32 KB route, one Node process | CPU-bound single process | ~8k RPS |
| Same route, clustered Node | Use more CPU cores | ~36k–50k RPS |
| `/simple` on large cloud server | Tiny payload, huge hardware | ~6M RPS |
| 32 KB route on 50 Gbit/s server | Network saturated | ~100k RPS |
| PostgreSQL writes | Disk/database path | ~30k–50k RPS |
| PostgreSQL indexed reads | Efficient lookup | ~400k RPS |
| Single Redis write | In-memory hot path | ~100k–150k RPS |
| Single Redis read | In-memory read | ~300k–400k RPS |
| Redis cluster write path | Horizontal in-memory scaling | ~1M RPS |
| Optimized C++ large-response route | Drogon + RapidJSON | ~1M avg, ~1.2M peak |

---

# 28. The Bottleneck Ladder

One of the clearest lessons from the experiment is that bottlenecks move.

```text
Framework overhead
      ↓
Single-core CPU utilization
      ↓
Whole-machine CPU utilization
      ↓
Network bandwidth
      ↓
Database query complexity
      ↓
Database CPU / storage
      ↓
Single Redis instance
      ↓
Redis memory capacity
      ↓
Runtime / parser overhead
      ↓
Traffic generator capacity
      ↓
OS / cloud / connection limits
```

Optimization is therefore not:

```text
make code faster once
```

It is:

```text
measure
  ↓
find the current bottleneck
  ↓
remove or move it
  ↓
measure again
  ↓
find the new bottleneck
  ↓
repeat
```

---

# 29. The Most Important Engineering Mindset

## Measure Before Optimizing

Never assume the bottleneck.

Examples from the experiment:

- CPU looked suspicious, but the real limit was network bandwidth.
- Faster disk did little because PostgreSQL CPU was already saturated.
- More server CPU could not help when the tester could not create enough load.
- Moving PostgreSQL data to Redis helped, but one Redis process then became the limit.

---

## Monitor Every Resource

The video repeatedly checks:

- CPU utilization;
- idle CPU;
- process-level CPU;
- RAM consumption;
- network throughput;
- disk IOPS;
- database connections;
- database CPU;
- request errors/timeouts;
- total transferred data.

Without this data, performance tuning becomes speculation.

---

## Think in Multiplication

At one million requests per second:

```text
1 extra operation/request
× 1,000,000 requests/sec
= 1,000,000 extra operations/sec
```

Likewise:

```text
30 KB response
× 1,000,000 requests/sec
≈ 30 GB/sec
```

Tiny per-request costs become giant system costs.

---

## Algorithmic Complexity Becomes Operational Cost

An inefficient query or algorithm is no longer merely “slower.”

It may require:

- more CPU;
- more database instances;
- more network capacity;
- more RAM;
- more cloud spending.

At sufficient scale, algorithm choice directly affects infrastructure cost.

---

## Scale Up and Scale Out Are Different Tools

### Scale Up

Use a bigger machine:

```text
more CPU
more RAM
faster network
```

Advantages:

- operationally simpler;
- easy to test;
- fewer distributed-system concerns.

But there is eventually a hardware and cost ceiling.

### Scale Out

Use more machines:

```text
more application servers
more Redis shards
more traffic generators
regional distribution
```

This allows growth beyond a single machine but introduces distributed-system complexity.

---

# 30. Architectural Pattern Extracted From the Experiment

A generalized version of the successful ideas in the transcript looks like this:

```text
                         ┌──────────────────┐
Clients ───────────────► │ Load Distribution│
                         └────────┬─────────┘
                                  │
                 ┌────────────────┼────────────────┐
                 │                │                │
                 ▼                ▼                ▼
          App Instance 1   App Instance 2   App Instance N
                 │                │                │
                 └────────────────┼────────────────┘
                                  │
                                  ▼
                         ┌──────────────────┐
                         │  Redis Cluster   │
                         │  hot-path data   │
                         └────────┬─────────┘
                                  │
                         async / batched sync
                                  │
                                  ▼
                         ┌──────────────────┐
                         │   PostgreSQL     │
                         │ durable storage  │
                         └──────────────────┘
```

The specific tools are less important than the responsibilities:

- distribute traffic;
- parallelize application work;
- keep hot operations fast;
- defer slow persistence where the use case allows it;
- batch expensive work;
- continuously monitor resource saturation.

---

# 31. What the Video Is *Not* Saying

The transcript repeatedly makes the experiment intentionally extreme.

It is **not** presenting one universal architecture that every application should copy.

The takeaway is not:

```text
Use C++ + Redis + giant AWS servers everywhere.
```

Instead:

```text
Understand the workload.
Measure the system.
Find the limiting resource.
Choose an architecture that addresses that specific limit.
```

For many normal systems, the simpler solution is the better solution.

Extreme optimization only becomes justified when the workload requires it.

---

# 32. Compact Mental Model

When a service is too slow, ask these questions in order:

### 1. Is the benchmark itself valid?

- Can the client generate enough traffic?
- Are tests long enough?
- Are errors being counted?

### 2. Is CPU saturated?

- one core or all cores?
- application process or another process?
- can work be parallelized?

### 3. Is memory the problem?

- allocation pressure?
- working set too large?
- cache/Redis approaching capacity?

### 4. Is the network saturated?

Calculate:

```text
payload size × requests/sec
```

### 5. Is storage or the database limiting you?

- expensive scans?
- missing indexes?
- too many writes?
- database CPU saturated?
- storage IOPS saturated?

### 6. Is software overhead significant?

- framework;
- runtime;
- JSON parser;
- logging;
- compression;
- process coordination.

### 7. Should the work remain synchronous?

Can some operations move to:

- queues;
- background workers;
- batching;
- asynchronous persistence?

### 8. Should you scale vertically or horizontally?

Only after identifying the real bottleneck should you decide which resource to add.

---

# 33. Final Takeaways

> ### The central lesson
> **Performance engineering is the discipline of discovering what the system is actually waiting on.**

The experiment repeatedly reinforces the same pattern:

1. **Establish a baseline.**
2. **Generate enough load to reveal a limit.**
3. **Monitor the system while the limit occurs.**
4. **Identify the saturated resource.**
5. **Change one architectural assumption.**
6. **Retest.**
7. **Expect the bottleneck to move somewhere else.**

At one million requests per second, software engineering becomes inseparable from:

- operating systems;
- networking;
- hardware;
- algorithms;
- databases;
- distributed systems;
- economics.

That is the real point of the experiment.

---

## One-Line Summary

**You do not reach extreme scale by endlessly optimizing application code; you reach it by repeatedly identifying which resource is currently limiting the system and redesigning the architecture around that constraint.**
