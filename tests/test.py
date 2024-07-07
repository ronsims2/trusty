# set the TRUSTY_HOME so that we can use a test db
from os import environ, popen
from subprocess import Popen, PIPE
import tempfile
import io

trusty_home = 'TRUSTY_HOME'
temp_dir = tempfile.gettempdir()
environ[trusty_home] = temp_dir

print(f'trusty home dir: {environ.get(trusty_home)}')

# Test the help command
help_output = Popen('tru --help', shell=True, stdout=PIPE).stdout.read()
assert 'tRusty: a command line notes app  🦀📝' in help_output.decode()







