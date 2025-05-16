use anyhow::Result;
use duct::cmd;
use std::path::{Path, PathBuf};

use crate::algorithm::Algorithm;
use crate::promela::{self, prepare_promela_code};

use log::{debug, trace};

const TRAIL_FILENAME: &str = "MainGathering.pml.trail";
const VOLUME: &str = "SynthLightsRamDisk";

#[derive(Debug)]
pub enum Workdir {
    Ramdisk(String, PathBuf),
}
impl Workdir {
    pub fn path(&self) -> &Path {
        match self {
            Workdir::Ramdisk(_, path) => path,
        }
    }
}

/// creates a root working directory (e.g, a ramdisk) and returns
/// a handle containing a path to the volume as well as the device name
///
/// # Examples
///
/// ```no_run
/// # use synth_lights::algorithm::Algorithm;
/// # fn foo(algo: Algorithm) -> std::io::Result<()> {
/// #   use synth_lights::runner::*;
/// //    let algo: Algorithm = /* ... */
///     let root_name: String = "MyRoot".into();
///     let workdir   = create_root_workdir(Some(root_name))?;
///     let enclosure = create_enclosure(workdir.path())?;
///     // ... do something with enclosure.
///     run_verification(&enclosure, &algo, "ASYNC")?;
///     close_workdir(workdir).unwrap();
/// #   Ok(())
/// # }
/// ```
pub fn create_root_workdir(ramdisk: Option<String>) -> Result<Workdir> {
    trace!("create_root_workdir({:?})", ramdisk);
    let ramdisk = ramdisk.unwrap_or_else(|| VOLUME.into());
    const SIZE: u16 = 512;

    let (dev, path) = ramdisk::create_ramdisk(SIZE, ramdisk.as_str())?;

    Ok(Workdir::Ramdisk(dev, path))
}

/// closes a working directory (e.g, unmount the ramdisk).
pub fn close_workdir(workdir: Workdir) -> Result<()> {
    trace!("close_workdir({:?})", workdir);
    ramdisk::eject_ramdisk(workdir.path())?;

    Ok(())
}

/// creates a subdirectory (enclosure) as a working space for a thread,
/// and returns a path to the newly created directory.
/// The call prepares the Promela code by calling [prepare_promela_code()]
/// in the created directory.
///
/// # Arguments
///
/// * `path` - a path where the enclosure will be created.
///
pub fn create_enclosure(path: &Path) -> Result<PathBuf> {
    let my_uuid = uuid::Uuid::new_v4();
    let dirname = format!("enclosure-{:x}", my_uuid);
    let mut path = PathBuf::from(path);
    path.push(dirname);

    // create the enclosure directory
    std::fs::create_dir(&path)?;
    // install the files
    prepare_promela_code(&path)?;

    Ok(path)
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpinOutcome {
    Fail, //< the verification fails. Details or counter-example should be obtained via regular verification.
    SearchIncomplete, //< the verification process is unconclusive because the search was incomplete.
    Pass,             //< the algorithms passes the check.
}
impl SpinOutcome {
    pub fn is_fail(&self) -> bool {
        self == &SpinOutcome::Fail
    }
}
impl std::fmt::Display for SpinOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fail => write!(f, "fail"),
            Self::Pass => write!(f, "PASS"),
            Self::SearchIncomplete => write!(f, "Incomplete"),
        }
    }
}

/// runs the verification proper on the given algorithm,
/// assuming that all promela files are already installed at the given path.
/// This includes the following:
/// 1. generate the algorithm file (TODO)
/// 2. run spin to create the `pan.c` program
/// 3. compile `pan.c` using `clang`.
/// 4. run the `pan` program to conduct the verification proper.
///
/// # Arguments
///
/// * `dir`   - path to the directory holding the promela files
/// * `algo`  - algorithm to verify
/// * `sched` - name of the scheduler to be used for the verification (check promela files or see below).
///
/// # Outputs
///
/// Returns an error if any of the operation fails.
///
/// _TODO_: will return adequate information regarding the outcome of the verification.
///
/// # Valid scheduler names
///
/// * ASYNC
/// * SSYNC
/// * FSYNC
/// * ... _see [`Scheduler`]_
pub fn run_verification<T>(dir: &Path, algo: &Algorithm, spin_args: T) -> Result<SpinOutcome>
where
    T: IntoIterator,
    T::Item: Into<String>,
{
    debug!("run_verification({:?}, {:?}, spin_args)", dir, algo);
    let mut trail_file: PathBuf = dir.to_path_buf();
    trail_file.push(TRAIL_FILENAME);
    let trail_file = trail_file.as_path();

    if trail_file.exists() {
        std::fs::remove_file(trail_file)?;
    }
    if trail_file.exists() {
        eprintln!("ERROR: trail file was not deleted");
    }

    let _ = promela::install_algorithm(dir, algo)?;
    run_spin_and_model(dir, trail_file, spin_args)
}

