#!/bin/sh

# generate graphs from sources
dot -Tsvg src/assets/dependencies.dot -o src/assets/dependencies.svg
dot -Tsvg src/technical/api/rpc1.dot -o src/technical/api/rpc1.svg
dot -Tsvg src/technical/api/rpc2.dot -o src/technical/api/rpc2.svg

# build this mdbook
mdbook build
