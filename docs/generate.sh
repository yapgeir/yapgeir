#!/bin/sh

cargo depgraph --workspace-only | dot -Tpng > dependencies.png