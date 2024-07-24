import os
import sys


temp_file = sys.argv[1]
# this just test to see if the internal api passes a temp file.  This implies that the app code is correct.
if os.path.isfile(temp_file):
    exit(0)
else:
    exit(1)


