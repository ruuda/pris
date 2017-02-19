#!/usr/bin/env python3

import subprocess
import sys
import os
import os.path

root_dir = os.path.join(os.path.dirname(__file__), '..')


def run_pris(fname):
    """ Run pris on the given file. Stops at a nonzero exit code. """
    binpath = os.path.join(root_dir, 'target/debug/pris')
    proc = subprocess.run([binpath, fname],
                          stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    if proc.returncode != 0:
        print('FAILED {}\n'.format(fname))
        print(proc.stdout.decode('utf-8'))
        sys.exit(proc.returncode)
    else:
        print('OK {}'.format(fname))


for fname in os.listdir(os.path.join(root_dir, 'examples')):
    if fname.endswith('.pris'):
        run_pris(os.path.normpath(os.path.join(root_dir, 'examples', fname)))
