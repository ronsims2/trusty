# tRusty 🦀📝
A lightweight CLI notes app written in Rust.

## Download prebuilt binary
[Windows x86_64](https://rsims2.com/downloads/crusty-0.8.2-win64.zip)

[MacOS Apple Silicon](https://rsims2.com/downloads/crusty-0.8.2-macos.zip)

[MacOS Apple Intel](https://rsims2.com/downloads/crusty-0.8.2-macos-intel.zip)

[Ubuntu x86_64](https://rsims2.com/downloads/crusty-0.8.2-ubuntu.zip)

This simple tool allows people (and scripts) to take notes from a terminal.

If you are like me, your desktop is cluttered with random text files that contain all sorts of program output. 
For me, using other note apps has always been painful for working with things like machine-readable text.
This app is designed to give you just enough organization to make your command line kung fu a bit easier.

tRusty is meant to be a catalog that allows you to use other CLI tools to create, edit, search and parse your notes.

tRusty is pretty good for:

* Taking quick notes
* Bookmarking URLs
* Saving responses from curl/httpie
* Jotting down program output and error messages
* Building a glossary of hard to remember CLI incantations
* Cataloging small base64 data (files/images)
* Drafting haikus

## getting started

When you first set up tRusty, it will create a `.trusty` directory in your home folder.
This is where configurations and your data are stored.

The CLI will also ask you to set a password.  This is used to encrypt notes.
This process will generate recovery code 🛟 that you can use set a new password if you forget yours.

⚠️ Save this recovery code in a safe place or else you protected notes will be lost forever!

## Usage

### Add a note

Add a note with a title.

`tru -t 'Foobar' -n "Who doesn't take notes in the terminal?🤷🏾"`

Add a quick note with a derived title.

`tru -q "The coolest kids use the terminal for everything."`

Pipe a note into tRusty.

`echo "Sometimes you need to save the output of a program | tru -i"`

```
echo "Sometimes you need to save the output of a program with a title | tru -i -t "Saved Output $(date)"
```

Save a requests response.

`http -b https://dog.ceo/api/breeds/list/all | tru -i -t 'Dog Breed JSON'`


### View notes

List a summary of all your notes.

`tru` or `tru -l`

Get a specific note using its ID:

`tru -f 10`

### Search notes

Use the full power of the command line to filter note titles.

`tru | grep -i untitled`

Open the first note that matches a search.

`tru | grep -i untitled | trusty -g`

Dump all your notes and search:

`tru --dump | grep -i foobar`

Use more or less to page through menu results.

`tru | less`

### Edit notes

Edit the body of the last note created, read or edited.

`tru -e`

Edit the title and content of the last note touched.

`tru -e -A`

Edit a note using the noted ID listed in the menu.

`tru -o 2`

Similarly, open an editor to change the title and body of a note.

`tru -o 2 -A`

Open a new blank note in an editor.

`tru -o`

### Encrypt notes

You can create an encrypted note by adding the encrypted flag to any note creation command.

`tru -o -E`

`tru -t 'A new note' -n 'Some text for a note here.' -E`

Encrypt an existing note.

`tru -p 2`

### Decrypt notes

When you try to open an encrypted note it will prompt for your password.

Decrypted and save a plain text note.

`tru --unprotect 4`

### Remove notes

Soft delete (trash) an unprotected note by ID.

`tru --trash 2`

Permanently delete an unprotected note by ID.

`tru --delete 2`

Hard delete a protected note by ID.☢️☢️☢️ 

`tru --force-delete 2`

Hard delete all notes in the trash.

`tru --clean`

Untrash a note by ID.

`tru --restore 2`

### Backing up your notes

You can back up your notes by copying and saving your crusty database wherever you like.

`cp ~/.trusty/trusty.db ~/trusty.db.bak`

⚠️ If you change you password, make sure you retain the recovery code(s) that matches your the snapshot(s) you have saved.
Otherwise, you man not have access to your encrypted notes.

### Release notes

* Version 0.11.0 - Updated the size of the title column to accommodate encrypted messages.
* Version 0.12.0 - Stable encryption support added.
* version 1.0.3 - Public tRusty release.
