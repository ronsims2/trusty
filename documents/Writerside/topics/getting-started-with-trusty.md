
# Get started with tRusty 🦀📝

![tRusty logo](../../images/trusty-logo.png){width=350}
A lightweight open source CLI notes app written in Rust.

## Build from source

[tRusty Source Code](https://github.com/ronsims2/trusty)

## Download a prebuilt binary

[Windows x86_64](https://orangemantis.net/trusty/downloads/trusty-1.0.5-win64.zip)

[MacOS Apple Silicon](https://orangemantis.net/trusty/downloads/trusty-1.0.5-macos.zip)

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

## Setup

When you first set up tRusty, it will create a `.trusty` directory in your home folder.
This is where configurations and your data are stored.

The CLI will also ask you to set a password.  This is used to encrypt notes.
The process will generate recovery code 🛟 that you can use set a new password if you forget yours.

> Save this recovery code in a safe place or else your protected notes will be lost forever!
{style="warning"}

## Usage

### Add a note

Add a note with a title.

```Shell
tru -t 'Foobar' -n "Who doesn't take notes in the terminal?🤷🏾"
```

Add a quick note with a derived title.

```Shell
tru -q "The coolest kids use the terminal for everything."
```

Pipe a note into tRusty.

```Shell
echo "Sometimes you need to save the output of a program | tru -i"
```

```Shell
echo "Sometimes you need to save the output of a program with a title | tru -i -t "Saved Output $(date)"
```

Save a requests response.

```Shell
http -b https://dog.ceo/api/breeds/list/all | tru -i -t 'Dog Breed JSON'
```


### View notes

List a summary of all your notes.

```Shell
tru
```

or 

```Shell
tru -l
```

Get a specific note using its ID:

```Shell
tru -f 10
```

### Search notes

Use the full power of the command line to filter note titles.

```Shell
tru | grep -i untitled
```

Open the first note that matches a search.

```Shell
tru | grep -i untitled | trusty -g
```

Dump all your notes and search.

```Shell
tru --dump | grep -i foobar
```

Use _more_ or _less_ to page through menu results.

```Shell
tru | less
```

### Edit notes

Edit the body of the last note created, read or edited.

```Shell
tru -e
```

Edit the title and content of the last note touched.

```Shell
tru -e -A
```

Edit a note using the noted ID listed in the menu.

```Shell
tru -o 2
```

Similarly, open a text editor to change the title and body of a note.

```Shell
tru -o 2 -A
```

Open a new blank note in a text editor like _vi_ or _nano_.

```Shell
tru -o
```

### Encrypt notes

You can create an encrypted note by adding the encrypted flag to any note creation command.

```Shell
tru -o -E
```

```Shell
tru -t 'A new note' -n 'Some text for a note here.' -E
```

Encrypt an existing note.

```Shell
tru -p 2
```

### Decrypt notes

> When you try to open an encrypted note it will prompt for your password.
{style="note"}

Decrypted and save a plain text note.

```Shell
tru --unprotect 4
```

### Remove notes

Soft delete (trash) an unprotected note by ID.

```Shell
tru --trash 2
```

Permanently delete an unprotected note by ID.

```Shell
tru --delete 2
```

Hard delete a protected note by ID. ☢️☢️☢️

```Shell
tru --force-delete 2
```

Hard delete all notes in the trash.

```Shell
tru --clean
```

Untrash a note by ID.

```Shell
tru --restore 2
```

### Backing up your notes

You can back up your notes by copying and saving your tRusty database wherever you like.

```Shell
cp ~/.trusty/trusty.db ~/trusty.db.bak
```


> If you change your password, make sure you retain the recovery code(s) that match the snapshot(s) you have saved.
> Otherwise, you man not have access to your encrypted notes.
{style="warning"}

## Configuration

Specify a custom home directory by setting the `TRUSTY_HOME` environment variable.

### Release notes

* Version 0.11.0 - Updated the size of the title column to accommodate encrypted messages.
* Version 0.12.0 - Stable encryption support added.
* version 1.0.3 - Public tRusty release.

## Archive

### Older versions

FYI: The app's code name was cRusty thus the binary was called `crusty`.
These versions are stable but do not support encryption.

[Windows x86_64](https://orangemantis.net/trusty/downloads/crusty-0.8.2-win64.zip)

[MacOS Apple Silicon](https://orangemantis.net/trusty/downloads/crusty-0.8.2-macos.zip)

[MacOS Apple Intel](https://orangemantis.net/trusty/downloads/crusty-0.8.2-macos-intel.zip)

[Ubuntu x86_64](https://orangemantis.net/trusty/downloads/crusty-0.8.2-ubuntu.zip)
