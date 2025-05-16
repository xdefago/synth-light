import re
import os
import sys
from itertools import groupby
from os.path import isfile, join
from pathlib import Path
from datetime import datetime
from common import *
import pandas as pd


NO_ALGO_FOUND = "\\FAIL"

TABLE_FILE = WORK_DIR / "summary_table.md"

LATEX_SCHED = {
    'centralized': "\\CENTRALIZED",
    'fsync': "\\FSYNC",
    'ssync': "\\SSYNC",
    'async-lc-strict': "LC-strict \\ASYNC",
    'async-lc-atomic': "LC-atomic \\ASYNC",
    'async-cm-atomic': "CM-atomic \\ASYNC",
    'async-move-atomic': "Move-atomic \\ASYNC",
    'async-move-regular': "Move-regular \\ASYNC",
    'async-move-safe': "Move-safe \\ASYNC",
    'async': "\\ASYNC",
    'async-regular': "\\ASYNC regular",
    'async-safe': "\\ASYNC safe"
}

LIGHTS = [ "full", "external", "internal" ]
CLASS_L = [ "L", " " ]


def get_models(dirname):
    """
    Given a directory, scans all reporting files and groups results according to:
    * light model: full, external, internal
    * number of colors
    * class L (or not)
    * name of scheduler
    The output is a list of entries, each of which consists of a dictionary with the
    following keys:
    * `lights`: (str) the light model; one of the strings 'full', 'external', 'internal'
    * `colors`: (int) the number of colors
    * `classL`: (bool) True if the algorithms are of class L
    * `scheduler`: (str) the name of the scheduler; e.g, 'async', 'async-lc-strict', ...
    * `file`: (str) the filename of the report
    * `algos`: (dict) the algorithms that PASS in the given model / scheduler
    """
    files = [f for f in os.listdir(dirname) if isfile(join(dirname, f))]
    models = []
    for filename in files:
        if m := re.match(OUTPUT_FILENAME_R, filename):
            model_L = 'L' if m.group(1) == "L_" else ' '
            model_lights = m.group(2)
            model_cols = int(m.group(3))
            model_sched = m.group(4)
            if model_sched in SKIP_MODELS:
                continue
            report = get_report_info(WORK_DIR / filename)
            if report:
                models.append({
                    'lights': model_lights,
                    'colors': model_cols,
                    'classL': model_L,
                    'scheduler':model_sched
                } | report )
            else:
                print("WARN: skipping", filename)
    return models

def get_report_info(from_file):
    """
    Given a filename, outputs a list of all algorithms that are reported as PASS.
    The list is a dictionary with the algorithm number as key and the algorithm code string as value.
    """
    with open(from_file, 'r') as f:
        cli_weak = False
        for l in f.readlines():
            if cli := cli_options(l):
                cli_weak = False
                if weak := cli.get('weak_filter') :
                    cli_weak = (weak == 'true')
            elif m := re.match(VERIF_FINISHED_LINE_R, l):
                algo_pass = int(m.group(1))
                algo_fail = int(m.group(2))
                algo_incom = int(m.group(3))
                algo_err   = int(m.group(4))
                algo_total = int(m.group(5))
                if algo_err > 0 or algo_incom > 0:
                    print(f"WARNING: {algo_err} errors {algo_incom} incomplete for {from_file}")
                return {'pass':algo_pass, 'fail': algo_fail, 'incomplete': algo_incom, 'errors': algo_err, 'total': algo_total, 'weak_filter': cli_weak}
        return None    
    

def model_from_entry(entry):
    return (entry['lights'], entry['colors'], entry['classL'])

def model_sort_key(model):
    (lights, colors, classL) = model
    return (LIGHTS.index(lights), CLASS_L.index(classL), colors)

def model_to_str(model):
    (lights, colors, classL) = model
    return f"{lights} {colors} {classL}"

