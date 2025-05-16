# Synth-lights

## Description

This program aims to synthesize rendez-vous algorithms of two robots with lights, using model checking and exhaustive search.

Given the description of a model (light variant, number of colors, scheduler), the program generates all possible algorithms, filters those that are not viable, reduces the number to eliminate isomorphism, and checks them one-by-one using the SPIN model checker.

## Verification Model Parameters

### Configurable

* __Lights model__: full, external, internal
* __class_L__: class L algorithms have no access to relative position.
* __num_colors__: number of distinct colors available to the robots. A value of 1 is equivalent to having no colors.
* __scheduler__: level of synchronization considered (scheduler). The following schedulers are supported:
    * centralized
    * fsync
    * ssync
    * async-lc-strict
    * async-lc-atomic
    * async-cm-atomic
    * async-move-atomic
    * async-move-regular
    * async-move-safe
    * async
    * async-regular
    * async-safe

### Default

* __non-Quasi Self-Stabilizing__: the initial color of the robots is selected non-deterministically, but it is the same for both robots.
* __non-rigid__: non-rigid moves; the initial position of the robots is selected non-deterministically to be: Same, Near, or Far.

## Installation / Requirements

### Supported Environments

* MacOS/Darwin: tried on both Intel and ARM
* Linux: tried on Ubuntu 20 LTS

1. It is dependent on _macOS_-specific code to create a RAM disk. To run it on other platforms (e.g., Linux), one needs to circumvent the platform-specific code with some equivalent functionality (creating a designated directory is a possible option). Doing this requires to adapt the code. NB: now done for linux; but requires to enter administrator password (`sudo`).
1. The `spin` program (model-checker) and `clang` (compiler) must both be in the `$PATH`. They are not installed by cargo, neither are they checked. If absent, the program will simply fail with an error.

