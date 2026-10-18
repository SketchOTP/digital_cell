#!/usr/bin/env python3
import argparse,json,pathlib
from empirical_constituent import analyze
p=argparse.ArgumentParser();p.add_argument('--source',required=True);p.add_argument('--output',required=True);a=p.parse_args();root=pathlib.Path(__file__).resolve().parents[1];report=analyze(a.source,json.loads((root/'docs/dc-m4-empirical-constituent-protocol.json').read_text()));out=pathlib.Path(a.output);out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
