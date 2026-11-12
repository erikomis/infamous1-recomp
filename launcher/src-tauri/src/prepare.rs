//! The launcher without its window, for programs that start it
//! (RecompLauncher, see `recomplauncher.json` at the repository's top and
//! the launcher README, "RecompLauncher"):
//!
//! - `--prepare --game <copy>`: what Install does in the window, for a player
//!   who has nothing yet or to bring an install up to date. The copy is
//!   checked (RPCS3's dev_hdd0/game/NPUA80480 folder, or the RPCS3 data
//!   folder above it) and its licence (.rap) found the way the Install screen
//!   finds it, both remembered in launcher.json; then Apple's Command Line
//!   Tools, and rebuild.py with the window's own arguments into the install
//!   folder. It prints `progress <0..1> <text>` lines, rebuild.py's output as
//!   plain lines (also kept in logs/install.log), and exits with one of the
//!   codes in `Exit`, 0 only when the build is there afterwards.
//! - `--play` with `RECOMPLAUNCHER=1`: the window's Play (the settings, the
//!   save backup, logs/game.log), no window; the launcher's process becomes
//!   run.sh's, which execs the game, so its exit code and its lifetime are
//!   the game's. `--game <copy>` checks and remembers that copy first.
//!
//! Both decide in `run()` before any window or Tauri event loop exists. The
//! rebuild and the game are started by the same code as the window's
//! (rebuild_command, run_sh_command in lib.rs).

use super::*;
use std::collections::VecDeque;
use std::io::Write;
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// The headless prepare's and `--play`'s exit codes (as recomplauncher.json's
/// exit_codes says them to the player).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    /// Prepare: the game is built. Play: the game was quit normally.
    Ok,
    /// No copy given, not NPUA80480, incomplete, or no licence (.rap) found.
    Copy,
    /// Something the player has to do first: Apple's Command Line Tools
    /// (their installer is opened), the build tools, RPCS3 or the firmware,
    /// or not a Mac.
    NeedsPlayer,
    /// A download the build makes (the toolkit, its Python packages) failed:
    /// the connection.
    Sources,
    /// A step of the rebuild failed (install.log has why), or no room.
    Build,
    /// Another prepare, a rebuild started by the window, or the game itself
    /// is running.
    Running,
    /// Play: the game is not built yet.
    NotBuilt,
    /// The launcher failed on its own (its settings, starting a program).
    Failed,
}

impl Exit {
    #[cfg(test)]
    pub const ALL: [Exit; 8] = [Exit::Ok, Exit::Copy, Exit::NeedsPlayer, Exit::Sources, Exit::Build, Exit::Running, Exit::NotBuilt, Exit::Failed];

    pub fn code(self) -> i32 {
        match self {
            Exit::Ok => 0,
            Exit::Copy => 40,
            Exit::NeedsPlayer => 41,
            Exit::Sources => 42,
            Exit::Build => 43,
            Exit::Running => 44,
            Exit::NotBuilt => 45,
            Exit::Failed => 46,
        }
    }

    /// What the player is told (recomplauncher.json's exit_codes; "ok" for
    /// a normal end).
    pub fn says(self) -> &'static str {
        match self {
            Exit::Ok => "ok",
            Exit::Copy => "Your copy isn't one this port reads: it needs your own inFAMOUS from the PlayStation Store (NPUA80480) as RPCS3 installed it, the dev_hdd0/game/NPUA80480 folder, with its licence (.rap) in RPCS3's exdata folder. The disc release (BCUS98119) doesn't work yet. Pick it again on the game's page.",
            Exit::NeedsPlayer => "Your Mac needs something first: Apple's Command Line Tools (their installer opened: finish it), the build tools (CMake and Python 3.10 or newer: the inFAMOUS launcher installs them with Homebrew), or RPCS3 with the PS3 firmware, which decrypts your copy. The setup's last lines say which. Then set the game up again.",
            Exit::Sources => "Couldn't download what the build needs (the ps3recomp toolkit or its Python packages). Check the internet connection and set the game up again.",
            Exit::Build => "Building the game failed. Check there are about 5 GB free, then set it up again: it continues where it stopped. install.log in the logs folder says why.",
            Exit::Running => "inFAMOUS is already running, or another setup of it is. Close it or wait for it, then try again.",
            Exit::NotBuilt => "The game isn't built yet. Set it up again on the game's page (the first time takes about 10 minutes).",
            Exit::Failed => "The inFAMOUS launcher couldn't start it. Open the launcher itself to see why.",
        }
    }
}

// ---------------------------------------------------------------------------
// The command line
// ---------------------------------------------------------------------------

pub const PREPARE_ARG: &str = "--prepare";
pub const GAME_ARG: &str = "--game";

#[derive(Debug, PartialEq)]
pub enum Mode {
    /// `--prepare [--game <copy>]`: the headless install.
    Prepare { game: Option<PathBuf> },
    /// `--play [--game <copy>]` started by RecompLauncher: the game, no window.
    PlayDirect { game: Option<PathBuf> },
    /// The window (with `--play`: Play once it is up).
    Window { game: Option<PathBuf> },
}

/// `--game <path>` or `--game=<path>`.
pub fn game_arg(args: &[String]) -> Option<PathBuf> {
    let mut it = args.iter().skip(1);
    while let Some(a) = it.next() {
        if let Some(v) = a.strip_prefix("--game=") {
            return Some(PathBuf::from(v)).filter(|p| !p.as_os_str().is_empty());
        }
        if a == GAME_ARG {
            return it.next().filter(|v| !v.is_empty()).map(PathBuf::from);
        }
    }
    None
}

/// What the command line (argv, program first) asks for; `recomplauncher`:
/// RECOMPLAUNCHER=1 is set (under_recomplauncher).
pub fn mode(args: &[String], recomplauncher: bool) -> Mode {
    let game = game_arg(args);
    if args.iter().skip(1).any(|a| a == PREPARE_ARG) {
        Mode::Prepare { game }
    } else if recomplauncher && play_shortcut::requested(args.iter().cloned()) {
        Mode::PlayDirect { game }
    } else {
        Mode::Window { game }
    }
}

/// RecompLauncher sets RECOMPLAUNCHER=1 for every game it starts.
pub fn under_recomplauncher() -> bool {
    std::env::var("RECOMPLAUNCHER").is_ok_and(|v| v.trim() == "1")
}

// ---------------------------------------------------------------------------
// The player's copy
// ---------------------------------------------------------------------------

/// The disc release of the same game: named when the player picks it.
const DISC_ID: &str = "BCUS98119";

#[derive(Debug, PartialEq)]
pub struct GameCopy {
    /// The NPUA80480 folder (PARAM.SFO, USRDIR/EBOOT.BIN).
    pub dir: PathBuf,
    /// PARAM.SFO's APP_VER (may be empty).
    pub app_ver: String,
}

