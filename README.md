# cpu_info

A lightweight, cross-platform Rust library that retrieves detailed CPU information with **zero unsafe FFI dependencies** and without relying on system locale or external commands.

This crate is designed to work reliably on **Linux** and **Windows** (more platforms coming soon).
It parses standardized kernel interfaces (`/sys`, `/proc`) and CPUID instructions when available, delivering:

* CPU vendor (Intel, AMD, ARM implementers, or custom vendor string)
* CPU architecture (x86, x86-64, ARM, ARM64)
* CPU model name (via CPUID on x86 or procfs on ARM)
* Total logical cores
* Total physical cores
* Core grouping and distribution analysis

  * Linear CPUs (all cores identical)
  * Hybrid CPUs (e.g., Intel P-cores + E-cores)
  * Custom configurations (multiple clock groups)

This crate is pure Rust and does **not** use any C libraries or bindings.

---

## ✨ Features

### ✔ Accurate vendor detection

* On x86/x86-64: via CPUID vendor string
* On ARM: via `implementer` ID from `/proc/cpuinfo`

### ✔ Architecture detection

Based on Rust’s built-in compile-time constants (always correct).

### ✔ Physical and logical core counting

Uses `/sys/devices/system/cpu` for reliable detection across all Linux distributions.

### ✔ Hybrid CPU detection

Detects P-cores and E-cores by analyzing clock frequency groups or `core_type` when available.

### ✔ No locale issues

Does **not** rely on text labels like “model name”, which vary by language.

### ✔ No external commands

No calls to `lscpu`, `nproc`, `dmidecode`, etc.

---

## 🔭 Upcoming features (work in progress)

The crate will soon include **runtime CPU feature detection**, such as:

* SSE, SSE2, SSE3, SSSE3
* AVX, AVX2
* BMI1, BMI2
* FMA
* POPCNT
* RDTSCP
* AES-NI
* And many other CPUID-based feature bits

This will allow applications and game engines to **adapt dynamically** to available instruction sets without relying on compile-time feature flags.

(Windows + Linux support planned; macOS may be added later.)

---

## 🚀 Example

```rust
use cpu_info::CpuInfo;

fn main() {
    let info = CpuInfo::new();
    println!("{:#?}", info);
}
```

---