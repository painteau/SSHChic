//! # SSHChic
//!
//! A fast, multi-threaded ED25519 SSH key generator that searches for public keys matching custom patterns.
//!
//! This tool generates ED25519 key pairs in parallel and tests them against a regex pattern,
//! allowing you to create "vanity" SSH keys with specific patterns in the public key or fingerprint.
//!
//! ## Features
//!
//! - **Multi-threaded generation**: Utilizes all CPU cores for maximum performance
//! - **Regex pattern matching**: Full regex support for flexible pattern matching
//! - **Dual match modes**: Match against public key or SHA256 fingerprint
//! - **Streaming mode**: Continue searching for multiple matches
//! - **Real-time monitoring**: Live statistics on key generation rate
//! - **Graceful shutdown**: Clean termination with Ctrl+C
//!
//! ## Performance
//!
//! The tool generates thousands of keys per second, with actual performance depending on:
//! - CPU core count and clock speed
//! - Regex pattern complexity
//! - Match target (fingerprint matching is slightly faster)
//!
//! ## Architecture
//!
//! ```text
//! Main Thread
//!   ├─ Parse CLI arguments
//!   ├─ Compile regex pattern
//!   ├─ Setup Ctrl+C handler
//!   ├─ Spawn N worker threads (N = CPU cores)
//!   │   └─ Each worker:
//!   │       - Generate key pair
//!   │       - Test against regex
//!   │       - Save on match (unless streaming)
//!   └─ Monitor loop (250ms interval):
//!       - Display progress/stats
//!       - Calculate moving average
//! ```

use clap::Parser;
use colored::*;
use ed25519_dalek::{SigningKey, VerifyingKey};
use humansize::{format_size, DECIMAL};
use regex::Regex;
use ssh_key::{LineEnding, PrivateKey};
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Command-line arguments for SSHChic
///
/// This structure defines all available CLI options for controlling
/// the key generation and pattern matching behavior.
///
/// # Examples
///
/// ```bash
/// # Search for keys ending with "SSH"
/// sshchic --regex "SSH$"
///
/// # Case-insensitive search for "github" in fingerprint
/// sshchic --regex "github" --insensitive --fingerprint
///
/// # Streaming mode to find multiple matches
/// sshchic --regex "^AAAA" --streaming
/// ```
#[derive(Parser, Clone)]
#[command(author, version, about)]
struct Args {
    /// Regex pattern to match against the generated SSH keys
    ///
    /// The pattern uses standard regex syntax. The match target depends
    /// on the `--fingerprint` flag:
    /// - Without flag: matches against the OpenSSH public key format
    /// - With flag: matches against the SHA256 fingerprint (base64 encoded)
    ///
    /// # Examples
    ///
    /// - `"^AAAA"` - Keys starting with AAAA
    /// - `"SSH$"` - Keys ending with SSH
    /// - `"[0-9]{4}"` - Keys containing 4 consecutive digits
    #[arg(short, long, help = "Regex pattern to search for")]
    regex: String,

    /// Enable case-insensitive pattern matching
    ///
    /// When enabled, the regex pattern will match regardless of case.
    /// For example, "ssh" will match "SSH", "ssh", or "SsH".
    #[arg(short, long, help = "Enable case-insensitive matching")]
    insensitive: bool,

    /// Continue generating keys after finding matches (streaming mode)
    ///
    /// By default, the program stops after finding the first match.
    /// With this flag enabled, it will continue searching and display
    /// all matching keys. Keys are NOT saved to files in streaming mode.
    ///
    /// **Warning**: This mode will consume significant CPU resources.
    #[arg(short, long, help = "Keep processing keys, even after a match")]
    streaming: bool,

    /// Match against the key's SHA256 fingerprint instead of the public key
    ///
    /// When enabled, the regex pattern is tested against the base64-encoded
    /// SHA256 fingerprint rather than the OpenSSH public key format.
    /// Fingerprint matching is typically slightly faster.
    #[arg(short, long, help = "Match against fingerprint instead of public key")]
    fingerprint: bool,
}

/// Global atomic counter tracking the total number of keys processed across all threads
///
/// This counter is incremented atomically by each worker thread for every key pair generated.
/// It's used for progress reporting and statistics calculation in the main monitoring loop.
///
/// The counter uses `SeqCst` ordering to ensure consistency across threads.
static COUNTER: AtomicI64 = AtomicI64::new(0);