/// A title ID as written in PARAM.SFO, compared without a dash: "NPUA-80480"
/// and "NPUA80480" are the same.
fn id_key(v: &str) -> String {
    v.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_ascii_uppercase()
}

fn wrong_copy(id: &str) -> String {
    if id.is_empty() {
        "That copy's title ID can't be read: this port needs inFAMOUS from the PlayStation Store, NPUA80480.".into()
    } else if id_key(id) == DISC_ID {
        format!("That is the disc release of inFAMOUS ({id}): this port is built from the PlayStation Store one, NPUA80480 (the disc doesn't work yet).")
    } else {
        format!("That copy is {id}, not inFAMOUS from the PlayStation Store (NPUA80480).")
    }
}

/// Whether `p` is the player's NPUA80480: the game folder RPCS3 installed
/// (or the RPCS3 data folder above it, as rebuild.py takes it), whose
/// PARAM.SFO says NPUA80480 and that has USRDIR/EBOOT.BIN. Reads only. The
/// error is said to the player.
pub fn check_copy(p: &Path) -> Result<GameCopy, String> {
    if !p.exists() {
        return Err(format!("{} isn't there: pick your inFAMOUS game folder (RPCS3's dev_hdd0/game/NPUA80480) again.", p.display()));
    }
    if p.is_file() {
        return Err(format!("{} is a file: this port reads the game folder RPCS3 installed from the PlayStation Store package (dev_hdd0/game/NPUA80480).", p.display()));
    }
    let Some(dir) = resolve_game_dir(p) else {
        // a disc's folder: say which game it is
        if let Some(sfo) = read_sfo(&p.join("PS3_GAME/PARAM.SFO")) {
            return Err(wrong_copy(sfo.get("TITLE_ID").map(String::as_str).unwrap_or("")));
        }
        return Err(format!("{} has no PARAM.SFO: pick the NPUA80480 folder in RPCS3's dev_hdd0/game.", p.display()));
    };
    let sfo = read_sfo(&dir.join("PARAM.SFO")).ok_or("Its PARAM.SFO can't be read: the copy may be incomplete.")?;
    let id = sfo.get("TITLE_ID").cloned().unwrap_or_default();
    if id_key(&id) != TITLE_ID {
        return Err(wrong_copy(&id));
    }
    if !dir.join("USRDIR/EBOOT.BIN").is_file() {
        return Err("That folder has no USRDIR/EBOOT.BIN: the copy is incomplete.".into());
    }
    let app_ver = sfo.get("APP_VER").or(sfo.get("VERSION")).cloned().unwrap_or_default();
    Ok(GameCopy { dir, app_ver })
}

/// The copy's licence: the .rap the Install screen would find for it (the
/// exact content id first), else the one remembered in the settings.
fn find_rap(copy: &GameCopy, s: &Settings) -> Option<PathBuf> {
    let found = search_licence(&[copy.dir.to_string_lossy().into_owned()]);
    found.raps.first().map(PathBuf::from).or_else(|| {
        let p = PathBuf::from(s.rap_path.trim());
        (!s.rap_path.trim().is_empty() && fs::metadata(&p).is_ok_and(|m| m.is_file() && m.len() == 16)).then_some(p)
    })
}

