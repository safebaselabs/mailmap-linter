# mailmap-linter

Small, easy to use, easy to install tool to lint your git mailmap.

## What it checks

- `.mailmap` file exists
- `.mailmap` is sorted (LC_ALL=C)
- All git authors are mapped
- Author format: `Name FamilyName <email@provider.com>`

## Exclusions

You can add a `.mailmap-exclude` file or use the `--exclude` (or `-e`) command line option to specify regular expressions to ignore certain authors from being linted, for example:

```bash
'^.* <.*noreply@github.com>$'
```

## Installation

### Run directly (no install)

```bash
nix run github:kamadorueda/mailmap-linter
```

### Install to profile

```bash
nix profile install github:kamadorueda/mailmap-linter
```

### Legacy installation

```bash
nix-env -i -f https://github.com/kamadorueda/mailmap-linter/archive/master.tar.gz
```

## Usage

Run from the root of the repository you want to lint:

```bash
$ mailmap-linter
```

Exclude with regex:

```bash
$ mailmap-linter --exclude '^.* <.*noreply@github.com>$'
```

## CI Example: GitHub Actions

```yaml
- name: Install Nix
  uses: DeterminateSystems/nix-installer-action@main

- name: Lint mailmap
  run: nix run github:kamadorueda/mailmap-linter
```

## Example Output

```bash
$ mailmap-linter
[INFO] Verifying if .mailmap exists
[INFO] Computing contributors
[INFO] Reading current .mailmap
[INFO] Found .mailmap-exclude file
  [INFO] Reading current .mailmap-exclude
[INFO] Verifying .mailmap format
[INFO] Verifying that every author is in the .mailmap
  [INFO] Verifying: GitHub <noreply@github.com>
  [INFO] Excluded: GitHub <noreply@github.com>
  [INFO] Verifying: Kevin Amado <kamadorueda@gmail.com>
  [INFO] Verifying: Robin Quintero <rohaquinlop301@gmail.com>
  [INFO] Verifying: Timothy DeHerrera <tim.deherrera@iohk.io>
[INFO] Verifying that .mailmap is sorted
  [INFO] OK
```

## Development

```bash
nix develop
cargo build
cargo test
```
