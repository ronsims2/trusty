# crusty
The code name for CLI notes app written in Rust.

## Usage

### Add a Note

Add a note with a title.

`crusty -t 'Foobar' -n "Who doesn't take notes in the terminal?🤷🏾"`

Add a quick note without a title.

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