#!/usr/bin/env python3
"""Run every physics check. Exit 0 means every claim the pages make holds."""
import sys
import bouncing_ball, optics, wave

failed = False
for mod in (optics, bouncing_ball, wave):
    try:
        print("PASS  " + mod.run())
    except AssertionError as e:
        failed = True
        print("FAIL  %s: %r" % (mod.__name__, e))
sys.exit(1 if failed else 0)
