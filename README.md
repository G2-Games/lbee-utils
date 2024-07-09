# lbee-utils
A small collection of utilities for exporting and importing assets from games
made with LUCA System by [Prototype Ltd](https://www.prot.co.jp/).

## Acknowledgments
The implementation for decompression of CZ1, CZ3, and CZ4 was derived from
[GARbro](https://github.com/morkt/GARbro/). The implementation of compresssion
and decompression of CZ2, and compression of CZ1, CZ3, and CZ4 was derived from
the implementation in [LuckSystem](https://github.com/wetor/LuckSystem).
This project would not have been possible without their amazing work.

## Features
These decoders and encoders are structured as libraries first and tools second. It's possible to use them as a base to build other applications.

### CZ Images
Completely accurate CZ# file decoding and encoding. Read more about that here:

https://g2games.dev/blog/2024/06/28/the-cz-image-formats/

### PAK Archives
Partial implementation of PAK files, enough to extract data from most I've encountered, and replace data as long as decoding is successful. Any extra metadata can't be changed as of yet, however.