pub fn run_verification_from_code<T>(dir: &Path, algo: &str, spin_args: T) -> Result<SpinOutcome>
where
    T: IntoIterator,
    T::Item: Into<String>,
{
    debug!("run_verification({:?}, {:?}, spin_args)", dir, algo);
    let mut trail_file: PathBuf = dir.to_path_buf();
    trail_file.push(TRAIL_FILENAME);
    let trail_file = trail_file.as_path();

    if trail_file.exists() {
        std::fs::remove_file(trail_file)?;
    }
    if trail_file.exists() {
        eprintln!("ERROR: trail file was not deleted");
    }

    let _ = promela::install_algorithm_from_code(dir, algo)?;
    run_spin_and_model(dir, trail_file, spin_args)
}

pub fn read_trail_file(dir: &Path) -> Result<Option<String>> {
    let mut trail_file: PathBuf = dir.to_path_buf();
    trail_file.push(TRAIL_FILENAME);
    let trail_file = trail_file.as_path();

    if trail_file.exists() {
        return Ok(Some(std::fs::read_to_string(trail_file)?));
    } else {
        Ok(None)
    }
}

fn run_spin_and_model<T>(dir: &Path, trail_file: &Path, spin_args: T) -> Result<SpinOutcome>
where
    T: IntoIterator,
    T::Item: Into<String>,
{
    debug!("run_spin_and_model({:?}, {:?}, spin_args)", dir, trail_file);
    let _s = run_spin(dir, spin_args)?;
    let _c = run_clang(dir)?;
    let check_result = run_pan(dir)?;

    if trail_file.exists() {
        return Ok(SpinOutcome::Fail);
    }
    Ok(outcome_from_output(&check_result))
}

fn outcome_from_output(check_result: &str) -> SpinOutcome {
    trace!("outcome_from_output({})", check_result);
    let found_warning = check_result
        .lines()
        .any(|l| l.starts_with("Warning: Search not completed"));
    if found_warning {
        SpinOutcome::SearchIncomplete
    } else {
        SpinOutcome::Pass
    }
}

fn run_spin<T>(dir: &Path, spin_args: T) -> Result<String>
where
    T: IntoIterator,
    T::Item: Into<String>,
{
    let mut args = vec!["-a".to_string(), "-DALGO=SYNTH".to_string()];
    for x in spin_args {
        args.push(x.into());
    }
    args.push("MainGathering.pml".to_string());

    trace!("run_spin({:?}, {:?})", dir, args);

    cmd("spin", args)
        .dir(dir)
        .read()
        .map_err(anyhow::Error::new)
}

fn run_clang(dir: &Path) -> Result<String> {
    trace!("run_clang({:?})", dir);
    cmd!(
        "clang",
        "-DMEMLIM=16384",
        "-DXUSAFE",
        "-DNOREDUCE",
        "-O2",
        "-w",
        "-o",
        "pan",
        "pan.c"
    )
    .dir(dir)
    .read()
    .map_err(anyhow::Error::new)
}

fn run_pan(dir: &Path) -> Result<String> {
    trace!("run_pan({:?})", dir);
    let full_pan = dir.join("pan");
    let full_pan = full_pan
        .to_str()
        .ok_or_else(|| anyhow::Error::msg("Cannot convert path to str"))?;
    cmd!(full_pan, "-m100000", "-a", "-f", "-E", "-n", "gathering")
        .dir(dir)
        .read()
        .map_err(anyhow::Error::new)
}

mod ramdisk {
    #![allow(unused_imports)]

    //! **Architecture-specific** module, grouping functions to create and close a RAM disk.
    //! The functionality is only fully functional on macOS simply because I don't know how to do it
    //! on other platforms cleanly.
    //! I have implemented some kludges to get it running on Linux, but this requires some manual preparations as follows:
    //! ```shell
    //! sudo mkdir /mnt/tmp/SynthLightsRamDisk
    //! sudo mount -t tmpfs -o size=2g tmpfs /mnt/tmp/SynthLightsRamDisk
    //! ```
    //!
    //! The module exports two functions, namely [create_ramdisk] to create a new ramdisk of
    //! a given size, and [eject_ramdisk()] to close it afterwards.
    //!
    //! Example:
    //! ```no_run
    //! # use super::ramdisk;
    //! # fn main() -> Result<()> {
    //!     let (dev, path) = create_ramdisk(100, "my_ram_disk")?;
    //!     // do some stuff under directory path
    //!     eject_ramdisk(dev);
    //! # Ok(())
    //! # }
    //! ```
    use duct::cmd;
    use log::trace;
    use std::io::{self, ErrorKind};
    use std::path::{Path, PathBuf};
    use std::process::Output;

    #[cfg(target_os = "macos")]
    fn run_hdiutil(size_mb: u16) -> std::io::Result<String> {
        trace!("run_hdiutil({})", size_mb);
        let size: usize = size_mb as usize * 2048;
        let ram = format!("ram://{size}");
        cmd!("hdiutil", "attach", "-nomount", ram).read()
    }