def print_markdown(out, data):
    schedulers = [ x for x in MODELS if x not in SKIP_MODELS ]
    print("| model        |", " | ".join(schedulers), file=out)
    print("| -----        |", "----- | " * len(schedulers), file=out)
    for entry in data:
        (model, values) = entry
        print(f"| {model_to_str(model):12} |", file=out, end='')
        for sched in schedulers:
            if sched in values:
                n = values.get(sched)
                print(f" {n['pass']:5} |", file=out, end='')
            else:
                print("       |", file=out, end='')
        print(file=out)

def model_to_latex(model):
    (lights, colors, classL) = model
    if classL == "L":
        return f"\\Model{{{lights} {colors} $\\mathcal{{L}}$}}"
    else:
        return f"\\Model{{{lights} {colors}}}"

def bracketed(whatever):
    return f"({whatever})"

def print_latex_table(out, data):
    schedulers = [ x for x in MODELS if x not in SKIP_MODELS ]
    scheduler_legend = "\n".join([
        f"    \\slanted{{\\Head{{{LATEX_SCHED[sched]}}}}} & " for sched in schedulers
    ])
    
    preamble = f"""
%
% Script-generated from result output reports.
% python {sys.argv[0]}
% generated (UTC): {datetime.utcnow()}
%
\\begin{{tabular}}{{{'r@{~~}' * (len(schedulers)-1)}rl}}
{scheduler_legend}
    \\\\ \\toprule"""
    print(preamble, file=out)
     
    prev_lights = ''
    for entry in data:
        (model, values) = entry
        (lights, _, _) = model
        if prev_lights != lights:
            print("    %", file=out)
            print("    %     ", lights.upper(), file=out)
            print("    %", file=out)
            if prev_lights:
                print("    \\graymidrule", file=out)
            prev_lights = lights
        print(f"    % {model_to_str(model)}", file=out)
        print("    ", file=out, end='')
        total = 0
        for sched in schedulers:
            if sched in values:
                n = values.get(sched)
                total = n['total']
                n_pass = n['pass'] if n['pass'] > 0 else NO_ALGO_FOUND
                cell_str = f"\\itshape {bracketed(n_pass):4}" if n['weak_filter'] else f"{n_pass:13}"
                print(f"{cell_str} & ", file=out, end='')
            else:
                print(f"{' '*13} & ", file=out, end='')
        print(f"    {model_to_latex(model)}", file=out, end='')
        print(f" % TOTAL: {total}", file=out)
        print("    \\\\ ", file=out)

    print("    \\bottomrule", file=out)
    print(f"\\end{{tabular}}", file=out)


if __name__ == '__main__':    
    data = get_models(WORK_DIR)
    
    schedulers = [ x for x in MODELS if x not in SKIP_MODELS ]
    
    data.sort(key=lambda x: model_sort_key(model_from_entry(x)))
    
    model_list = [
        (model, {
            entry['scheduler']: {
                'pass':entry['pass'],
                'total':entry['total'],
                'fail':entry['fail'],
                'err':entry['errors'],
                'incom':entry['incomplete'],
                'weak_filter':entry['weak_filter'],
            } 
            for entry in values if entry['scheduler'] in schedulers
        })
        for model, values in groupby(data, model_from_entry)
    ]
    
    model_list.sort(key=lambda x: model_sort_key(x[0]))

    model_dict = [
        { 'model':model, 'ncols': ncols, 'classL': (classL == 'L'), 'sched':sched, 'pass':res['pass'], 'total':res['total'] }
        for ((model, ncols, classL), result_dict) in model_list for (sched, res) in result_dict.items()
    ]

    df = pd.DataFrame.from_records(model_dict)
    df.to_csv(OUTCOMES_FILE)
    
    print()
    print()
    print()
        
    print_markdown(sys.stdout, model_list)
    
    print()
    print()
    print()
    
    print_latex_table(sys.stdout, model_list)
    
    print()
    print()

