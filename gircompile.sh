#!/bin/sh
set -e
gir -c Gir.toml -m normal -o evolution-glue
gir -c Gir.toml -m sys -o evolution-glue-sys
