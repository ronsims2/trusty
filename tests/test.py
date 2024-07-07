# set the TRUSTY_HOME so that we can use a test db
from os import environ
import subprocess
import tempfile
import io
from contextlib import redirect_stdout,redirect_stderr

trusty_home = 'TRUSTY_HOME'
temp_dir = tempfile.gettempdir()
environ[trusty_home] = temp_dir

print(f'trusty home dir: {environ.get(trusty_home)}')

std_o = io.StringIO()
with redirect_stdout(std_o):
    subprocess.run(['tru', '--help'])

print(f'STD OUT: {std_o.getvalue()}')





