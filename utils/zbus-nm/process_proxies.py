#!/bin/python
# SPDX-FileCopyrightText: 2022 Christopher A. Grant <grantchristophera@gmail.com>
#
# SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

"""Generates a zbus API for NetworkManager. This is a script."""

import argparse
import os
import pathlib
import subprocess
import sys

parser = argparse.ArgumentParser(
        description="Generate a zbus API for NetworkManager",
        epilog="This is not a stable program.")

parser.add_argument('target',
        type=pathlib.Path,
        help="path to the target source file")

parser.add_argument('--dry',
        action="store_true",
        help="run without writing to target")

parser.add_argument('--verbose',
        action="store_true",
        help="spew forth information from the generation")

args = parser.parse_args()

MAGIC_INTERFACE_PATH = "/usr/share/dbus-1/interfaces/"
MAGIC_INTERFACE = "org.freedesktop.NetworkManager"
buf = ""

def indenttext(string, indentlevel):
    """Splits text by lines and applies indents in a, probably inefficient, readable way."""
    retval = ""
    for line in string.splitlines():
        retval += "    " * indentlevel + line + "\n"
    return retval

#Check if file exists, ask if user is OK overwriting it.
if not args.dry and os.access(args.target, os.F_OK):
    answer = input(f"Overwrite {args.target} [y/N]? ")
    if answer.upper() not in ["Y", "YE", "YES"]:
        sys.exit()

#Find all interface definitions.
paths = [i for i in os.scandir(MAGIC_INTERFACE_PATH) if MAGIC_INTERFACE in i.name]

#FIXME: Disable warnings for snake_case
buf += "#![allow(non_snake_case)]"

#Open the proxies module
buf += "pub(crate) mod NetworkManager {\n"
indent = 1
for path in paths:
    interfacename = path.name.removesuffix(".xml") 
    
    if args.verbose:
        print(f"zbus-xmlgen {path.path}")

    #run zbus-xmlgen to generate base module code
    #TODO: modify all traits into snake_case
    genresult = subprocess.run(["zbus-xmlgen", path.path],
            capture_output=True,
            check=True,
            encoding="utf-8",
            text=True)

    #Given that it is not a file per proxy, replace generated outer comments.
    text = genresult.stdout.replace("//!", "///")

    #NetworkManager interface does not need its own sub-module.
    if interfacename == MAGIC_INTERFACE:
        buf += indenttext(text, indent)
        continue

    #TODO: VPN Plugin is not used and not currently necessary. The generated module breaks.
    if interfacename == "org.freedesktop.NetworkManager.VPN.Plugin":
        continue 

    #Strip the interface prefix info off to appease rust naming convention
    modname = interfacename.replace("org.freedesktop.", "")
    modname = modname.replace("NetworkManager.", "")
    modname = modname.replace(".", "")

    buf += indenttext(f"pub(crate) mod {modname} {{\n", indent) #} escaped lbrace
    indent += 1
    buf += indenttext(text, indent)
    indent -= 1
    buf += indenttext("}\n", indent)
#Close proxies module
indent -= 1
buf += "}\n"
assert indent == 0

"""
TODO: Fix state functions. 
The signals (state_changed) and the methods (state) interfere with each other. 
Additionally the state property in NetworkManager interferes with the state method.
For now, just remove them as they are not currently necessary.

#line = line.replace("fn state", "fn sstate") #rename the token as to not screw up the macro for now
#if line.find("fn state") > 0: #fragile
#    line = "//" + line
"""

if args.verbose:
    print(buf)

if not args.dry:
    with args.target.open("w", encoding="utf-8") as file:
        file.write(buf)
