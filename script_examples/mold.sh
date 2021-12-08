#!/bin/sh

curl -s https://github.com/rui314/mold/releases |  rg 'tag/(v.*?)"' -o -r '$1' | head -n 1

