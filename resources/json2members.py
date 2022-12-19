import argparse
import json

parser = argparse.ArgumentParser()
parser.add_argument('-i', '--input', help='Input file (JSON)', required=True)
parser.add_argument('-o', '--output', help='Output file (Tab-Separated Membership)', required=True)

args = parser.parse_args()

with open(args.input, 'r') as f:
    with open(args.output, 'w+') as g:
        for line in f:
            cluster = json.loads(line)
            label = cluster["label"]
            for node in cluster["nodes"]:
                g.write(f"{node}\t{label}\n")