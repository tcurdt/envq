# envq

A jq/yq-like CLI tool to query and manipulate .env files

## Installation

### homebrew

### nix

### cargo

```bash
cargo build --release
```

The binary will be at `target/release/envq`

## Usage

### List all keys

```bash
envq list .env
cat .env | envq list
```

### Get operations

```bash
# Get a key's value
envq get KEY .env
envq get key KEY .env  # explicit syntax

envq get comment KEY .env

# Get the header (comments before first key)
envq get header .env
```

### Set operations

```bash
envq set KEY value .env
envq set key KEY value .env  # explicit syntax

envq set comment KEY "comment" .env

envq set header "header" .env
```

### Delete operations

```bash
# delete a key (also removes its comment)
envq del KEY .env
envq del key KEY .env  # explicit syntax

# delete only the comment (preserves key and value)
envq del comment KEY .env

# delete the header
envq del header .env
```

### Stdin/Stdout mode

When no file is specified, envq reads from stdin and writes to stdout:

```bash
# direct piping
cat .env | envq set KEY value

# encrypted file workflow
decrypt .env.encrypted | envq set KEY value | encrypt > .env.encrypted.new
mv .env.encrypted.new .env.encrypted
```

## Env File Format

### Headers

Headers are all comments before the first key:

```bash
# This is the header.
# It can have multiple lines.

KEY=value
```

### Comments

Comments appear after values on the same line:

```bash
KEY=value # this is a comment
```

### Setting a key preserves its comment

```bash
$ cat .env
KEY=old # important comment
$ envq set KEY new .env
$ cat .env
KEY=new # important comment
```

### Deleting a key removes its comment

```bash
$ cat .env
KEY=value # comment
$ envq del KEY .env
$ cat .env
# empty
```
