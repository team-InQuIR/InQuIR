import os
import sys
import glob
import matplotlib.pyplot as plt
import seaborn as sns
import pandas
import argparse

parser = argparse.ArgumentParser()
parser.add_argument('file', help='a file where timestamps were stored.', type=str)
parser.add_argument('-o', '--out', type=str)
#parser.add_argument('-t', '--type', default='png', type=str)
parser.add_argument('--interval', default=1000, type=int)
args = parser.parse_args()

# read inputs
path = args.file
#filetype = args.type

if args.out is None:
    out_path = os.path.splitext(os.path.basename(path))[0]
else:
    out_path = args.out

interval = args.interval


def load_as_2darray(path):
    data = []
    with open(path) as f:
        for line in f:
            tmp = []
            for time in line.split(','):
                try:
                    num = int(time)
                except ValueError as e:
                    print(e, file=sys.stderr)
                    exit(0)
                tmp.append(num)
            data.append(tmp)
        return data

def df_of_timestamps(data):
    max_value = 0
    for times in data:
        max_value = max(max_value, times[len(times) - 1])
    arr = [[0] * (max_value + 1) for i in range(len(data))]
    for i in range(len(data)):
        times = data[i]
        arr[i][0] = len(times)
        for t in times:
            arr[i][t] -= 1;
        for j in range(0, len(arr[i])):
            arr[i][j] += arr[i][j - 1]
        arr[i] = arr[i][0:len(arr[i]):interval]

    max_value = len(arr[0])
    df = pandas.DataFrame(arr, range(len(data)), range(max_value)).T
    return df


data = load_as_2darray(path)
df = df_of_timestamps(data)

sns.set(font_scale = 1.4)
p = sns.lineplot(df)
p.set_ylabel('#(Remaining Ops)')
if interval == 1000:
    p.set_xlabel('Time [μs]')
elif interval < 1000:
    p.set_xlabel(f'Time [×{interval} ns]')
else:
    raise ValueError('interval must not be more than 1000.')

plt.tight_layout()
plt.savefig(out_path, dpi=300)
