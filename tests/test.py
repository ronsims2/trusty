# set the TRUSTY_HOME so that we can use a test db
# Test password: 1234
# test recovery code: 7e85e714-60fd-4e8c-a58e-73674314101c
import os
from io import BytesIO
from os import environ, path, mkdir, getcwd, curdir, remove
from subprocess import Popen, PIPE, run
import tempfile
import pathlib
import shutil
import pexpect

TEST_PASSWORD = '1234'
TEST_RECOVERY_CODE = '7e85e714-60fd-4e8c-a58e-73674314101c'
TERM = 'xterm'
# set this so that we can edit
environ['TERM'] = TERM

def get_menu_output(tru):
    return Popen(f'{tru}', shell=True, stderr=None, stdout=PIPE).stdout.read().decode()


def get_note_by_id(tru, id, should_err=False):
    if not should_err:
        return Popen(f'{tru} -f {id}', shell=True, stderr=None, stdout=PIPE).stdout.read().decode()
    else:
        return Popen(f'{tru} -f {id}', shell=True, stderr=PIPE, stdout=None).stderr.read().decode()


def get_encrypted_note_by_id(tru, note_id, pwd):
    # buf = BytesIO()
    proc = pexpect.spawn(f'{tru} -f {note_id}')
    # proc.logfile = buf
    proc.expect('Enter password:')
    proc.sendline(pwd)

    return proc.read().decode()


def hard_delete_by_id(tru, id):
    Popen(f'{tru} -F {id}', shell=True, stderr=PIPE, stdout=PIPE).stdout.read().decode()
    return Popen(f'{tru} -f {id}',
                 shell=True,
                 stderr=PIPE,
                 stdout=PIPE).stderr.read().decode()


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

trusty_test_dir = path.join(getcwd(), 'tests')
# set the pseudo-editor
EDITOR = f'truncate -s 0'
environ['EDITOR'] = EDITOR
print(f'$EDITOR env var set to: {EDITOR}')

if not path.exists(trusty_config_dir):
    mkdir(trusty_config_dir)

    # clean up old database
if path.isfile(trusty_db_path):
    print('Cleaning up previous workspace🧹')
    remove(trusty_db_path)

shutil.copyfile(path.join(trusty_test_dir, 'trusty.db'), trusty_db_path)
print('Initialized database 🧑🏽‍💻')
print(f'Database location: {trusty_db_path}')

environ[trusty_home] = trusty_home_dir
# the path to the built executable
trusty = path.join(getcwd(), 'target', 'debug', 'tru')

print(f'{trusty_home} env var set to: {environ.get(trusty_home)}')

# Build debug tRusty
try:
    run('cargo build', shell=True, capture_output=True, text=True)
    print('Built tRusty app 🛠️')
except Exception as e:
    print('Could not build project')

# Test the help command
help_output = Popen(f'{trusty} --help', shell=True, stderr=None, stdout=PIPE).stdout.read().decode()
assert 'tRusty: a command line notes app  🦀📝' in help_output
print('✅ --help test passed')

# Test tRusty --list
control_default_output = 'Get Started with tRusty'
default_output = Popen(f'{trusty}', shell=True, stderr=None, stdout=PIPE).stdout.read().decode()
assert control_default_output in default_output
list_output = Popen(f'{trusty}', shell=True, stderr=None, stdout=PIPE).stdout.read().decode()
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

# Test adding a note
Popen(f'{trusty} -t "{control_title}" -n "{control_body}"', shell=True, stderr=None, stdout=PIPE).stdout.read().decode()
menu_output = get_menu_output(trusty)
note_output = get_note_by_id(trusty, 2)
assert control_body in note_output
assert '🤣Foobar Barbaz 🥷 Bazbez Lorem ipsum dolor s' in menu_output
print('✅ -n -t test passed')
test_id = 9999
# test find failure too
result = get_note_by_id(trusty, test_id, True)
assert f'Could not find note for id: {test_id}' == result.strip()
print('✅ -f test passed')

# Test adding a quick note
control_quick_note = '''🥷🤣🐶Nulla tincidunt, sem vitae luctus dignissim, 🥷 
lacus nibh consequat erat, nec tristique ipsum dui et ex.\r\n
Lorem  🤣 ipsum dolor sit amet, consectetur adipiscing elit.'''
Popen(f'{trusty} -q "{control_quick_note}"', shell=True, stderr=None, stdout=PIPE).stdout.read().decode()
menu_output = get_menu_output(trusty)
assert '🥷🤣🐶Nulla tincidunt, sem vitae luctus digni' in menu_output
print('✅ -q test passed')