/// Generates a new ED25519 key pair using cryptographically secure random number generation
///
/// This function creates a fresh ED25519 signing key using the thread-local random number
/// generator and derives the corresponding verifying (public) key from it.
///
/// # Returns
///
/// A tuple containing:
/// - `SigningKey`: The private key used for signing operations
/// - `VerifyingKey`: The public key derived from the signing key
///
/// # Security
///
/// This function uses `rand::thread_rng()` which provides cryptographically secure
/// random numbers suitable for key generation. Each call produces a unique,
/// unpredictable key pair.
///
/// # Examples
///
/// ```no_run
/// let (signing_key, verifying_key) = generate_key_pair();
/// // signing_key: used for SSH authentication
/// // verifying_key: distributed to servers in authorized_keys
/// ```
fn generate_key_pair() -> (SigningKey, VerifyingKey) {
    let signing_key = SigningKey::from_bytes(&rand::random());
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

/// Converts an ED25519 public key to OpenSSH authorized_keys format
///
/// This function takes a raw ED25519 verifying key and converts it to the
/// standard OpenSSH public key format that can be added to `~/.ssh/authorized_keys`
/// files on SSH servers.
///
/// # Arguments
///
/// * `public_key` - A reference to the ED25519 verifying key to convert
///
/// # Returns
///
/// A `String` containing the public key in OpenSSH format, which looks like:
/// `ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAA... [optional comment]`
///
/// # Examples
///
/// ```no_run
/// let (_, verifying_key) = generate_key_pair();
/// let authorized_key = get_authorized_key(&verifying_key);
/// // authorized_key can now be appended to ~/.ssh/authorized_keys
/// ```
fn get_authorized_key(public_key: &VerifyingKey) -> String {
    use ssh_key::{public::Ed25519PublicKey, public::KeyData};

    // Ed25519PublicKey is a newtype wrapper around [u8; 32]
    let ed25519_key = Ed25519PublicKey(*public_key.as_bytes());
    let key_data = KeyData::Ed25519(ed25519_key);
    let ssh_public_key = ssh_key::PublicKey::new(key_data, "Generated by SSHChic");
    ssh_public_key.to_string()
}

/// Calculates the SHA256 fingerprint of an ED25519 public key
///
/// This function computes the SHA256 hash of the raw public key bytes and
/// returns it as a base64-encoded string. This fingerprint format is commonly
/// used for key verification and identification.
///
/// # Arguments
///
/// * `public_key` - A reference to the ED25519 verifying key to fingerprint
///
/// # Returns
///
/// A `String` containing the base64-encoded SHA256 hash of the public key.
/// This is the same format displayed by `ssh-keygen -l` when prefixed with "SHA256:".
///
/// # Examples
///
/// ```no_run
/// let (_, verifying_key) = generate_key_pair();
/// let fingerprint = get_fingerprint(&verifying_key);
/// println!("Key fingerprint: SHA256:{}", fingerprint);
/// ```
///
/// # Note
///
/// The fingerprint is computed from the raw key bytes, not the OpenSSH format.
/// This matches the standard SSH fingerprint calculation.
fn get_fingerprint(public_key: &VerifyingKey) -> String {
    use base64::{engine::general_purpose, Engine as _};
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(public_key.as_bytes());
    general_purpose::STANDARD.encode(hasher.finalize())
}

/// Worker thread function that continuously generates and tests SSH keys against a regex pattern
///
/// This is the core search function executed by each worker thread. It runs in a loop,
/// generating ED25519 key pairs and testing them against the provided regex pattern.
/// When a match is found, it displays the keys and optionally saves them to files.
///
/// # Arguments
///
/// * `regex` - The compiled regex pattern to match against
/// * `args` - Command-line arguments controlling match behavior
/// * `running` - Atomic flag to signal when the thread should terminate
///
/// # Behavior
///
/// For each iteration:
/// 1. Increments the global `COUNTER` atomically
/// 2. Generates a new ED25519 key pair
/// 3. Tests against either fingerprint or public key (based on `args.fingerprint`)
/// 4. On match:
///    - Prints the private key, public key, and fingerprint
///    - In non-streaming mode: saves to `id_ed25519` and `id_ed25519.pub`, then exits
///    - In streaming mode: continues searching for more matches
///
/// # Thread Safety
///
/// This function is designed to be called from multiple threads simultaneously.
/// All shared state access uses atomic operations to ensure thread safety.
///
/// # Examples
///
/// ```no_run
/// let regex = Regex::new("SSH$").unwrap();
/// let args = Args { /* ... */ };
/// let running = Arc::new(AtomicBool::new(true));
///
/// // Spawn worker thread
/// thread::spawn(move || {
///     find_ssh_keys(&regex, &args, running);
/// });
/// ```
fn find_ssh_keys(regex: &Regex, args: &Args, running: Arc<AtomicBool>) {
    while running.load(Ordering::SeqCst) {
        // Increment the global counter atomically
        COUNTER.fetch_add(1, Ordering::SeqCst);
        let (signing_key, verifying_key) = generate_key_pair();

        // Match against either fingerprint or public key based on args
        let matched = if args.fingerprint {
            regex.is_match(&get_fingerprint(&verifying_key))
        } else {
            regex.is_match(&get_authorized_key(&verifying_key))
        };

        if matched {
            println!("{}", "\nMatch found!".green());
            println!("Total keys processed: {}", COUNTER.load(Ordering::SeqCst));

            // Convert to OpenSSH private key format
            use ssh_key::private::{Ed25519Keypair, Ed25519PrivateKey, KeypairData};
            use ssh_key::public::Ed25519PublicKey;

            // Ed25519PrivateKey and Ed25519PublicKey are newtype wrappers
            let private_bytes = Ed25519PrivateKey(signing_key.to_bytes());
            let public_bytes = Ed25519PublicKey(*verifying_key.as_bytes());

            let keypair = Ed25519Keypair {
                private: private_bytes,
                public: public_bytes,
            };
            let keypair_data = KeypairData::Ed25519(keypair);

            let private_key = PrivateKey::new(keypair_data, "Generated by SSHChic")
                .expect("Failed to create private key");

            let public_key_str = get_authorized_key(&verifying_key);
            let private_key_str = private_key
                .to_openssh(LineEnding::LF)
                .expect("Failed to encode private key");

            println!("\nPrivate key:\n{}", private_key_str);
            println!("Public key:\n{}", public_key_str);
            println!("Fingerprint: SHA256:{}", get_fingerprint(&verifying_key));

            // Save keys to files unless in streaming mode
            if !args.streaming {
                fs::write("id_ed25519", private_key_str).expect("Failed to write private key");
                fs::write("id_ed25519.pub", public_key_str).expect("Failed to write public key");
                running.store(false, Ordering::SeqCst);
                break;
            }
        }
    }
}

/// Calculates an exponential moving average for smoothing key generation rate metrics
///
/// This function implements an exponential moving average (EMA) with a configurable time window.
/// It's used to smooth out fluctuations in the key generation rate display, providing a more
/// stable and readable metric.
///
/// # Arguments
///
/// * `value` - The new value to incorporate into the average
/// * `old_value` - The previous moving average value
/// * `delta_time` - Time elapsed since the last update (in seconds)
/// * `time_window` - The time constant for the exponential decay (in seconds)
///
/// # Returns
///
/// The updated exponential moving average value
///
/// # Algorithm
///
/// The function uses the formula:
/// ```text
/// alpha = 1 - exp(-delta_time / time_window)
/// new_avg = alpha * value + (1 - alpha) * old_value
/// ```
///
/// Where `alpha` represents the weight given to the new value. A larger `time_window`
/// results in slower adaptation to changes (smoother average).
///
/// # Examples
///
/// ```no_run
/// let mut avg_rate = 1000.0;  // Initial average
/// let new_rate = 1200.0;      // New measurement
/// let delta = 0.25;           // 250ms elapsed
/// let window = 5.0;           // 5-second smoothing window
///
/// avg_rate = exp_moving_average(new_rate, avg_rate, delta, window);
/// // avg_rate is now a smoothed value between 1000 and 1200
/// ```
///
/// # Performance Note
///
/// SSHChic uses a 5-second time window with updates every 250ms to balance
/// responsiveness with stability in the displayed key generation rate.
fn exp_moving_average(value: f64, old_value: f64, delta_time: f64, time_window: f64) -> f64 {
    let alpha = 1.0 - (-delta_time / time_window).exp();
    alpha * value + (1.0 - alpha) * old_value
}

fn main() {
    // Parse command line arguments
    let args = Args::parse();
    let regex_str = if args.insensitive {
        format!("(?i){}", args.regex)
    } else {
        args.regex.clone()
    };

    // Compile regex pattern
    let regex = match Regex::new(&regex_str) {
        Ok(re) => re,
        Err(e) => {
            eprintln!("Invalid regex pattern: {}", e);
            std::process::exit(1);
        }
    };

    println!("Using regex pattern: {}", regex_str);
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Set up Ctrl+C handler for graceful shutdown
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    // Create worker threads based on CPU count
    let num_threads = num_cpus::get();
    let mut handles = vec![];

    for _ in 0..num_threads {
        let regex_clone = regex.clone();
        let args_clone = args.clone();
        let running_clone = running.clone();

        handles.push(thread::spawn(move || {
            find_ssh_keys(&regex_clone, &args_clone, running_clone);
        }));
    }

    println!("Press Ctrl+C to stop");

    // Initialize performance monitoring variables
    let mut old_counter = 0i64;
    let mut old_time = Instant::now();
    let mut avg_key_rate = 0f64;

    // Main monitoring loop
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(250));
        let current_counter = COUNTER.load(Ordering::SeqCst);
        let elapsed = old_time.elapsed().as_secs_f64();

        if old_counter == 0 {
            avg_key_rate = current_counter as f64;
        }

        // Update progress display
        print!("{}", "\x1B[2K\r");
        print!(
            "Keys processed: {}",
            format_size(current_counter as u64, DECIMAL)
        );
        print!(" | Rate: {:.2} kKeys/s", avg_key_rate / elapsed / 1000.0);

        // Calculate moving average of key generation rate
        avg_key_rate = exp_moving_average(
            (current_counter - old_counter) as f64,
            avg_key_rate,
            elapsed,
            5.0,
        );
        old_counter = current_counter;
        old_time = Instant::now();
    }

    // Wait for all worker threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    println!("\nDone!");
}
