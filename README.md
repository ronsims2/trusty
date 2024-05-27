# cRusty
A lightweight open-sourced CLI notes app written in Rust.

This simple tool allows people (and scripts) to take notes from a terminal.

If you are like me, you desktop is cluttered with random text files that contain all sorts of program output. 
For me, using other notes apps has always painful for working with things that are machine-readable text.
This app is designed to give you just enough organization to make your command line Kung Fu a bit easier.

cRusty is meant to be your catalog of notes that allows you to use other CLI tools to create, edit, search and parse your notes.

Its pretty good for:

* Taking quick notes
* Bookmarking URLs
* Saving JSON responses from curl/httpie
* Jotting down program output and error messages
* Building a glossary of hard to remember CLI incantations
* Cataloging small base64 data files/images
* Drafting haikus

## Usage

### Add a Note

Add a note with a title.

`crusty -t 'Foobar' -n "Who doesn't take notes in the terminal?🤷🏾"`

Add a quick note without with an derived title.

`crusty -q "The coolest kids use the terminal for everything."`

Pipe a note into crusty.

`echo "Sometimes you need to save the output of a program | crusty -i"`

```
echo "Sometimes you need to save the output of a program with a title | crusty -i -t "Saved Output $(date)"
```

### View Notes

List all your notes.

`crusty -m` or just `crusty`

Get a specific note using its ID:

`crusty -r 10`

### Search Notes

Use the full power of the command line to filter notes.

`crusty | grep -i untitled`

Use more or less to page through menu results.

`crusty | less`

### Edit Notes

Edit the body of the last note created, read or edited.

`crusty -e`

Edit the title and content of the last note touched.

`crusty -e -A`

Edit a note using the noted ID listed in the menu.

`crusty -o 2`

Open a blank note in an editor.

`crusty -o`