# lbee-utils
A small collection of utilities for exporting and importing assets from Little Busters! English Edition

## Specifications and Info
<sup>Bytes are in Little Endian</sup>

Each `CZ#` file starts with a header. The first 14 (`0x0E`) bytes of the header are common to all
`CZ#` files. The data which come after that are specific to each format, although several
`CZ#` formats do share the same layout.

### Common header:
| Offset      | Ex. Values  | ASCII | Purpose                           |
|-------------|-------------|-------|-----------------------------------|
| 0x00 - 0x03 | 43 5A 30 00 | CZ0   | Magic bytes                       |
| 0x04 - 0x07 | 24 00 00 00 | 36    | Header length in bytes            |
| 0x08 - 0x09 | 58 01       | 344   | Width of the image in pixels      |
| 0x0A - 0x0B | DC 02       | 732   | Height of the image in pixels     |
| 0x0C - 0x0D | 20 00       | 32    | Bit depth of the image            |
| 0x0E        | 03          | 3     | Color block                       |

### The CZ0 and CZ3 header extra data:
| Offset      | Ex. Values  | ASCII | Purpose                           |
|-------------|-------------|-------|-----------------------------------|
| 0x0F - 0x13 | ---         | ---   | ---[Unknown]---                   |
| 0x14 - 0x15 | 58 01       | 344   | Width of image crop               |
| 0x16 - 0x17 | DC 02       | 732   | Height of image crop              |
| 0x18 - 0x19 | 00 05       | 1280  | Width of image bounding box       |
| 0x1A - 0x1B | 34 03       | 820   | Height of image bounding box      |
| 0x1C - 0x1D | 80 02       | 640   | X offset of image, optional       |
| 0x1E - 0x1F | 02 03       | 770   | Y offset of image, optional       |
| 0x20 - 0x23 | ---         | ---   | ---[Unknown]---, optional         |


### The CZ2 header extra data:
| Offset      | Ex. Values  | ASCII | Purpose                           |
|-------------|-------------|-------|-----------------------------------|
| 0x0F - 0x12 | ---         | ---   | ---[Unknown]---                   |
