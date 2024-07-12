# lbee-utils
A small collection of utilities for exporting and importing assets from games
made with LUCA System by [Prototype Ltd](https://www.prot.co.jp/).

Tested on the following games:
 - Little Busters! English Edition (2017)
 - LOOPERS (2023)
 - Harmonia Full HD Edition (2024)
 - Kanon (2024)

## Acknowledgments
The implementation for decompression of CZ1, CZ3, and CZ4 was originally
derived from [GARbro](https://github.com/morkt/GARbro/). The implementation of
compresssion and decompression of CZ1, CZ2, CZ3, and CZ4 was derived from
[LuckSystem](https://github.com/wetor/LuckSystem). This project would not have
been possible without their amazing work.

## Features
These decoders and encoders are structured as libraries first and tools second.
It's possible to use them as a base to build other applications.

### CZ Images
Completely accurate CZ# file decoding and encoding. Read more about that here:

https://g2games.dev/blog/2024/06/28/the-cz-image-formats/

### PAK Archives
Partial implementation of PAK files, enough to extract data from most I've
encountered, and replace data as long as decoding is successful. Any extra
metadata can't be changed as of yet, however.

## Programs

### [lbee-utils](https://github.com/G2-Games/lbee-utils/releases/tag/utils-0.1.0)
Small command line tools for modifying CZ images and PAK archives. Usage for each
is as follows:

#### pakutil
```
Utility to maniuplate PAK archive files from the LUCA System game engine by Prototype Ltd

Usage: pakutil <PAK FILE> <COMMAND>

Commands:
  extract  Extracts the contents of a PAK file into a folder
  replace  Replace the entries in a PAK file
  help     Print this message or the help of the given subcommand(s)

Arguments:
  <PAK FILE>  

Options:
  -h, --help     Print help
  -V, --version  Print version
```

#### czutil
```
Utility to maniuplate CZ image files from the LUCA System game engine by Prototype Ltd

Usage: czutil <COMMAND>

Commands:
  decode   Converts a CZ file to a PNG
  replace  Replace a CZ file's image data
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
------

### [PAK Explorer](https://github.com/G2-Games/lbee-utils/releases/tag/explorer-0.1.1)
This is a basic explorer application for PAK files which allows you to see
their contents, replace the contents, extract files, and save them again.

While this is a useful tool for just viewing and extracting the contents of 
a PAK file, it is recommended to use the command line tools for doing 
anything important as  they offer many more options and allow for batch 
operations on many files at once.

![image](https://github.com/user-attachments/assets/0ae93c40-a951-45a7-b5ee-17b60aa96157)