# Test piping a note in
piped_note = '''🥷Nulla tincidunt, sem vitae luctus dignissim, 🥷 
lacus nibh consequat erat, 🤣🐶nec tristique ipsum dui et ex.
Lorem  🤣 ipsum dolor sit amet, consectetur adipiscing elit.🥷'''
piped_id = 4
Popen(f'echo "{piped_note}" | {trusty} -i', shell=True, stderr=None, stdout=PIPE).stdout.read().decode()
menu_output = get_menu_output(trusty)
assert 'Untitled' in menu_output
note_output = get_note_by_id(trusty, piped_id)
assert piped_note.strip() == note_output.strip()
print('✅ -i test passed')
# test pipe dnote with title
piped_title = "🥷Bar Foo 🥷"
Popen(f'echo "{piped_note}" | {trusty} -i -t "{piped_title}"',
      shell=True, stderr=None, stdout=PIPE).stdout.read().decode()
menu_output = get_menu_output(trusty)
assert piped_title in menu_output
print('✅ -i -t test passed')

# Test catting a note from stdin (pipping)
catted_note = 'I ❤️ 🌮'
Popen(f'echo "{catted_note}" | {trusty} -i --cat {piped_id}', shell=True, stderr=None, stdout=PIPE).stdout.read().decode()
note_output = get_note_by_id(trusty, piped_id)
expected_note = f'{piped_note}\n\n{catted_note}'
assert note_output.strip() == expected_note
print('✅ -i --cat test passed')

control_encrypted_title = '🔒 ENCRYPTED'
# Add an encrypted note with title
encrypted_note_title = '🥷🏽❤️🗡️'
encrypted_note_body = 'Ninjas loves swords'
child = pexpect.spawn(f'{trusty} -t "{encrypted_note_title}" -n "{encrypted_note_body}" -E')
child.expect('Enter password:')
child.sendline(TEST_PASSWORD)
menu_output = get_menu_output(trusty)
assert control_encrypted_title in menu_output
assert encrypted_note_body in get_encrypted_note_by_id(trusty,6, TEST_PASSWORD)
print('✅ -t -n -E test passed')
hard_delete_by_id(trusty, 6)

# Add an encrypted quicknote
encrypted_quick_note = '🐶🐶🐶 Foobar Dog 🐶🐶🐶'
child = pexpect.spawn(f'{trusty} -q "{encrypted_quick_note}" -E')
child.expect('Enter password:')
child.sendline(TEST_PASSWORD)
menu_output = get_menu_output(trusty)
assert control_encrypted_title in menu_output
assert encrypted_quick_note in get_encrypted_note_by_id(trusty, 7, TEST_PASSWORD)
print('✅ -q -E test passed')
hard_delete_by_id(trusty, 7)

# Add an encrypted piped note
encrypted_piped_note = '👀 peek-a-boo'
child = pexpect.spawn(f'{trusty} -i -E')
child.sendline(encrypted_piped_note)
child.sendeof()
child.expect('Enter password:')
child.sendline(TEST_PASSWORD)
menu_output = get_menu_output(trusty)
assert control_encrypted_title in menu_output
assert encrypted_piped_note in get_encrypted_note_by_id(trusty, 8, TEST_PASSWORD)
print('✅ -i -E test passed')
hard_delete_by_id(trusty, 8)

# Add an encrypted piped note with title
encrypted_piped_id = 9
encrypted_piped_title = '👀👀👀 LOOKEY HERE'
child = pexpect.spawn(f'{trusty} -i -t "{encrypted_piped_title}" -E')
child.sendline(encrypted_piped_note)
child.sendeof()
child.expect('Enter password:')
child.sendline(TEST_PASSWORD)
menu_output = get_menu_output(trusty)
assert control_encrypted_title in menu_output
assert encrypted_piped_note in get_encrypted_note_by_id(trusty, encrypted_piped_id, TEST_PASSWORD)
print('✅ -i -E test passed')


# Hard delete an encrypted note
result = hard_delete_by_id(trusty, 9)
assert 'Could not find note for id: 9' == result.strip()
print('✅ -D test passed')

# test find from grep
search_term = 'whodiwho'
search_note = f'Falafel and waffles and {search_term} and drizzle'
Popen(f'{trusty} -q "{search_note}"', shell=True, stderr=None, stdout=PIPE)
result = Popen(f'echo "       10 | 2024-07-12 03:16:27 | Falafel and waffles and whodiwho and drizzle" | {trusty} -g', shell=True, stderr=None, stdout=PIPE).stdout.read().decode().strip()
assert search_note == result
print('✅ -g test passed')

# test dump feature
result = Popen(f'{trusty} --dump', shell=True, stderr=None, stdout=PIPE).stdout.read().decode()
lines = result.strip().split('\n')
# Modify the count below if we add more text
assert (len(lines) == 35)
print('✅ --dump test passed')

# test edit
Popen(f'{trusty} -f 1', shell=True, stderr=None, stdout=PIPE)
cmd = f'{trusty} -e'
proc = Popen(cmd, shell=True, stderr=None, stdout=PIPE)
Popen(f'ps {proc.pid}', shell=True, stderr=None, stdout=PIPE).stdout.read().decode()
result = get_note_by_id(trusty, 1)
assert result.strip() == ''
print('✅ -e test passed')





