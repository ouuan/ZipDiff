import os
import sys
import json
import requests
import subprocess
from typing import List, Dict

with open(f'{os.path.dirname(__file__)}/../parsers/parsers.json') as f:
    parser_map = json.load(f)

gh_token = os.environ.get('GITHUB_TOKEN') or subprocess.check_output(['gh', 'auth', 'token']).decode()
queries = []
for key, parser in parser_map.items():
    if 'github' not in parser:
        continue
    owner, name = parser['github'].split('/')
    queries.append(f'_{len(queries)}: repository(owner: "{owner}", name: "{name}") {{ stargazerCount nameWithOwner }}')
query = f"""query {{
    {'\n    '.join(queries)}
}}"""
response = requests.post(
    'https://api.github.com/graphql',
    headers={ "Authorization": f"token {gh_token.strip()}"},
    json={ "query": query }
)
if not response.ok:
    print(response.text)
    exit(1)
star_map = {}
for data in response.json()['data'].values():
    star_map[data['nameWithOwner']] = data['stargazerCount']

parsers : List[Dict[str, str]] = sorted(parser_map.values(), key = lambda p : (p['type'], p['language'], p['name'].lower(), p['version']))

for i, parser in enumerate(parsers):
    name = parser["name"]
    std = parser.get("std", False)
    lang = parser["language"]
    if std:
        lang += '*'
    ver = parser['version']
    repo = parser.get('github')
    link = parser.get('link')
    if repo:
        name = rf'\href{{https://github.com/{repo}}}{{{name}}}'
        star = star_map[repo]
        if star >= 1000:
            star = f'{star/1000:.1f}'.rstrip('0').rstrip('.')
            star += 'k'
    else:
        if link:
            name = rf'\href{{{link}}}{{{name}}}'
        else:
            print(f'no link for {name}', file=sys.stderr)
        star = '-'
    print(rf'        {i+1} & {name} & {lang} & {ver} & {star} \\ \hline'.replace('#', r'\#'))
