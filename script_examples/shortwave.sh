#!/bin/sh
curl -s https://gitlab.gnome.org/World/Shortwave/-/jobs | rg flatpak | rg '/(\d+)' -r '$1' -o | head -n1