/// launcher.json as it is, or the defaults when there is none; an unreadable
/// one is an error (it is not overwritten).
fn load_settings_strict() -> Result<Settings, String> {
    let p = settings_path();
    match fs::read_to_string(&p) {
        Ok(t) => serde_json::from_str(&t).map_err(|e| format!("{}: {e}", p.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(e) => Err(format!("{}: {e}", p.display())),
    }
}

/// The copy (and its licence) as the Install screen's inputs.
fn remember_copy(dir: &Path, rap: Option<&Path>) -> Result<(), String> {
    let mut s = load_settings_strict()?;
    s.game_paths = vec![dir.to_string_lossy().into_owned()];
    if let Some(r) = rap {
        s.rap_path = r.to_string_lossy().into_owned();
    }
    write_settings_file(&s)
}

/// `--game` with the window: check it and remember it as the copy before the
/// window opens. Returns whether `--play` may go ahead (no `--game`, or a
/// good one).
pub fn window_game(game: Option<&Path>) -> bool {
    let Some(g) = game else { return true };
    match check_copy(g).and_then(|c| remember_copy(&c.dir, None).map(|_| c)) {
        Ok(c) => {
            eprintln!("[launcher] --game: the copy is {}", c.dir.display());
            true
        }
        Err(e) => {
            eprintln!("[launcher] --game {}: {e} (--play is not started)", g.display());
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Progress
// ---------------------------------------------------------------------------

/// One output line, ignoring a closed stdout (the program that started us
/// may have stopped reading).
fn say(line: &str) {
    let mut o = std::io::stdout().lock();
    let _ = writeln!(o, "{line}");
    let _ = o.flush();
}

/// `progress <fraction> <text>` (RecompLauncher's prepare contract): the
/// fraction from 0 to 1 with three decimals, the text on one line.
pub fn progress_line(f: f64, text: &str) -> String {
    let text: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    format!("progress {:.3} {text}", f.clamp(0.0, 1.0))
}

/// A terminal's colour and bold escapes (\x1b[...m) taken out.
pub fn strip_esc(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut it = line.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\x1b' && it.peek() == Some(&'[') {
            it.next();
            for c in it.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

/// A script's output line as it is printed: without escapes, and indented
/// when it would read as a progress line itself.
pub fn log_line(raw: &str) -> String {
    let l = strip_esc(raw);
    if l.starts_with("progress") {
        format!("  {l}")
    } else {
        l
    }
}

/// The progress lines printed: the fraction never goes back, and a line is
/// printed only when the text changes or the fraction moves by half a
/// percent.
#[derive(Default)]
pub struct Reporter {
    last: f64,
    printed: f64,
    text: String,
    any: bool,
}

impl Reporter {
    pub fn update(&mut self, f: f64, text: &str) -> Option<String> {
        let f = f.clamp(0.0, 1.0).max(self.last);
        self.last = f;
        if self.any && text == self.text && f < self.printed + 0.005 && !(f >= 1.0 && self.printed < 1.0) {
            return None;
        }
        self.any = true;
        self.printed = f;
        self.text = text.to_string();
        Some(progress_line(f, text))
    }

    /// The same, always printed (a step's end or a failure's reason).
    pub fn force(&mut self, f: f64, text: &str) -> String {
        self.any = false;
        self.update(f, text).unwrap_or_default()
    }

    fn say(&mut self, f: f64, text: &str) {
        if let Some(l) = self.update(f, text) {
            say(&l);
        }
    }

    fn fail(&mut self, text: &str) {
        let l = self.force(self.last, text);
        say(&l);
    }

    #[cfg(test)]
    pub fn last(&self) -> f64 {
        self.last
    }
}

/// rebuild.py's steps (its STEPS) and their rough share of a first rebuild
/// on an Apple-silicon Mac, as the window's bar has them (src/progress.ts).
const WEIGHT: [(&str, f64); 13] = [
    ("toolkit", 3.0),
    ("venv", 3.0),
    ("game", 2.0),
    ("pup", 1.0),
    ("decrypt", 2.0),
    ("port", 0.2),
    ("firmware", 2.0),
    ("ppu", 30.0),
    ("spu", 1.0),
    ("jobs", 3.0),
    ("dlc", 0.2),
    ("compile", 50.0),
    ("finish", 0.1),
];

fn weight(id: &str) -> f64 {
    WEIGHT.iter().find(|(s, _)| *s == id).map(|(_, w)| *w).unwrap_or(1.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum StepState {
    Pending,
    Running,
    Done,
    Error,
}

#[derive(Clone, Debug)]
struct Step {
    id: String,
    state: StepState,
    start: f64,
    sub: f64,
}

/// The rebuild's progress from rebuild.py's machine-readable lines (the
/// protocol in its docstring), read as src/progress.ts reads them: PLAN lists
/// the steps (up-to-date ones count as done), PROGRESS starts one,
/// SUBPROGRESS moves it along, STEP-DONE / STEP-SKIP end it, ERROR fails it;
/// failed CHECK lines are kept for classify. Times are in seconds from any
/// start.
#[derive(Default)]
pub struct Tracker {
    steps: Vec<Step>,
    /// The names of the CHECK fail lines, and their messages.
    pub failed_checks: Vec<String>,
    pub failed_messages: Vec<String>,
    /// ERROR lines' text, the last one last.
    pub errors: Vec<String>,
}

impl Tracker {
    fn step(&mut self, id: &str) -> Option<&mut Step> {
        self.steps.iter_mut().find(|s| s.id == id)
    }

    pub fn feed(&mut self, raw: &str, t: f64) {
        let line = strip_esc(raw);
        let mut w = line.split(' ');
        let kind = w.next().unwrap_or("");
        let words: Vec<&str> = w.collect();
        let at = |i: usize| words.get(i).copied().unwrap_or("");
        match kind {
            "PLAN" => {
                // PLAN i/n id state title...
                let (id, state) = (at(1), at(2));
                if !id.is_empty() && self.steps.iter().all(|s| s.id != id) {
                    let state = if state == "run" { StepState::Pending } else { StepState::Done };
                    self.steps.push(Step { id: id.to_string(), state, start: t, sub: 0.0 });
                }
            }
            "PROGRESS" => {
                if let Some(s) = self.step(at(0)) {
                    if s.state == StepState::Pending {
                        s.state = StepState::Running;
                        s.start = t;
                    }
                }
            }
            "SUBPROGRESS" => {
                let frac = at(1).split_once('/').and_then(|(a, b)| Some((a.parse::<f64>().ok()?, b.parse::<f64>().ok()?)));
                if let (Some(s), Some((a, b))) = (self.step(at(0)), frac) {
                    if b > 0.0 {
                        s.sub = (a / b).clamp(0.0, 1.0);
                    }
                }
            }
            "STEP-DONE" | "STEP-SKIP" => {
                if let Some(s) = self.step(at(0)) {
                    s.state = StepState::Done;
                    s.sub = 1.0;
                }
            }
            "ERROR" => {
                let id = at(0).to_string();
                if let Some(s) = self.step(&id) {
                    s.state = StepState::Error;
                }
                self.errors.push(words.get(1..).map(|m| m.join(" ")).unwrap_or_default());
            }
            "CHECK" if at(0) == "fail" && !at(1).is_empty() => {
                self.failed_checks.push(at(1).to_string());
                self.failed_messages.push(words.get(2..).map(|m| m.join(" ")).unwrap_or_default());
            }
            _ => {}
        }
    }

    /// The share of the rebuild done at `t`: a running step counts as far as
    /// it says, or as half done after two minutes and at most 90 %.
    pub fn fraction(&self, t: f64) -> f64 {
        let total: f64 = self.steps.iter().map(|s| weight(&s.id)).sum();
        if total <= 0.0 {
            return 0.0;
        }
        let mut done = 0.0;
        for s in &self.steps {
            let w = weight(&s.id);
            match s.state {
                StepState::Done => done += w,
                StepState::Running => {
                    let el = (t - s.start).max(0.0);
                    done += w * s.sub.max((el / (el + 120.0)).min(0.9));
                }
                _ => {}
            }
        }
        done / total
    }

    /// The step running, or the one that failed.
    pub fn stage(&self) -> Option<&str> {
        self.steps.iter().find(|s| matches!(s.state, StepState::Running | StepState::Error)).map(|s| s.id.as_str())
    }
}

/// A step's text under the progress bar.
pub fn stage_text(id: Option<&str>) -> &'static str {
    match id {
        Some("toolkit") => "Getting the ps3recomp toolkit",
        Some("venv") => "Setting up Python for the translators",
        Some("game") => "Copying your game folder",
        Some("pup") => "Taking the firmware out of PS3UPDAT.PUP",
        Some("decrypt") => "Decrypting EBOOT.BIN and the firmware",
        Some("port") => "Copying the port's sources",
        Some("firmware") => "Translating the PS3 firmware",
        Some("ppu") => "Translating the game's code",
        Some("spu") => "Translating the SPU programs",
        Some("jobs") => "Translating the SPU job modules",
        Some("dlc") => "Checking the DLC licence",
        Some("compile") => "Compiling the game (the longest step)",
        Some("finish") => "Finishing the install",
        _ => "Checking what the build needs",
    }
}

/// The share of the bar before rebuild.py starts (the copy, the tools) and
/// where its share ends.
const REBUILD_FROM: f64 = 0.03;
const REBUILD_TO: f64 = 0.99;

pub fn overall(rebuild_fraction: f64) -> f64 {
    REBUILD_FROM + (REBUILD_TO - REBUILD_FROM) * rebuild_fraction.clamp(0.0, 1.0)
}

/// Why rebuild.py failed, from its exit status (2: a prerequisite), the
/// checks that failed and its last lines.
pub fn classify(code: i32, failed_checks: &[String], tail: &[String]) -> Exit {
    let failed = |names: &[&str]| failed_checks.iter().any(|c| names.contains(&c.as_str()));
    if code == 2 || (code != 1 && !failed_checks.is_empty()) {
        return if failed(&["game", "rap"]) {
            Exit::Copy
        } else if failed(&["disk"]) {
            Exit::Build
        } else if failed(&["patch", "out", "toolkit"]) {
            // the bundled sources or the launcher's own arguments
            Exit::Failed
        } else if failed_checks.is_empty() {
            Exit::Build
        } else {
            Exit::NeedsPlayer
        };
    }
    let text = tail.join("\n").to_ascii_lowercase();
    let net = [
        "could not resolve host",
        "unable to access",
        "failed to connect",
        "connection timed out",
        "connection refused",
        "network is unreachable",
        "no route to host",
        "temporary failure in name resolution",
        "could not fetch",
        "connectionerror",
        "max retries exceeded",
    ];
    if net.iter().any(|n| text.contains(n)) {
        Exit::Sources
    } else {
        Exit::Build
    }
}

// ---------------------------------------------------------------------------
// One run at a time
// ---------------------------------------------------------------------------

/// One headless prepare at a time: an exclusive lock on <data>/prepare.lock,
/// held while the returned file is open (the system drops it when the
/// process ends, however it ends). None: another one holds it.
fn take_lock() -> Result<Option<fs::File>, String> {
    let data = data_root();
    fs::create_dir_all(&data).map_err(|e| format!("{}: {e}", data.display()))?;
    let p = data.join("prepare.lock");
    let f = fs::OpenOptions::new().create(true).truncate(false).write(true).open(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    match f.try_lock() {
        Ok(()) => Ok(Some(f)),
        Err(fs::TryLockError::WouldBlock) => Ok(None),
        Err(fs::TryLockError::Error(e)) => Err(format!("{}: {e}", p.display())),
    }
}

/// Every process's command line (ps), for game_running / rebuild_running.
fn command_lines() -> Vec<String> {
    command_stdout(Command::new("/bin/ps").args(["-axww", "-o", "command="]))
        .map(|s| s.lines().map(str::to_string).collect())
        .unwrap_or_default()
}

/// This game's build, started anywhere: the program itself (argv[0], an
/// absolute path, as run.sh execs it) is its executable under
/// infamous1-lle/build/, the bundle's or the plain one. Not the name alone
/// (the other ports' games run on this Mac too), and not a command that only
/// mentions the path (a shell script, an editor, tail).
pub fn is_game_line(l: &str) -> bool {
    let exe = exe_name();
    l.starts_with('/')
        && ["/Infamous1Recomp.app/Contents/MacOS/", "/"].iter().any(|mid| {
            let pat = format!("infamous1-lle/build{mid}{exe}");
            l.match_indices(&pat).any(|(i, _)| {
                let program = &l[..i];
                matches!(l[i + pat.len()..].chars().next(), None | Some(' ')) && !program.contains(" -") && !program.contains(" /")
            })
        })
}

/// A rebuild.py of this port doing work (the window's Install, or one by
/// hand): a Python whose script is .../infamous1-lle/tools/rebuild.py (after
/// options such as -u), not a command that only mentions it; its quick
/// --check-json / --dry-run / --list-steps runs are not.
pub fn is_rebuild_line(l: &str) -> bool {
    let Some(i) = l.find("infamous1-lle/tools/rebuild.py") else { return false };
    if ["--check-json", "--dry-run", "--list-steps"].iter().any(|a| l[i..].split_whitespace().any(|w| w == *a)) {
        return false;
    }
    let mut words = l[..i].split(' ');
    let program = words.next().unwrap_or("");
    if !program.rsplit('/').next().unwrap_or("").to_ascii_lowercase().starts_with("python") {
        return false;
    }
    for w in words.filter(|w| !w.is_empty()) {
        if !w.starts_with('-') {
            break; // the script's path
        }
        if w == "-c" || w == "-m" {
            return false;
        }
    }
    true
}

fn game_running() -> bool {
    command_lines().iter().any(|l| is_game_line(l))
}

fn rebuild_running() -> bool {
    command_lines().iter().any(|l| is_rebuild_line(l))
}

/// The bundle's Resources folder (Contents/MacOS/<exe> -> Contents/Resources),
/// where the release keeps port/ (rebuild_script_in).
fn resource_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    Some(exe.parent()?.parent()?.join("Resources"))
}

/// Where the build goes: the install folder the settings name when a
/// rebuild already wrote there, else <data>/install (rebuild.py's default;
/// never a development checkout, which rebuild.py refuses).
fn install_out(s: &Settings) -> PathBuf {
    let d = PathBuf::from(s.install_dir.trim());
    if !s.install_dir.trim().is_empty() && (d.join("install.json").is_file() || d.join(".rebuild").is_dir()) {
        d
    } else {
        data_root().join("install")
    }
}

fn install_log_path() -> PathBuf {
    data_root().join("logs/install.log")
}

// ---------------------------------------------------------------------------
// --prepare
// ---------------------------------------------------------------------------

/// `--prepare`'s process: run_prepare, then on a failure what its exit code
/// means to the player as the last line. Returns the exit code.
pub fn prepare_main(game: Option<&Path>) -> i32 {
    let e = run_prepare(game);
    if e != Exit::Ok {
        say(&format!("Exit code {}: {}", e.code(), e.says()));
    }
    e.code()
}

/// `--prepare --game <copy>`: see the module's comment. Prints progress and
/// log lines on stdout; never opens a window.
pub fn run_prepare(game: Option<&Path>) -> Exit {
    let mut rep = Reporter::default();
    rep.say(0.0, "Checking your copy of the game");
    if !cfg!(target_os = "macos") || !host_supported() {
        rep.fail("This port builds and runs on a Mac (Apple silicon) only.");
        return Exit::NeedsPlayer;
    }
    let Some(game) = game else {
        rep.fail("No copy of the game given: --game <your inFAMOUS NPUA80480 folder>.");
        return Exit::Copy;
    };
    // read only so far: nothing is written before the copy is known good
    let copy = match check_copy(game) {
        Ok(c) => c,
        Err(e) => {
            rep.fail(&e);
            return Exit::Copy;
        }
    };
    say(&format!("Your copy: {} (NPUA80480{})", copy.dir.display(), if copy.app_ver.is_empty() { String::new() } else { format!(", version {}", copy.app_ver) }));
    let s = match load_settings_strict() {
        Ok(s) => s,
        Err(e) => {
            rep.fail(&format!("The launcher's settings can't be read: {e}"));
            return Exit::Failed;
        }
    };
    let Some(rap) = find_rap(&copy, &s) else {
        rep.fail("Your copy's licence (.rap) wasn't found: RPCS3 keeps it in dev_hdd0/home/<user>/exdata. Install the game's PlayStation Store package in RPCS3 with its licence, then pick the game folder again.");
        return Exit::Copy;
    };
    say(&format!("Its licence: {}", rap.display()));

    let _lock = match take_lock() {
        Ok(Some(f)) => f,
        Ok(None) => {
            rep.fail("Another setup of inFAMOUS is already running.");
            return Exit::Running;
        }
        Err(e) => {
            rep.fail(&format!("Couldn't write the launcher's folder: {e}"));
            return Exit::Failed;
        }
    };
    if game_running() {
        rep.fail("inFAMOUS is running: close it, then set it up again.");
        return Exit::Running;
    }
    if rebuild_running() {
        rep.fail("A rebuild of inFAMOUS is running already (the launcher's Install, or one by hand): wait for it, then set it up again.");
        return Exit::Running;
    }

    // the copy and its licence, as the Install screen's own
    if let Err(e) = remember_copy(&copy.dir, Some(&rap)) {
        rep.fail(&format!("Couldn't remember the copy in the launcher's settings: {e}"));
        return Exit::Failed;
    }

    // Apple's Command Line Tools: their installer needs the player
    rep.say(0.01, "Checking Apple's Command Line Tools");
    if !clt_installed() {
        let opened = install_clt().is_ok();
        rep.fail(if opened {
            "Apple's Command Line Tools are needed: finish their installer (it just opened), then set the game up again."
        } else {
            "Apple's Command Line Tools are needed: run xcode-select --install in Terminal, then set the game up again."
        });
        return Exit::NeedsPlayer;
    }
    let Some(python) = find_python() else {
        rep.fail("Python 3 wasn't found: install the Command Line Tools or Homebrew's python@3.12, then set the game up again.");
        return Exit::NeedsPlayer;
    };
    let Some(script) = rebuild_script_in(resource_dir()) else {
        rep.fail("The port's rebuild.py isn't in this launcher (a build without port/): download the release again.");
        return Exit::Failed;
    };

    // the window's Install, with its output here
    let out = install_out(&s);
    let req = RebuildRequest {
        games: vec![copy.dir.to_string_lossy().into_owned()],
        rap: rap.to_string_lossy().into_owned(),
        out: out.to_string_lossy().into_owned(),
        dry_run: false,
        force: false,
        rap_key: None,
        decrypt: DecryptOptions { keys: s.keys_path.clone(), firmware: s.firmware_path.clone(), decrypt_with: s.decrypt_with.clone() },
    };
    let mut cmd = match rebuild_command(&python, &script, &req) {
        Ok(c) => c,
        Err(e) => {
            rep.fail(&e);
            return Exit::Failed;
        }
    };
    say(&format!("$ {} {} --game {} --rap {} --out {}", python.display(), script.display(), req.games[0], req.rap, req.out));
    let mut log = fs::create_dir_all(install_log_path().parent().unwrap()).and_then(|_| fs::File::create(install_log_path())).ok();
    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            rep.fail(&format!("Couldn't start rebuild.py: {}: {e}", python.display()));
            return Exit::Failed;
        }
    };
    // rebuild.py's output as it comes: stdout, and stderr as LOG lines
    let (tx, rx) = mpsc::channel::<String>();
    let readers = [
        child.stdout.take().map(|o| Box::new(o) as Box<dyn Read + Send>).map(|o| (o, "")),
        child.stderr.take().map(|o| Box::new(o) as Box<dyn Read + Send>).map(|o| (o, "LOG ")),
    ]
    .into_iter()
    .flatten()
    .map(|(o, prefix)| {
        let tx = tx.clone();
        thread::spawn(move || {
            for line in BufReader::new(o).lines().map_while(Result::ok) {
                let _ = tx.send(format!("{prefix}{line}"));
            }
        })
    })
    .collect::<Vec<_>>();
    drop(tx);

    let t0 = Instant::now();
    let mut steps = Tracker::default();
    let mut tail: VecDeque<String> = VecDeque::new();
    rep.say(REBUILD_FROM, stage_text(None));
    loop {
        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(raw) => {
                let line = log_line(&raw);
                say(&line);
                if let Some(f) = log.as_mut() {
                    let _ = writeln!(f, "{line}");
                }
                steps.feed(&line, t0.elapsed().as_secs_f64());
                tail.push_back(line);
                if tail.len() > 400 {
                    tail.pop_front();
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        // the bar moves inside a long step too (the lift, the compile)
        let t = t0.elapsed().as_secs_f64();
        rep.say(overall(steps.fraction(t)), stage_text(steps.stage()));
    }
    for r in readers {
        let _ = r.join();
    }
    let code = child.wait().map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);

    if code != 0 {
        let tail: Vec<String> = tail.into_iter().collect();
        let exit = classify(code, &steps.failed_checks, &tail);
        // a prerequisite: what is missing; a step: its error
        let why = if code == 2 && !steps.failed_messages.is_empty() {
            format!("Missing: {}", steps.failed_messages.join("; "))
        } else {
            steps.errors.last().cloned().filter(|e| !e.is_empty()).unwrap_or_else(|| format!("{} failed (code {code}).", stage_text(steps.stage())))
        };
        rep.fail(&why);
        say(&format!("The setup's log: {}", install_log_path().display()));
        return exit;
    }
    let built = resolve_install(&out).ok().filter(|i| Path::new(&i.executable).is_file());
    if built.is_none() {
        rep.fail(&format!("The rebuild ended, but {} has no game built.", out.display()));
        return Exit::Build;
    }
    // Play now plays this install
    let saved = load_settings_strict().and_then(|mut s| {
        s.install_dir = out.to_string_lossy().into_owned();
        write_settings_file(&s)
    });
    if let Err(e) = saved {
        rep.fail(&format!("The game is built, but the launcher's settings couldn't be written: {e}"));
        return Exit::Failed;
    }
    let l = rep.force(1.0, "Ready to play");
    say(&l);
    Exit::Ok
}

// ---------------------------------------------------------------------------
// --play under RecompLauncher
// ---------------------------------------------------------------------------

/// `--play` under RecompLauncher: the copy given with `--game` remembered,
/// then the window's Play (the settings, the save backup, logs/game.log) and
/// the launcher's process replaced by run.sh's (exec), which execs the game.
/// Returns an exit code only when the game could not start.
pub fn play_direct(game: Option<&Path>) -> i32 {
    let refuse = |e: Exit, why: &str| {
        eprintln!("[launcher] --play: {why}");
        e.code()
    };
    if let Some(g) = game {
        let copy = match check_copy(g) {
            Ok(c) => c,
            Err(e) => return refuse(Exit::Copy, &e),
        };
        if let Err(e) = remember_copy(&copy.dir, None) {
            return refuse(Exit::Failed, &e);
        }
    }
    if let Err(e) = require_supported_host() {
        return refuse(Exit::NeedsPlayer, &e);
    }
    if game_running() {
        return refuse(Exit::Running, "the game is already running");
    }
    let s = load_settings_file();
    let inst = match resolve_install(Path::new(&s.install_dir)) {
        Ok(i) if Path::new(&i.executable).is_file() => i,
        Ok(i) => return refuse(Exit::NotBuilt, &format!("{}: not built", i.executable)),
        Err(e) => return refuse(Exit::NotBuilt, &e),
    };
    if let Err(e) = saves::backup_before_play(&save_locations(&s), s.save_backups_keep, now_secs()) {
        eprintln!("[launcher] --play: the save backup failed: {e}");
    }
    let logs = data_root().join("logs");
    let log_path = logs.join("game.log");
    let files = fs::create_dir_all(&logs).and_then(|_| fs::File::create(&log_path)).and_then(|f| f.try_clone().map(|g| (f, g)));
    let (out, err) = match files {
        Ok(f) => f,
        Err(e) => return refuse(Exit::Failed, &format!("{}: {e}", log_path.display())),
    };
    let (mut cmd, _) = run_sh_command(&s, &inst);
    cmd.stdin(Stdio::null()).stdout(out).stderr(err);
    eprintln!("[launcher] --play: starting the game, log {}", log_path.display());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let e = cmd.exec();
        refuse(Exit::Failed, &format!("{}: {e}", inst.run_sh))
    }
    #[cfg(not(unix))]
    {
        match cmd.status() {
            Ok(st) => st.code().unwrap_or(Exit::Failed.code()),
            Err(e) => refuse(Exit::Failed, &format!("{}: {e}", inst.run_sh)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("inf1-prep-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn args(v: &[&str]) -> Vec<String> {
        std::iter::once("launcher").chain(v.iter().copied()).map(String::from).collect()
    }

    #[test]
    fn command_line() {
        assert_eq!(mode(&args(&["--prepare", "--game", "/x/NPUA80480"]), false), Mode::Prepare { game: Some("/x/NPUA80480".into()) });
        assert_eq!(mode(&args(&["--game=/x/g", "--prepare"]), true), Mode::Prepare { game: Some("/x/g".into()) });
        assert_eq!(mode(&args(&["--prepare"]), false), Mode::Prepare { game: None });
        assert_eq!(mode(&args(&["--prepare", "--game"]), false), Mode::Prepare { game: None });
        assert_eq!(mode(&args(&["--prepare", "--game", ""]), false), Mode::Prepare { game: None });
        // --play opens the window, unless RecompLauncher started it
        assert_eq!(mode(&args(&["--play"]), false), Mode::Window { game: None });
        assert_eq!(mode(&args(&["--play", "--game", "/a b/NPUA80480"]), true), Mode::PlayDirect { game: Some("/a b/NPUA80480".into()) });
        assert_eq!(mode(&args(&["--game", "/x"]), true), Mode::Window { game: Some("/x".into()) });
        assert_eq!(mode(&args(&[]), true), Mode::Window { game: None });
        // the program's own name is never an argument
        assert_eq!(mode(&["--prepare".to_string()], false), Mode::Window { game: None });
    }

    #[test]
    fn exit_codes_are_distinct_and_in_the_profile() {
        let codes: std::collections::BTreeSet<i32> = Exit::ALL.iter().map(|e| e.code()).collect();
        assert_eq!(codes.len(), Exit::ALL.len());
        assert_eq!(Exit::Ok.code(), 0);
        // recomplauncher.json says each one to the player, word for word
        let profile: serde_json::Value = serde_json::from_str(include_str!("../../../recomplauncher.json")).unwrap();
        let said = profile["exit_codes"].as_object().unwrap();
        assert_eq!(said.len(), Exit::ALL.len());
        for e in Exit::ALL {
            assert_eq!(said[&e.code().to_string()].as_str(), Some(e.says()), "{e:?}");
        }
    }

    #[test]
    fn failures_are_told_apart() {
        let v = |n: &[&str]| n.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        // exit 2: a prerequisite, by the checks that failed
        assert_eq!(classify(2, &v(&["rap"]), &[]), Exit::Copy);
        assert_eq!(classify(2, &v(&["firmware", "game"]), &[]), Exit::Copy);
        assert_eq!(classify(2, &v(&["firmware"]), &[]), Exit::NeedsPlayer);
        assert_eq!(classify(2, &v(&["rpcs3", "cmake"]), &[]), Exit::NeedsPlayer);
        assert_eq!(classify(2, &v(&["disk"]), &[]), Exit::Build);
        assert_eq!(classify(2, &v(&["patch"]), &[]), Exit::Failed);
        assert_eq!(classify(2, &[], &[]), Exit::Build);
        // exit 1: a step, by its last lines
        let net = ["LOG fatal: unable to access 'https://github.com/NoahBPeterson/ps3recomp/': Could not resolve host: github.com".to_string(), "ERROR toolkit setup_toolkit.sh failed (128)".into()];
        assert_eq!(classify(1, &[], &net), Exit::Sources);
        assert_eq!(classify(1, &[], &["LOG ninja: build stopped: subcommand failed.".into(), "ERROR compile cmake --build failed (1)".into()]), Exit::Build);
        assert_eq!(classify(-1, &[], &[]), Exit::Build);
    }

    /// RecompLauncher edits launcher.json with the profile's settings: every
    /// key is one of Settings' own, every value it writes reads back as that
    /// value, and the defaults are the launcher's.
    #[test]
    fn profile_settings_are_the_launchers() {
        let profile: serde_json::Value = serde_json::from_str(include_str!("../../../recomplauncher.json")).unwrap();
        let ours = serde_json::to_value(Settings::default()).unwrap();
        // numbers as numbers (an f32 setting comes back as 0.05000000074...)
        let same = |a: &serde_json::Value, b: &serde_json::Value| match (a.as_f64(), b.as_f64()) {
            (Some(x), Some(y)) => (x - y).abs() < 1e-6,
            _ => a == b,
        };
        // the file RecompLauncher edits is launcher.json in the data folder
        let file = profile["config"][0]["file"][0]["path"].as_str().unwrap();
        assert_eq!(file, "{home}/Library/Application Support/infamous1-recomp/launcher.json");
        assert!(settings_path().ends_with("Library/Application Support/infamous1-recomp/launcher.json"));
        for s in profile["settings"].as_array().unwrap() {
            let id = s["id"].as_str().unwrap();
            for (k, v) in s["defaults"].as_object().unwrap() {
                let want: serde_json::Value = serde_json::from_str(v.as_str().unwrap()).unwrap();
                assert!(same(&ours[k], &want), "{id}: default of {k} is {} here, {want} in the profile", ours[k]);
            }
            let mut writes: Vec<(String, serde_json::Value)> = Vec::new();
            for c in s["choices"].as_array().into_iter().flatten() {
                for (k, v) in c["set"].as_object().unwrap() {
                    writes.push((k.clone(), serde_json::from_str(v.as_str().unwrap()).unwrap()));
                }
            }
            if let Some(sl) = s.get("slider") {
                writes.push((sl["key"].as_str().unwrap().into(), sl["max"].clone()));
                writes.push((sl["key"].as_str().unwrap().into(), sl["min"].clone()));
            }
            assert!(!writes.is_empty(), "{id}");
            for (k, v) in writes {
                assert!(ours.get(&k).is_some(), "{id}: {k} is not a launcher setting");
                let mut file = ours.clone();
                file[&k] = v.clone();
                let back = serde_json::to_value(serde_json::from_value::<Settings>(file).unwrap_or_else(|e| panic!("{id}: {k}={v}: {e}"))).unwrap();
                assert!(same(&back[&k], &v), "{id}: {k}={v} reads back as {}", back[&k]);
            }
        }
    }

    #[test]
    fn progress_lines() {
        assert_eq!(progress_line(0.25, "Translating  the\ngame's code"), "progress 0.250 Translating the game's code");
        assert_eq!(progress_line(1.7, "x"), "progress 1.000 x");
        assert_eq!(progress_line(-1.0, "x"), "progress 0.000 x");
        // RecompLauncher reads "progress <n> <text>": a script line must not pass for one
        assert_eq!(log_line("progress 0.5 fake"), "  progress 0.5 fake");
        assert_eq!(log_line("\x1b[1mLOG x\x1b[0m"), "LOG x");

        let mut r = Reporter::default();
        assert_eq!(r.update(0.1, "a").as_deref(), Some("progress 0.100 a"));
        assert_eq!(r.update(0.102, "a"), None, "under half a percent");
        assert_eq!(r.update(0.05, "b").as_deref(), Some("progress 0.102 b"), "never back");
        assert_eq!(r.update(0.2, "b").as_deref(), Some("progress 0.200 b"));
        assert_eq!(r.force(0.0, "failed"), "progress 0.200 failed");
        assert_eq!(r.update(1.0, "failed").as_deref(), Some("progress 1.000 failed"));
    }

    /// rebuild.py's output on a first run (the shape of the 2026-11-11 run
    /// from a clean folder), and a rerun with most steps up to date.
    const FIRST: &[&str] = &[
        "CHECK ok host macOS 26.0 arm64",
        "CHECK ok rpcs3 /Applications/RPCS3.app",
        "PLAN 1/13 toolkit run Toolkit (ps3recomp fork + patches)",
        "PLAN 2/13 venv run Python environment for the translators",
        "PLAN 3/13 game run Import the game copy",
        "PLAN 4/13 pup up-to-date Extract the firmware from PS3UPDAT.PUP",
        "PLAN 5/13 decrypt run Decrypt EBOOT and firmware",
        "PLAN 6/13 port run Port sources",
        "PLAN 7/13 firmware run Translate the firmware (liblv2, libsysmodule, libsre)",
        "PLAN 8/13 ppu run Translate the PPU code",
        "PLAN 9/13 spu run Translate the SPU programs",
        "PLAN 10/13 jobs run Translate the SPU job modules",
        "PLAN 11/13 dlc run DLC licence key (optional)",
        "PLAN 12/13 compile run Compile",
        "PLAN 13/13 finish run Write install.json",
        "PROGRESS toolkit 1/13 Toolkit (ps3recomp fork + patches)",
        "LOG Cloning into 'ps3recomp-fork'...",
        "STEP-DONE toolkit 1/13 31",
        "PROGRESS venv 2/13 Python environment for the translators",
        "STEP-DONE venv 2/13 20",
        "PROGRESS game 3/13 Import the game copy",
        "PROGRESS game 3/13 cloning /x/NPUA80480",
        "STEP-DONE game 3/13 1",
        "STEP-SKIP pup 4/13 up to date",
        "PROGRESS decrypt 5/13 Decrypt EBOOT and firmware",
        "PROGRESS decrypt 5/13 EBOOT.BIN",
        "STEP-DONE decrypt 5/13 4",
        "PROGRESS port 6/13 Port sources",
        "STEP-DONE port 6/13 0",
        "PROGRESS firmware 7/13 Translate the firmware (liblv2, libsysmodule, libsre)",
        "STEP-DONE firmware 7/13 9",
        "PROGRESS ppu 8/13 Translate the PPU code",
        "SUBPROGRESS ppu 3/10",
        "SUBPROGRESS ppu 10/10",
        "STEP-DONE ppu 8/13 40",
        "PROGRESS spu 9/13 Translate the SPU programs",
        "STEP-DONE spu 9/13 3",
        "PROGRESS jobs 10/13 Translate the SPU job modules",
        "STEP-DONE jobs 10/13 5",
        "STEP-SKIP dlc 11/13 no DLC",
        "PROGRESS compile 12/13 Compile",
        "SUBPROGRESS compile 100/251",
        "SUBPROGRESS compile 251/251",
        "STEP-DONE compile 12/13 125",
        "PROGRESS finish 13/13 Write install.json",
        "STEP-DONE finish 13/13 0",
        "DONE /x/install/install.json",
    ];

    #[test]
    fn rebuild_lines_move_the_bar_forward() {
        let mut steps = Tracker::default();
        let mut rep = Reporter::default();
        let mut last = -1.0;
        let mut stages = Vec::new();
        for (i, l) in FIRST.iter().enumerate() {
            let t = i as f64 * 30.0;
            steps.feed(l, t);
            for dt in [0.0, 10.0, 29.0] {
                let f = overall(steps.fraction(t + dt));
                assert!((0.0..=1.0).contains(&f));
                rep.update(f, stage_text(steps.stage()));
                assert!(rep.last() >= last, "{l}: {} < {last}", rep.last());
                last = rep.last();
            }
            if let Some(s) = steps.stage() {
                if stages.last() != Some(&s.to_string()) {
                    stages.push(s.to_string());
                }
            }
        }
        assert_eq!(stages, ["toolkit", "venv", "game", "decrypt", "port", "firmware", "ppu", "spu", "jobs", "compile", "finish"]);
        assert!((steps.fraction(1e9) - 1.0).abs() < 1e-9);
        assert!((overall(1.0) - REBUILD_TO).abs() < 1e-9 && overall(0.0) == REBUILD_FROM);
        assert!(steps.failed_checks.is_empty() && steps.errors.is_empty());

        // the compile's own count moves the bar: halfway through it is past
        // the share of every step before
        let mut s2 = Tracker::default();
        for l in &FIRST[..42] {
            s2.feed(l, 0.0);
        }
        let before: f64 = WEIGHT.iter().take_while(|(id, _)| *id != "compile").map(|(_, w)| w).sum();
        let total: f64 = WEIGHT.iter().map(|(_, w)| w).sum();
        let f = s2.fraction(0.0);
        assert!(f > before / total && f < (before + 50.0) / total, "{f}");

        // a rerun: up-to-date steps count as done from the start
        let mut s3 = Tracker::default();
        for l in ["PLAN 1/3 toolkit up-to-date T", "PLAN 2/3 ppu up-to-date P", "PLAN 3/3 compile run C"] {
            s3.feed(l, 0.0);
        }
        assert!((s3.fraction(0.0) - 33.0 / 83.0).abs() < 1e-9);
        // a failure keeps the step it stopped at, its message and the checks
        s3.feed("PROGRESS compile 3/3 Compile", 0.0);
        s3.feed("ERROR compile cmake --build failed (1)", 1.0);
        assert_eq!(s3.stage(), Some("compile"));
        assert_eq!(s3.errors, ["cmake --build failed (1)"]);
        let mut s4 = Tracker::default();
        for l in ["CHECK fail rap no licence given (--rap file.rap)", "CHECK warn disk 9 GB free", "ERROR checks 1 prerequisite(s) missing"] {
            s4.feed(l, 0.0);
        }
        assert_eq!(s4.failed_checks, ["rap"]);
        assert_eq!(s4.failed_messages, ["no licence given (--rap file.rap)"]);
        // lines that only look like steps change nothing
        let mut s5 = Tracker::default();
        for l in ["LOG PLAN 1/1 ppu run x", "progress 0.5 ppu", "  PROGRESS ppu 1/1 x", "PLAN"] {
            s5.feed(l, 0.0);
        }
        assert_eq!(s5.stage(), None);
        assert_eq!(s5.fraction(0.0), 0.0);
    }

    #[test]
    fn this_games_processes_only() {
        let app = "/Users/p/Library/Application Support/infamous1-recomp/install/infamous1-lle/build/Infamous1Recomp.app/Contents/MacOS/Infamous1Recomp /x/vfs/PS3_GAME/USRDIR/EBOOT.elf";
        assert!(is_game_line(app));
        assert!(is_game_line("/r/infamous1-lle/build/Infamous1Recomp"));
        assert!(is_game_line("/r/infamous1-lle/build/Infamous1Recomp /v/EBOOT.elf"));
        // the other ports, the launcher itself, a log viewer
        assert!(!is_game_line("/r/infamous2-lle/build/Infamous2Recomp.app/Contents/MacOS/Infamous2Recomp /v/EBOOT.elf"));
        assert!(!is_game_line("/Applications/infamous1-recomp.app/Contents/MacOS/infamous1-launcher --play"));
        assert!(!is_game_line("/r/infamous1-lle/build/Infamous1Recomp.log"));
        assert!(!is_game_line("tail -f /r/infamous1-lle/build/Infamous1Recomp_old"));
        assert!(!is_game_line("tail -f /r/infamous1-lle/build/Infamous1Recomp"));
        assert!(!is_game_line("/bin/zsh -c cp /bin/sleep /x/infamous1-lle/build/Infamous1Recomp && echo"));
        assert!(!is_game_line("/usr/bin/less /r/infamous1-lle/build/Infamous1Recomp"));

        assert!(is_rebuild_line("/opt/homebrew/bin/python3.12 -u /a/port/infamous1-lle/tools/rebuild.py --game /x --rap /y.rap --out /o --nice 10"));
        assert!(!is_rebuild_line("python3 /a/port/infamous1-lle/tools/rebuild.py --check-json"));
        assert!(!is_rebuild_line("python3 /a/port/infamous1-lle/tools/rebuild.py --dry-run --game /x"));
        assert!(!is_rebuild_line("python3 /a/infamous2-lle/tools/rebuild.py --game /x"));
        // the Command Line Tools' and Homebrew's Python run as Python.app; a path with spaces
        assert!(is_rebuild_line("/Library/Developer/CommandLineTools/Library/Frameworks/Python3.framework/Versions/3.9/Resources/Python.app/Contents/MacOS/Python -u -I /Users/p/Library/Application Support/infamous1-recomp/x/infamous1-lle/tools/rebuild.py --game /g"));
        // commands that only mention it
        assert!(!is_rebuild_line("/bin/zsh -c ls /a/port/infamous1-lle/tools/rebuild.py && echo"));
        assert!(!is_rebuild_line("vim /a/infamous1-lle/tools/rebuild.py"));
        assert!(!is_rebuild_line("python3 -c import x; open('/a/infamous1-lle/tools/rebuild.py')"));
    }

    fn sfo(id: &str) -> Vec<u8> {
        saves::psf_write(&[
            saves::PsfEntry { key: "APP_VER".into(), fmt: saves::PSF_UTF8, data: b"01.00\0".to_vec(), max_len: 8 },
            saves::PsfEntry { key: "TITLE_ID".into(), fmt: saves::PSF_UTF8, data: format!("{id}\0").into_bytes(), max_len: 16 },
        ])
    }

    #[test]
    fn game_folders() {
        let d = tmp("folder");
        // RPCS3's layout: <data>/dev_hdd0/game/NPUA80480
        let rpcs3 = d.join("rpcs3");
        let game = rpcs3.join("dev_hdd0/game/NPUA80480");
        assert!(check_copy(&game).unwrap_err().contains("isn't there"));
        fs::create_dir_all(game.join("USRDIR")).unwrap();
        assert!(check_copy(&game).unwrap_err().contains("no PARAM.SFO"));
        fs::write(game.join("PARAM.SFO"), sfo("NPUA80480")).unwrap();
        assert!(check_copy(&game).unwrap_err().contains("EBOOT.BIN"));
        fs::write(game.join("USRDIR/EBOOT.BIN"), b"x").unwrap();
        let good = GameCopy { dir: game.clone(), app_ver: "01.00".into() };
        assert_eq!(check_copy(&game).unwrap(), good);
        // the RPCS3 data folder above it, and its dev_hdd0/game, find it too
        assert_eq!(check_copy(&rpcs3).unwrap(), good);
        assert_eq!(check_copy(&rpcs3.join("dev_hdd0/game")).unwrap(), good);
        // a file is not a copy
        assert!(check_copy(&game.join("USRDIR/EBOOT.BIN")).unwrap_err().contains("is a file"));
        // another game
        fs::write(game.join("PARAM.SFO"), sfo("NPUA80662")).unwrap();
        assert!(check_copy(&game).unwrap_err().contains("NPUA80662"));
        // the disc release, as a disc folder (PS3_GAME) or PS3_GAME itself
        let disc = d.join("BCUS98119");
        fs::create_dir_all(disc.join("PS3_GAME/USRDIR")).unwrap();
        fs::write(disc.join("PS3_GAME/PARAM.SFO"), sfo("BCUS98119")).unwrap();
        assert!(check_copy(&disc).unwrap_err().contains("disc release"));
        assert!(check_copy(&disc.join("PS3_GAME")).unwrap_err().contains("disc release"));
        fs::remove_dir_all(&d).unwrap();
    }
}
