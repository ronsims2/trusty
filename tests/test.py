# set the TRUSTY_HOME so that we can use a test db
# Test password: 1234
# test recovery code: 7e85e714-60fd-4e8c-a58e-73674314101c
import os
from os import environ, path, mkdir, getcwd, curdir, remove
from subprocess import Popen, PIPE, run
import tempfile
import pathlib
import shutil


curr_dir = getcwd()
# look for cargo.toml to make sure we are in the project root execution context
if not path.isfile(pathlib.Path(path.join(curdir, 'Cargo.toml'))):
    print('Could not find Cargo.toml, this doesn\'t look like a Rust project.')
    exit(1)

print(f'Current directory: {getcwd()}')
trusty_home = 'TRUSTY_HOME'
trusty_home_dir = tempfile.gettempdir()
trusty_config_dir = path.join(trusty_home_dir, '.trusty')


trusty_db_path = path.join(trusty_config_dir, 'trusty.db')

if not path.exists(trusty_config_dir):
    mkdir(trusty_config_dir)

    # clean up old database
if path.isfile(trusty_db_path):
    print('Cleaning up previous workspace.🧹')
    remove(trusty_db_path)


shutil.copyfile(path.join(getcwd(), 'tests', 'trusty.db'), trusty_db_path)
print('Initialized database 🧑🏽‍💻')
print(f'Database location: {trusty_db_path}')


environ[trusty_home] = trusty_home_dir
# the path to the built executable
tru = path.join(getcwd(), 'target', 'debug', 'tru')

print(f'{trusty_home} env var set to: {environ.get(trusty_home)}')

# Build debug tRusty
try:
    run('cargo build', shell=True, capture_output=True, text=True)
    print('Built tRusty app 🛠️')
except Exception as e:
    print('Could not build project')


# Test the help command
help_output = Popen(f'{tru} --help', shell=True, stderr=None, stdout=PIPE).stdout.read().decode()
assert 'tRusty: a command line notes app  🦀📝' in help_output
print('✅ --help test passed')

# Test tRusty --list
control_default_output = 'Get Started with tRusty'
default_output = Popen(f'{tru}', shell=True, stderr=None, stdout=PIPE).stdout.read().decode()
assert control_default_output in default_output
list_output = Popen(f'{tru}', shell=True, stderr=None, stdout=PIPE).stdout.read().decode()
assert control_default_output in list_output
print('✅ --list test passed')

# Test add note
control_title = '''🤣Foobar Barbaz 🥷 Bazbez Lorem ipsum dolor sit amet, 🐶 consectetur adipiscing elit.\r\n
Aliquam tellus nunc, tincidunt in placerat dictum, mattis non augue.\r\n 🦀'''

control_body = '''
Lorem  🤣 ipsum dolor sit amet, consectetur adipiscing elit.\r\n
Aliquam tellus nunc, tincidunt in placerat 🥷 dictum, mattis non augue.\r\n🥷
Quisque turpis nisl, feugiat nec metus et, fermentum bibendum ex.\r\n
Cras fringilla quam in odio 🐶 congue, eget rutrum felis fermentum.\r\n
Nam et ornare magna. Class aptent taciti sociosqu ad litora torquent per 🐶 conubia nostra, per inceptos himenaeos.\r\n
Vivamus semper ligula id felis pulvinar 🥷 venenatis. Aliquam urna risus, consequat non gravida ac, laoreet eu ex.\r\n
Nulla tincidunt, sem vitae luctus dignissim, 🥷 lacus nibh consequat erat, nec tristique ipsum dui et ex.\r\n
'''

Popen(f'{tru} -t "{control_title}" -n "{control_body}"', shell=True, stderr=None, stdout=PIPE).stdout.read().decode()
menu_output = Popen(f'{tru}', shell=True, stderr=None, stdout=PIPE).stdout.read().decode()
note_output = Popen(f'{tru} -f 2', shell=True, stderr=None, stdout=PIPE).stdout.read().decode()
assert control_body in note_output
assert '🤣Foobar Barbaz 🥷 Bazbez Lorem ipsum dolor s' in menu_output
print('✅ -n -t test passed')
print(menu_output)








