# set the TRUSTY_HOME so that we can use a test db
from os import environ, popen
import subprocess
import tempfile
import io

trusty_home = 'TRUSTY_HOME'
temp_dir = tempfile.gettempdir()
environ[trusty_home] = temp_dir

print(f'trusty home dir: {environ.get(trusty_home)}')


output = popen('tru --help')

for line in output.readlines():
    print(f'STD OUT: 🐍{line}🐍')







