# cRusty 🦀📝
A lightweight CLI notes app written in Rust.

## Download
[Windows x86_64](https://rsims2.com/downloads/crusty-0.8.2-win64.zip)

[MacOS Apple Silicon](https://rsims2.com/downloads/crusty-0.8.2-macos.zip)

[MacOS Apple Intel](https://rsims2.com/downloads/crusty-0.8.2-macos-intel.zip)

[Ubuntu x86_64](https://rsims2.com/downloads/crusty-0.8.2-ubuntu.zip)

This simple tool allows people (and scripts) to take notes from a terminal.

If you are like me, your desktop is cluttered with random text files that contain all sorts of program output. 
For me, using other note apps has always been painful for working with things like machine-readable text.
This app is designed to give you just enough organization to make your command line kung fu a bit easier.

cRusty is meant to be a catalog that allows you to use other CLI tools to create, edit, search and parse your notes.

Its pretty good for:

* Taking quick notes
* Bookmarking URLs
* Saving responses from curl/httpie
* Jotting down program output and error messages
* Building a glossary of hard to remember CLI incantations
* Cataloging small base64 data (files/images)
* Drafting haikus

## Usage

### Add a Note

Add a note with a title.

`crusty -t 'Foobar' -n "Who doesn't take notes in the terminal?🤷🏾"`

Add a quick note with a derived title.

`crusty -q "The coolest kids use the terminal for everything."`

Pipe a note into crusty.

`echo "Sometimes you need to save the output of a program | crusty -i"`

```
echo "Sometimes you need to save the output of a program with a title | crusty -i -t "Saved Output $(date)"
```

Save a requests response.

`http -b https://dog.ceo/api/breeds/list/all | crusty -i -t 'Dog Breed JSON'`


### View Notes

List a summary of all your notes.

`crusty` or `crusty -l`

Get a specific note using its ID:

`crusty -f 10`

### Search Notes

Use the full power of the command line to filter note titles.

`crusty | grep -i untitled`

Open the first note that matches a search.

`crusty | grep -i untitled | crusty -g`

Dump all your notes and search:

`crusty --dump | grep -i foobar`

Use more or less to page through menu results.

`crusty | less`

### Edit Notes

Edit the body of the last note created, read or edited.

`crusty -e`

Edit the title and content of the last note touched.

`crusty -e -A`

Edit a note using the noted ID listed in the menu.

`crusty -o 2`

Open a new blank note in an editor.

`crusty -o`

### Remove Notes

Soft delete (trash) an unprotected note by ID.

`crusty --trash 2`

Permanently delete an unprotected note by ID.

`crusty --delete 2`

Hard delete a protected note by ID.☢️☢️☢️ 

`crusty --force-delete 2`

Hard delete all notes in the trash.

`crusty --clean`

Untrash a note by ID.

`crusty --restore 2`
