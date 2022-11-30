import os
import sys
import json
import glob

def fix_name(s):
    r = ''
    for ch in s:
        if ch == '_':
            r += '\\';
        r += ch
    return r

args = sys.argv
outdir = args[1]

result_jsons = glob.glob(f'{outdir}/*.json')
data=[]
for json_path in result_jsons:
    circ_name = os.path.splitext(os.path.basename(json_path))[0]
    with open(json_path) as f:
        data.append((circ_name, json.load(f)))

s = '\\begin{tabular}{|'
if len(data) > 0:
    keys = data[0][1].keys()
    s += 'r|' * (len(keys) + 1) + '} \\hline\n'
    # header
    s += 'circuit name'
    for key in keys:
        s += ' & ' + fix_name(key)
    s += ' \\\\ \\hline\\hline\n'

    # each data
    for i in range(len(data)):
        (name, json) = data[i]
        s += fix_name(name)
        for key in keys:
            s += ' & ' + str(json[key])
        s += ' \\\\ \\hline\n'

    s += '\\end{tabular}'
    print(s)

