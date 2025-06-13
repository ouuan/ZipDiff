import sys
import json
from os.path import dirname

PARSER_COUNT = 50
COL_WIDTH = '-2.5pt'

with open(f'{dirname(__file__)}/../constructions/inconsistency-types.json') as f:
    data = json.load(f)

s = f'\\begin{{tabular}}{{|*{{{PARSER_COUNT + 1}}}{{wc{{{COL_WIDTH}}}|}}}}\n\\hline\n'

for i in range(PARSER_COUNT):
    s += f' & {i + 1}'

s += r' \\ \hline' + '\n'

total_types = 0
total_pairs = 0

for i in range(PARSER_COUNT):
    s += f'{i+1}'
    for j in range(PARSER_COUNT):
        x = len(data[i * PARSER_COUNT + j]['inconsistency_types'])
        total_types += x
        if x > 0:
            total_pairs += 1
        s += f' & \\cellcolor{{blue!{0 if x == 0 else x * 3 + 10}}}{"-" if i == j else x}'
    s += r' \\ \hline' + '\n'

s += '\\end{tabular}'
print(s)

total_types /= 2
total_pairs /= 2
print(f'{total_types = }\n{total_pairs = }', file=sys.stderr)
