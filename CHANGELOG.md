# Changelog

## 0.2.2

Fixed:
- Properly extract crate version

## 0.2.1

Fixed:
- dependency status link

## 0.2.0

Added:
- Generation of diff.rs links

Fixed:
- Verbose report output. Shows more details for added and removed crates
- Sort workspace targets

## 0.1.1

Added:
- Simple and verbose report outputs
- Changes grouping

Fixed:
- Remove double '/' in the repository name
- Disable colored output of the 'cargo info' command

## 0.1.0

Initial release.

Features:
- Generate GitHub links for the crate versions diff
- Analyze workspace dependencies for updates
- Extract crate nested dependencies from 'cargo metadata'
- Extract commit version from '.cargo_vcs_info.json' file
- Show report in simple and verbose mode
