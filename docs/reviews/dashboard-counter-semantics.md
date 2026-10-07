# Dashboard counter semantics review

Date: 2026-10-07. This section records the memory-counter investigation and author validation. Numerical integration and estimator state are outside this change.

## macOS memory accounting

The apparently contradictory readings came from native counters with overlapping meanings. Pinned `sysinfo 0.39.6`, `src/unix/apple/system.rs:189–208`, computes system used bytes from anonymous pages minus purgeable pages, plus wired and compressed pages. Its available-memory counter combines active, inactive and free pages. Active anonymous memory therefore contributes to both counters. The native free counter separately subtracts speculative pages from the free-page count. No counter was accidentally summed over worker threads.

Apple describes the broad non-compressed availability measure as including reclaimable active/inactive pages; reclaimability is not the same as memory being unused. Its memory-pressure model also differs from a simple used/total ratio. This explains why used and available can sum to more than physical RAM. [Apple XNU memory accounting](https://github.com/apple-oss-distributions/xnu/blob/main/doc/vm/memorystatus_notify.md#available-memory). Apple's Activity Monitor documentation likewise distinguishes app, wired, compressed and cached memory; a claim that the discrepancy consists only of flushable cache would be inaccurate. [Activity Monitor memory categories](https://support.apple.com/guide/activity-monitor/view-memory-usage-actmntr1004/mac).

The CLI now samples the existing public `System::free_memory()` into an additive `system_free_bytes` field. `Snapshot::free_or_available()` chooses the `Free` label and native free bytes on macOS; it retains the `Available` label and native available bytes on other platforms. Generation and integration presentation share this pair. Existing `system_used_bytes`, `system_total_bytes` and `system_available_bytes` JSON meanings are unchanged. In particular, the old available field is neither silently redefined nor used as a free-memory value. No fabricated cache category or `total - used` estimate was added. Free and used are also not asserted to partition all physical memory.

## Native process RSS

The public `Process::memory()` remains the process resident set. The macOS implementation reads `proc_taskinfo.pti_resident_size`, rather than multiplying a value by the number of threads or using virtual size. RSS excludes swapped-out memory and is not the macOS physical-footprint/Activity Monitor process metric. The observed peak remains the maximum sampled RSS of this one process, not an OS high-water mark. These distinctions are preserved in existing labels and comments.

## Focused host evidence

A standalone Rust probe linked the pinned built `sysinfo` library and sampled only the current PID through its public API. One observed snapshot reported total **38,654,705,664 B**, used **33,783,103,488 B**, available **15,989,227,520 B**, and free **115,507,200 B**. The used-plus-available value exceeded total by more than 11 billion bytes, directly reproducing the overlap. These are time-varying host readings, not reference constants.

Two worker threads each touched 4 MiB; one-PID RSS increased by **8,650,752 B**. The resulting native RSS of **11,599,872 B** exactly matched the independent `ps` reading of **11,328 KiB**. A contemporaneous `vm_stat` capture retains native page categories for diagnosis. The probe allocated only 8 MiB and did not change system settings or disturb other processes.

A second standalone probe imported the actual CLI memory module. It passed native free/available sampling, the 500 ms cached-read behavior, preservation of both distinct JSON fields, shared macOS label/value selection, a fixture where 15 GiB available must not be shown as free, zero free bytes, and unavailable-counter rendering. Only public dependency APIs were reused; no new platform FFI or sampling dependency was introduced. Evidence resides under ignored `output/dashboard-counter-semantics/`: `memory_native_probe.rs`, `memory-native.txt`, `vm-stat.txt`, `memory_monitor_probe.rs`, and `memory-monitor.txt`. No permanent tests, other examples, or release build were changed by this slice. Execution evidence is macOS-only; other-platform behavior follows the existing native available-memory API and has not been executed here.
