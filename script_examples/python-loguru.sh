#!/bin/sh
curl -s https://github.com/Delgan/loguru/releases | rg 'href="/Delgan/loguru/tree' | rg '/(\d.*?)"' -r '$1' -o | head -n 1
