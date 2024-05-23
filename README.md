# cRusty
An open-sourced CLI notes app written in Rust.

This simple tool allows people (and scripts) to take notes from a terminal.

Its pretty good for:

* Making quick notes
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

List all your notes.

`crusty -m` or just `crusty`

Get a specific note using its ID:

`crusty -r 10`

Use the full power of the command line to filter notes.

`crusty | grep -i untitled`

Edit the last note created, read or edited.

`crusty -e`

Edit a note using the noted ID listed in the menu.

`crusty -o 2`

Open a blank note in an editor.

`crusty -o`