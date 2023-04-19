# lbee-utils
A small collection of utilities for exporting and importing assets from Little Busters! English Edition

The CZ0 header:
| Offset | Ex. Values | ASCII | Purpose                           |
|--------|------------|-------|-----------------------------------|
| 0x00   | 43 5A 30   | CZ0   | Magic bytes to identify CZ0 files |
| 0x04   | 24         | 36    | Header length in bytes            |
| 0x08   | 58 01      | 344   | Width of the image in pixels      |
| 0x0A   | DC 02      | 732   | Height of the image in pixels     |
| 0x0C   | 20         | 32    | Bit depth of the image            |
| 0x14   | 58 01      | 344   | Width of image crop               |
| 0x16   | DC 02      | 732   | Height of image crop              |
| 0x18   | 00 05      | 1280  | Width of image bounding box       |
| 0x1A   | 34 03      | 820   | Height of image bounding box      |
| 0x1C   | 80 02      | 640   | X offset of image                 |
| 0x1E   | 02 03      | 770   | Y offset of image                 |

<sup>Bytes are in Little Endian order</sup>
