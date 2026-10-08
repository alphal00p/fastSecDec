# macOS serial-worker socket control (2026-10-08)

The native double-box smoke run on `91d2ac9` failed immediately in serial
generation with `Resource temporarily unavailable (os error 35)` after starting
the preparation child. Ordinary generation completed. The failure was in the
CLI control transport, before sector algebra or evaluator construction.

## Native API and focused probe

`ProcessPool` makes its loopback `TcpListener` nonblocking so the caller can poll
the child exit and handshake deadline. On Darwin, the accepted socket inherits
that nonblocking state. Rust's native `TcpListener::accept` implementation on
this platform calls `accept`, sets close-on-exec, and does not reset blocking
mode. The dedicated control reader uses `Read::read_exact` for framed messages;
an idle gap therefore returned `WouldBlock` and terminated that reader. This
also affected gaps within a frame, and potentially parent writes under socket
backpressure.

A standalone Rust standard-library probe on aarch64 macOS reproduced an idle
accepted read returning `WouldBlock` / OS error 35 in 1.8 microseconds. Calling
the public `TcpStream::set_nonblocking(false)` on that same connection made its
next read wait for a delayed writer and succeed after 105 milliseconds. Probe
source and raw evidence are ignored under
`output/latest-main-double-box-review/socket-probe/`.

## Change and ownership

Immediately after accepting a child connection, explicitly restore blocking
mode before creating either stream clone. The listener remains nonblocking;
the coordinator continues polling a bounded channel, and the dedicated reader
thread owns blocking frame reads. Shutdown still closes the socket and unblocks
the reader before joining it. No busy retry loop, protocol replacement,
additional dependency, native algebra helper or numerical change is needed.

The regression uses the existing Rust test binary as a child. It leaves idle
gaps before two frames and splits magic, length and payload across separate
writes. It requires complete ordered messages, a clean child exit and release
of its residency slot. Existing tests continue covering stdout isolation,
inherited OS-lock ownership, frame rejection and worker replacement.

The focused `process::tests` suite passed: five tests, including both socket
regressions, with two subprocess fixtures ignored by the parent harness. Raw
results are in the ignored probe directory's `tests.log`. The existing
`AtomicUsize::fetch_update` deprecation warning on Rust 1.99 is unrelated; the
API remains unchanged to preserve the project's Rust 1.96 baseline.

The release rebuild and full double-box ordinary/serial verification passed.
The [execution report](gghh-serial-macos-validation.md) records both generation
modes, all four integration combinations, achieved QMC accuracy, actual resident
sampling times and completed-checkpoint continuation with a changed worker count.
An independent reviewer accepted the socket fix and fragmented-frame regression.
