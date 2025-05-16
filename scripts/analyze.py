import re
import os
import pandas as pd
from os.path import isfile, join
from pathlib import Path
from common import *


OUTPUT_FILE = WORK_DIR / "summary.csv"


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
        print(filename)
        if m := re.match(OUTPUT_FILENAME_R, filename):
            model_L = 'L' if m.group(1) == "L_" else ' '
            model_lights = m.group(2)
            model_cols = int(m.group(3))
            model_sched = m.group(4)
            algos = get_pass_list(WORK_DIR / filename)
            for algo in algos:
                models.append({ 'lights':model_lights, 'colors':model_cols, 'classL':model_L, 'scheduler':model_sched, 'file':filename, 'num':algo['num'], 'code':algo['code'] })
    return models

def get_pass_list(from_file):
    """
    Given a filename, outputs a list of all algorithms that are reported as PASS.
    The list is a dictionary with the algorithm number as key and the algorithm code string as value.
    """
    with open(from_file, 'r') as f:
        algo_list = []
        for l in f.readlines():
            if m := re.match(PASS_LINE_R, l):
                algo_num  = int(m.group(1))
                algo_code = m.group(2)
                algo_list.append( {'num':algo_num, 'code': algo_code} )
        return algo_list    


if __name__ == '__main__':    
    model_list = get_models(WORK_DIR)
    
    df = pd.DataFrame.from_records(model_list)
    df['ones'] = 'O'
    df = (df[['classL', 'lights', 'colors', 'num', 'code', 'scheduler', 'ones']]
          .pivot(index=['lights', 'classL', 'colors', 'num', 'code'], columns='scheduler', values='ones')
          .fillna(value='')
    )
    # reorder columns
    columns = [ x for x in MODELS if x in df.columns and x not in SKIP_MODELS ]
    df = df[columns]

    df.to_csv(OUTPUT_FILE)
    print(df)
    
