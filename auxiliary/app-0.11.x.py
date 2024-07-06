import os
import sqlite3
from os import environ, path, remove
import shutil



user_home = environ.get('HOME')

if user_home is None or user_home.strip() == '':
    print('Error: HOME not set. Set you HOME environment variable so that this script can find your tRusty database.')
    exit(1)

crusty_home = path.join(user_home, '.crusty')
crusty_db_path = path.join(crusty_home, 'crusty.db')
crusty_bak = f'crusty.db.pre0.12x.bak'
crusty_bak_path = path.join(crusty_home, crusty_bak)

if path.exists(crusty_db_path):
    print(f'cRusty database found at: {crusty_db_path}')
else:
    print(f'Error: Could not find cRusty DB at: {crusty_db_path}')
    exit(2)


shutil.copyfile(crusty_db_path, crusty_bak_path)
print(f'cRusty database backed up to: {crusty_bak_path}')
print('Attempting to update cRusty database.')

create_temp_col_sql = 'ALTER TABLE notes ADD COLUMN temp_title TEXT;'
copy_col_data_sql = 'UPDATE notes SET temp_title = CAST(title as temp_title);'
delete_old_col_sql = 'ALTER TABLE notes DROP COLUMN title;'
rename_temp_col_sql = 'ALTER TABLE notes RENAME COLUMN temp_title TO title;'

try:
    conn = sqlite3.connect(crusty_db_path)
    cursor = conn.cursor()
    cursor.execute(create_temp_col_sql)
    cursor.execute(copy_col_data_sql)
    cursor.execute(delete_old_col_sql)
    cursor.execute(rename_temp_col_sql)
    conn.commit()
    cursor.close()
    conn.close()
except Exception as err:
    shutil.copyfile(crusty_bak_path, crusty_db_path)
    print(f'Error: {err}, restoring original database')
    exit(3)

print('Cleaning up workspace 🧹')
remove(crusty_bak_path)
print('Success, cRusty 🦀📝 database updated!')


