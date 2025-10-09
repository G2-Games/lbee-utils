<p align="center">
    <img width="80%" src="https://github.com/user-attachments/assets/6807854b-aa4b-431d-933f-9e5b63ff5ed3">
</p>


# Lbee-Utils
A small collection of utilities for exporting and importing assets from games
made with LUCA System by [Prototype Ltd](https://www.prot.co.jp/).

Tested on the following games:
 - Little Busters! English Edition (2017)
 - LOOPERS (2023)
 - Butterfly's Poison; Blood Chains (Steam) (2023)
 - Harmonia Full HD Edition (2024)
 - Kanon (2024)
 - planetarian \~Snow Globe~ (Nintendo Switch) (2024)

Please test on your own games and open an issue if something isn't working.

## Disclaimer
This utility is experimental. It is based on reverse-engineering efforts by myself 
and others, and makes no guarantees about stability, performance, or utility. I 
will attempt to fix issues as they arise, but anything that takes an unacceptable 
amount of time unfortunately has to be deprioritized. I would gratefully accept 
any contributions which fix pain points of this project!

## Acknowledgments
The implementation of compresssion and decompression of CZ1, CZ2, CZ3, and CZ4 
was derived from [LuckSystem](https://github.com/wetor/LuckSystem). The 
implementation for decompression of CZ1, CZ3, and CZ4 was originally derived from 
[GARbro](https://github.com/morkt/GARbro/), but no longer is. 

This project would not have been possible without the work of these previous 
projects!

## Licensing
The libraries are licensed under the 
[MIT License](https://choosealicense.com/licenses/mit/) which allows for easy
integration into existing projects. The applications and programs are licensed
under the [GPLv3](https://choosealicense.com/licenses/gpl-3.0/).

The logo is licensed under 
[CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/) which means 
you are allowed to do whatever you want with it so long as you do not use it
for commercial gain (NC), and continue using the same license (SA).

Please read the licences before deciding how to use this project in your own, 
thank you!

## Features
These decoders and encoders are structured as libraries first and tools second.
It's possible to use them as a base to build other applications.

### CZ Images
[Completely accurate](https://g2games.dev/blog/2024/06/28/the-cz-image-formats/) 
CZ# (CZ0, CZ1, CZ2, CZ3, CZ4) file decoding and encoding. If you encounter a CZ5,
please provide it to me!

### PAK Archives
Partial implementation of PAK files, enough to extract data from most I've
encountered, and replace data as long as decoding is successful. Any extra
metadata can't be changed as of yet, however. Contributions to the PAK
decoding such as testing on many games and providing test files is *greatly*
appreciated.

## Programs

### [lbee-utils](https://github.com/G2-Games/lbee-utils/releases/tag/utils-0.1.1)
Small command line tools for modifying CZ images and PAK archives.

To install with Cargo:
```
cargo install --git https://github.com/G2-Games/lbee-utils lbee-utils
```

Otherwise, download the binaries from the Releases page here.

------

### [PAK Explorer](https://github.com/G2-Games/lbee-utils/releases/tag/explorer-0.1.2)
This is a basic explorer application for PAK files which allows you to see
their contents, replace the contents, extract files, and save them again.

While this is a useful tool for just viewing and extracting the contents of 
a PAK file, it is recommended to use the command line tools for doing 
anything important as  they offer many more options and allow for batch 
operations on many files at once.

![image](https://github.com/user-attachments/assets/0ae93c40-a951-45a7-b5ee-17b60aa96157)


To install with Cargo:

```
cargo install --git https://github.com/G2-Games/lbee-utils pak_explorer
```

Otherwise, download the binaries from the Releases page here.