### MacOS
1. Install Xcode command-line tools
1. Install [Homebrew](https://brew.sh)
2. Download or clone synth-lights
3. `brew install rustup-init`
4. `rustup-init`
5. `brew install spin`
6. `cargo test`
7. `cargo build --release`
8. To execute the program: `cargo run --release --bin synth-lights -- ` followed by command-line arguments
    or `./target/release/synth-lights` followed by command-line arguments (try `-h` for help).

Alternative ways to execute:

* `cargo run --bin synth-lights -- -h` or `./target/release/synth-lights -h`
    to check command-line arguments
* `cargo run --bin synth-lights -- full 2` or `./target/release/synth-lights full 2`
    to check all algorithms in model full lights with 2 colors (non-L) and ASYNC scheduler (default) with reporting on `stdout`
* `./target/release/synth-lights -L -o output_file.txt -s async-lc-strict full 2`
    to check class L (flag `-L`) algorithms in full lights with 2 colors and ASYNC LC-strict (`-s` option) with reporting written in a file named `output_file.txt`.
* `./target/release/synth-lights -L -f -s centralized external 4`
    to check class L (flag `-L`) algorithms in external lights with 4 colors and centralized scheduler with reporting written to a file with default name (`parout_L_external_4_centralized.txt` in this case).
* `./target/release/synth-lights -L -f -S -s centralized external 4`
    same as above but execution is sequential (`-S`) instead of being parallel over all available CPU cores (default).


## Limitations:

The program is still incomplete yet and there are several major limitations.

* If the program is interrupted during the verification (e.g., via Ctrl-C), it will quit without closing the ramdisk which must then be ejected manually by running the following command in a terminal: `diskutil eject /Volumes/SynthLightsRamDisk`.

## Usage

```
USAGE:
    synth-lights [OPTIONS] <CATEGORY> <N_COLORS>

ARGS:
    <CATEGORY>    Category of algorithms [possible values: full, internal, external]
    <N_COLORS>    Number of colors allowed in the model

OPTIONS:
    -f, --file                 Write output to a file (use default filename made from command line
                               arguments if no name is specified with -o; stdout by default)
    -h, --help                 Print help information
    -L                         Limits search to class L algorithms
    -o, --out <OUTPUT_DIR>     Output file for reporting outcomes (-f is implicit if this option is
                               provided)
    -r, --ramdisk <RAMDISK>    
    -R                         Enables Viglietta's retain rule filtering ("A robot retains its color
                               if and only if it sees the other robot set to a different color.")
    -s, --sched <SCHEDULER>    Scheduler of the model [default: async] [possible values:
                               centralized, fsync, ssync, async-lc-strict, async-lc-atomic, async-
                               cm-atomic, async-move-atomic, async-move-regular,
                               async-move-safe, async, async-regular, async-safe]
    -S, --sequential           Enables sequential execution
    -V, --version              Print version information
    -w                         Enables weak filtering
```

### Examples

In order to check *sequentially* all algorithms in *full lights* with *2 colors* under an *SSYNC* scheduler,
using a ramdisk mounted as `/Volumes/SuperExternal`, the command is as follows:
```
synth-lights -S -r SuperExternal -s ssync full 2
```
or, to check in parallel all *class $\mathcal{L}$* algorithms in *external lights* with *4 colors* under an *SSYNC* scheduler
```
synth-lights -L -s ssync external 4
```


## Troubleshooting (mac-only)

Depending on error circumstances, it is possible that the ramdisk is not properly ejected. In that case, you need to eject it manually. The easy way is when it appears as a volume on the desktop; simply drag it to the trash to eject it. Otherwise, you need to do it manually as follows:

1. Find the correct device by executing `df` in a terminal.
1. Eject the device with `diskutil`, as follows: \
    `diskutil eject /dev/disk` (_complete with proper device name)_

# Other Tools

There are now two additional utility programs in directory (source code in `./src/bin`).

## Count algorithms

The program `count_filter` counts the number of algorithms generated and filtered for a given model.
It is executed as follows (example):

* `cargo run --bin count_filter -- -L external 5` or directly `./target/release/count_filter -L external 5`
    counts algorithms for model _external 5 L_.
* `cargo run --bin count_filter full 2`
    counts algorithms for model _full 2_.

## Translate algorithm code string

The program `algo_from_string` parses the code string of an algorithm and outputs its code in Promela.
It is executed as follows:

* `cargo run --bin algo_from_string -- -L external 4 0_1_2_3__S3_H0_O1_O2`
    outputs the promela code corresponding to the algorithm `0_1_2_3__S3_H0_O1_O2`.
* `cargo run --bin algo_from_string full 2 00s_01s_10s_11s_00d_01d_10d_11d__S0_S0_S1_S1_S1_S0_O1_H0`
    outputs the promela code corresponding to one of the algorithms working in ASYNC in model full 2.
    This outputs the code below:

```promela
#ifndef __ALGORITHMS_PML__
#define __ALGORITHMS_PML__
#  define ALGO_NAME      "ALGO_SYNTH_00s_01s_10s_11s_00d_01d_10d_11d__S0_S0_S1_S1_S1_S0_O1_H0"
#  define Algorithm(o,c) Alg_Synth(o,c)
#  define MAX_COLOR      (2)
#  define NUM_COLORS     (2)
inline Alg_Synth(obs, command)
{
    command.move      = STAY;
    command.new_color = obs.color.me;
    if
    :: (obs.color.me == 0) && (obs.color.other == 0) && (obs.same_position) -> command.move = STAY; command.new_color = 0;
    :: (obs.color.me == 0) && (obs.color.other == 1) && (obs.same_position) -> command.move = STAY; command.new_color = 0;
    :: (obs.color.me == 1) && (obs.color.other == 0) && (obs.same_position) -> command.move = STAY; command.new_color = 1;
    :: (obs.color.me == 1) && (obs.color.other == 1) && (obs.same_position) -> command.move = STAY; command.new_color = 1;
    :: (obs.color.me == 0) && (obs.color.other == 0) && ! (obs.same_position) -> command.move = STAY; command.new_color = 1;
    :: (obs.color.me == 0) && (obs.color.other == 1) && ! (obs.same_position) -> command.move = STAY; command.new_color = 0;
    :: (obs.color.me == 1) && (obs.color.other == 0) && ! (obs.same_position) -> command.move = TO_OTHER; command.new_color = 1;
    :: (obs.color.me == 1) && (obs.color.other == 1) && ! (obs.same_position) -> command.move = TO_HALF; command.new_color = 0;
    fi;
}
#endif
```

## Check a given algorithm in Promela

The program `model_check_algo` takes an algorithm written in VALID Promela code and runs it through the model checker.
Note that there is no validation of the code and malformed code will result in a non-descriptive error.

The expected format for the promela code is exactly the same as the example above, including all of the `#define`s. The exact string for `ALGO_NAME` is not important, the second define (`Algorithm(o,c)`) and the function name (`Alg_Synth(obs, command)`) must remain as is.

The program is executed as follows:
* `cargo run --bin model_check_algo -- -a <promela file.pml> --sched ssync`
    check the algorithm in the promela code with a semi-synchronous scheduler.

