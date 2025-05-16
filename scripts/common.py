from pathlib import Path
import re

WORK_DIR = Path("results")

OUTCOMES_FILE = WORK_DIR / "outcomes.csv"

MODELS = [
    'centralized', 'fsync', 'ssync', 'async-lc-strict', 'async-lc-atomic',
    'async-cm-atomic', 'async-move-atomic', 'async-move-regular',
    'async-move-safe','async', 'async-regular', 'async-safe'
]
SKIP_MODELS = [
    'async-move-regular', 'async-move-safe', 'async-regular', 'async-safe'
]

VERIF_FINISHED_LINE_R = re.compile(r'Verification Finished with (\d+) pass, (\d+) fail, (\d+) incomplete, (\d+) errors \((\d+) algorithms\)')
OUTPUT_FILENAME_R = re.compile(r'parout_(L_)?(external|internal|full)_(\d+)_([a-z-_]+)(_rigid)?(_qss)?.txt')
CLI_RUN_OPTIONS_R = re.compile(r'Run options: Cli {(.*)}')
PASS_LINE_R = re.compile(r'\s*(\d+)\s*: PASS ([0-9sdSOH_]+)\s*')

def cli_options(line: str):
    """returns a dictionary of run options if the line matches an entry of run options, or None otherwise."""
    if m := re.match(CLI_RUN_OPTIONS_R, line):
        return { k.strip():v.strip() for k, v in map(lambda kv: kv.split(":"), m.group(1).split(",")) }
    else:
        return None