    #[cfg(target_os = "macos")]
    fn run_diskutil(device: &str, volume: &str) -> std::io::Result<Output> {
        trace!("run_diskutil({}, {})", device, volume);
        cmd!(
            "diskutil",
            "partitionDisk",
            device,
            "1",
            "GPTFormat",
            "APFS",
            volume,
            "100%"
        )
        .stdout_capture()
        .stderr_capture()
        .run()
    }

    #[cfg(target_os = "linux")]
    fn create_mount_point(path: &Path) -> std::io::Result<()> {
        if path.exists() {
            if path.is_dir() {
                Ok(())
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!("mount point cannot be created: file exists at {:?}", path),
                ))
            }
        } else {
            // sudo mkdir /mnt/tmp/SynthLightsRamDisk
            cmd!("sudo", "mkdir", path)
                .stdout_capture()
                .stderr_capture()
                .run()?;
            Ok(())
        }
    }

    #[cfg(target_os = "linux")]
    fn mount_filesystem(path: &Path) -> std::io::Result<Output> {
        // sudo mount -t tmpfs -o size=2g tmpfs /mnt/tmp/SynthLightsRamDisk
        cmd!("sudo", "mount", "-t", "tmpfs", "-o", "size=2g", "tmpfs", path)
            .stdout_capture()
            .stderr_capture()
            .run()
    }

    #[allow(unused_variables)]
    pub fn create_ramdisk(size_mb: u16, volume: &str) -> std::io::Result<(String, PathBuf)> {
        #[cfg(target_os = "macos")]
        {
            let path: PathBuf = ["/Volumes", volume].into_iter().collect();

            if size_mb < 2 {
                return Err(io::Error::new(
                    ErrorKind::Other,
                    format!("'size' must be at least 2MB. Found: {size_mb}"),
                ));
            }
            if path.exists() {
                return Err(io::Error::new(
                    ErrorKind::AlreadyExists,
                    format!("Volume already exists: {:?}", path),
                ));
            }

            // make the device
            let res = run_hdiutil(size_mb)?;
            let devname = res.trim().to_string();

            // create the filesystem
            let _ = run_diskutil(&devname, volume)?;

            if !path.exists() {
                return Err(io::Error::new(
                    ErrorKind::NotFound,
                    format!("Volume not properly created: {:?}", path),
                ));
            }
            if !path.is_dir() {
                return Err(io::Error::new(
                    ErrorKind::NotConnected,
                    format!("Failed to mount filesystem: {:?}", path),
                ));
            }
            Ok((devname, path))
        }
        #[cfg(target_os = "linux")]
        {
            let path: PathBuf = ["/", "mnt", "tmp", volume].iter().collect();
            // create the enclosure directory
            create_mount_point(&path)?;
            mount_filesystem(&path)?;
            Ok(("tmpfs".to_string(), path.to_owned()))
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            let path: PathBuf = PathBuf::from(volume);
            // create the enclosure directory
            std::fs::create_dir(&path)?;
            Ok(("local directory".to_string(), path.to_owned()))
        }
    }

    pub fn eject_ramdisk(path: &Path) -> std::io::Result<()> {
        #[cfg(target_os = "macos")]
        {
            cmd!("diskutil", "eject", path)
                .stdout_capture()
                .stderr_capture()
                .run()?;
            Ok(())
        }
        #[cfg(target_os = "linux")]
        {
            cmd!("sudo", "umount", path)
                .stdout_capture()
                .stderr_capture()
                .run()?;
            Ok(())
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Ok(())
        }
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_ramdisk() {
            let (_, path) = create_ramdisk(10, "Ramdisk1").unwrap();

            assert!(path.exists());
            assert!(path.is_dir());

            let mut test_file = path.clone();
            test_file.push("test_file.txt");
            let test_file = test_file.as_path();

            let content = "This is a test";
            let res = std::fs::write(test_file, content);
            assert!(res.is_ok());
            assert!(test_file.exists());

            assert_eq!(std::fs::read_to_string(test_file).unwrap(), content);

            eject_ramdisk(&path).unwrap();
            assert!(!path.exists());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::promela;

    #[test]
    fn test_enclosure() {
        const TEST_VOLUME: &str = "TestRamDisk_enclosure";

        let workdir = create_root_workdir(Some(TEST_VOLUME.into())).unwrap();
        let enclosure = create_enclosure(workdir.path()).unwrap();

        for (fname, _) in promela::PML_FILES {
            let fpath: PathBuf = [&enclosure, &PathBuf::from(fname)].into_iter().collect();
            eprintln!("> {:?}", fpath.file_name());
            assert!(fpath.exists());
            assert!(fpath.is_file());
            let content = std::fs::read_to_string(fpath).unwrap();
            assert!(content.trim_start().starts_with("#ifndef"));
        }

        eprintln!("workdir: {:?}", workdir);
        close_workdir(workdir).unwrap();
    }
}
