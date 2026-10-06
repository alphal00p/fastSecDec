//! Scoped CLI terminal and interrupt ownership. Signal callbacks only touch
//! atomics; a native signal-hook iterator wakes ordinary Rust cleanup code.
#[cfg(not(windows))]
use std::thread::{self, JoinHandle};
use std::{
    io,
    sync::{
        Arc, Mutex, Once, Weak,
        atomic::{AtomicBool, AtomicI32, Ordering},
    },
};

use crossterm::{cursor, execute, terminal};
use signal_hook::consts::{SIGINT, SIGTERM};
#[cfg(not(windows))]
use signal_hook::iterator::{Handle, Signals};

#[derive(Default)]
struct Mode {
    raw_owned: bool,
    alternate_screen: bool,
}

#[derive(Default)]
struct TerminalState {
    mode: Mutex<Mode>,
    active: AtomicBool,
}

impl TerminalState {
    fn enter(&self) -> io::Result<()> {
        let _output = io::stderr().lock();
        let mut mode = self.mode.lock().unwrap_or_else(|error| error.into_inner());
        if !terminal::is_raw_mode_enabled()? {
            terminal::enable_raw_mode()?;
            mode.raw_owned = true;
        }
        // Mark before the compound write: EnterAlternateScreen may succeed
        // even if hiding the cursor subsequently fails.
        mode.alternate_screen = true;
        execute!(io::stderr(), terminal::EnterAlternateScreen, cursor::Hide)?;
        self.active.store(true, Ordering::Release);
        Ok(())
    }

    fn restore(&self) {
        self.active.store(false, Ordering::Release);
        // Serialize restoration after any active frame. Stderr's native lock
        // is reentrant, so a panic hook on the drawing thread can restore too.
        // Always acquire it before `mode` to preserve the same lock order.
        let _output = io::stderr().lock();
        let mut mode = self.mode.lock().unwrap_or_else(|error| error.into_inner());
        if mode.raw_owned && terminal::disable_raw_mode().is_ok() {
            mode.raw_owned = false;
        }
        if mode.alternate_screen
            && execute!(io::stderr(), terminal::LeaveAlternateScreen, cursor::Show).is_ok()
        {
            mode.alternate_screen = false;
        }
    }
}

static PANIC_HOOK: Once = Once::new();
static ACTIVE_TERMINAL: Mutex<Weak<TerminalState>> = Mutex::new(Weak::new());

fn register_panic_cleanup(state: &Arc<TerminalState>) {
    PANIC_HOOK.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |information| {
            let active = ACTIVE_TERMINAL
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .upgrade();
            if let Some(active) = active {
                active.restore();
            }
            previous(information);
        }));
    });
    *ACTIVE_TERMINAL
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Arc::downgrade(state);
}

struct Registrations(Vec<signal_hook::SigId>);
impl Drop for Registrations {
    fn drop(&mut self) {
        for id in &self.0 {
            signal_hook::low_level::unregister(*id);
        }
    }
}

pub(super) struct Control {
    pub flag: Arc<AtomicBool>,
    state: Arc<TerminalState>,
    force: Arc<AtomicI32>,
    _registrations: Registrations,
    #[cfg(not(windows))]
    close_signals: Handle,
    #[cfg(not(windows))]
    signal_thread: Option<JoinHandle<()>>,
}

impl Control {
    pub fn new() -> io::Result<Self> {
        let state = Arc::new(TerminalState::default());
        let flag = Arc::new(AtomicBool::new(false));
        let force = Arc::new(AtomicI32::new(0));
        let mut registrations = Registrations(Vec::new());
        for signal in [SIGINT, SIGTERM] {
            let flag = Arc::clone(&flag);
            let force = Arc::clone(&force);
            // SAFETY: the handler only operates on lock-free atomics allocated
            // before registration. It performs no I/O, allocation or locking.
            let id = unsafe {
                signal_hook::low_level::register(signal, move || {
                    if flag.swap(true, Ordering::SeqCst) {
                        force.store(signal, Ordering::SeqCst);
                    }
                })?
            };
            registrations.0.push(id);
        }
        // Register the wakeup after the atomic actions so a wake sees their
        // state, including two signals coalesced before the listener runs.
        #[cfg(not(windows))]
        let (close_signals, signal_thread) = {
            let mut signals = Signals::new([SIGINT, SIGTERM])?;
            let close_signals = signals.handle();
            let terminal = Arc::clone(&state);
            let force = Arc::clone(&force);
            let thread = thread::Builder::new()
                .name("fastsecdec-signals".into())
                .spawn(move || {
                    for _ in signals.forever() {
                        let signal = force.load(Ordering::SeqCst);
                        if signal != 0 {
                            terminal.restore();
                            std::process::exit(128 + signal);
                        }
                    }
                })?;
            (close_signals, thread)
        };
        Ok(Self {
            flag,
            state,
            force,
            _registrations: registrations,
            #[cfg(not(windows))]
            close_signals,
            #[cfg(not(windows))]
            signal_thread: Some(signal_thread),
        })
    }

    pub fn enter_terminal(&self) -> io::Result<()> {
        self.state.enter()?;
        register_panic_cleanup(&self.state);
        Ok(())
    }

    pub fn terminal_active(&self) -> bool {
        self.state.active.load(Ordering::Acquire)
    }

    pub fn restore(&self) {
        self.state.restore();
    }

    pub fn keyboard_interrupt(&self) {
        if self.flag.swap(true, Ordering::SeqCst) {
            self.state.restore();
            std::process::exit(128 + SIGINT);
        }
    }

    pub fn poll_interrupt(&self) {
        // Signal-hook's pipe iterator is Unix-only. Polling also provides the
        // supported cleanup boundary on Windows without signal-context I/O.
        let signal = self.force.load(Ordering::SeqCst);
        if signal != 0 {
            self.state.restore();
            std::process::exit(128 + signal);
        }
    }
}

impl Drop for Control {
    fn drop(&mut self) {
        self.state.restore();
        #[cfg(not(windows))]
        {
            self.close_signals.close();
            if let Some(thread) = self.signal_thread.take() {
                let _ = thread.join();
            }
        }
    }
}
