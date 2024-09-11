#let ver = version(0, 4, 0)
#set document(
    title: "The CZx Image Formats - " + str(ver),
    author: "G2",
)
#set text(
    font: "Roboto",
    lang: "en",
    size: 9pt,
)
#set page(
    numbering: "1",
    margin: 1.5cm,
    paper: "a4",
)
#set par(leading: 0.7em)
#set block(spacing: 1.7em)

// Styling
#show link: underline
#show link: set text(blue)

#text(size: 22pt, weight: "bold", font: "Roboto Slab")[The CZx Image Formats]
#v(1.5em, weak: true)
#text(size: 1.1em)[Specification #strong[Version #ver] — Sept. 11, 2024]

#line(length: 100%, stroke: 1.5pt + gray)

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED",  "MAY", and "OPTIONAL" in this document are to be
interpreted as described in IETF
#link("https://datatracker.ietf.org/doc/html/rfc2119")[RFC2119].

The CZx family of image formats (CZ0, CZ1, CZ2, CZ3, CZ4, and CZ5) are used in
the LUCA System visual novel engine developed by
#link("https://www.prot.co.jp/")[Prototype Ltd]\. These image formats can be
used for storing lossless compressed and uncompressed pixel data over a wide
range of bit depths and with accompanying metadata useful for a visual novel.
All bytes in CZx files MUST be stored in little-endian format.

#show heading: set text(1.2em)
#show heading.where(level: 1): head => [
    #set text(18pt, font: "Roboto Slab", weight: "bold")
    #head
    #v(0.3em)
]
#show heading.where(level: 2): set text(weight: 600)
#show raw: it => [
    #box(stroke: 1pt, width: 100%, inset: 5pt, radius: 3pt)[
        #it
    ]
]

= Header

#columns(2)[

== Block 1 — Basic Header
All CZx files MUST begin with this header block. The header contains information
about basic parameters of the image, such as the bitmap dimensions. The common
part of the header is as follows:

```rust
CommonHeader {
    magic: [char; 4],  // magic bytes, ex. “CZ0\0”
    header_length: u8, // header length in bytes

    width: u16,        // image width in pixels
    height: u16,       // image height in pixels
    bit_depth: u16,    // bit depth (BPP)
    unknown: u8,       // unknown purpose, often 3
}
```

This common header MAY be followed by an extended header, which contains
metadata such as cropping parameters and positioning on the screen. This part of
the header exists if the header_length value is greater than 15 bytes, with the
exception of the CZ2 format. An example of the extended header is as follows:

```rust
ExtendedHeader {
    unknown: [u8; 5],   // Set to 0

    crop_width: u16,    // width of image crop
    crop_height: u16,   // height of image crop

    bounds_width: u16,  // width of crop bounds
    bounds_height: u16, // height of crop bounds
}
```

The extended header MAY be followed by image offset information, used for
positioning the image. This information only exists if the header_length value
is greater than 28 bytes. An example of the offset header is as follows:

```rust
OffsetHeader {
    offset_width: u16,
    offset_height: u16,

    unknown: [u8; 4],
}
```

== Block 2 — Indexed Color Information
If the depth of the image is 8 bits per pixel, the header MUST be followed by a
palette block containing the information required to properly decode an image
which is encoded in indexed color.

The palette is an ordered list of colors. The color in the first position MUST
correspond to an index value of 0 in the image, the second corresponding to a
value of 1, and so on. These colors are stored in 8 bit RGBA format.

The length of the palette corresponds to the bit depth of the image. Therefore,
the color list MUST be 256 colors long.

```rust
Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

ColorPalette {
    colors: [Color; 256]
}
```

== Block 3 — Compression Information <compression-info>
All CZx formats except for CZ0 MUST have a block immediately following the color
information which contains information about the size of chunks in the following
compressed image data. The compression block starts with the number of
compressed blocks, followed by a list of the sizes of the compressed data and
original data.

```rust
ChunkInfo {
	compressed_size: u32, // compressed size, bytes
	original_size: u32,	 // original size, bytes
}

CompressionInfo {
	chunk_number: u32,	 // the number of chunks
	chunks: ChunkInfo,
}
```
]

#pagebreak()

= File Types

#columns(2)[

== CZ0
CZ0 files are uncompressed, storing raw RGBA pixel data in a linear bitmap.

This format is most often used to store character sprites, UI elements, and
various other game assets. Use of CZ0 has decreased in more recent LUCA System
games.

The encoding used in these files is a simple bitmap of RGBA pixels. Decoding CZ0
is as simple as reading the header to determine the width and height of the
image in pixels, and then reading the image data as 4 byte RGBA chunks, which
correspond directly to pixels.

== CZ1
CZ1 files are compressed, storing raw RGBA pixel data using LZW compression.

This format is used to store text bitmaps in older LUCA System games, along with
UI elements and other small image assets in more recent games. It is most often
encountered with 8 bit indexed color, but 32 bit RGBA is also relatively common.

== CZ2
CZ2 files are compressed, storing raw RGBA pixel data using LZW compression.
This method of compression is different from CZ1.

This format is primarily used for storing text bitmaps in newer LUCA System
games. Its use outside of text bitmaps is limited.

#colbreak()

== CZ3
CZ3 files are compressed, storing modified RGBA pixel data using LZW
compression. This compression scheme is the same as CZ1.

This format is primarily used for storing backgrounds, but is also used for
sprites, character graphics, and general files. It appears to be the most
general form of more highly compressed CZx files. The compression ratios
achieved by CZ3 are similar to or slightly worse than a
PNG file with a compression level of 5.

== CZ4
CZ4 files are compressed, storing modified RGBA pixel data using LZW
compression. This compression scheme is the same as CZ1.

This format only appears in newer LUCA System games, and is primarily used for
storing sprites, character graphics, and backgrounds. It seems to have replaced
the use of CZ3 files and CZ0 files in many places in the engine, but not
entirely. The compression ratios achieved by CZ4 are similar to or slightly
better than a PNG file with a compression level of 9.

== CZ5
Little is known about the CZ5 format, as it has not been encountered in any
released games so far. The only information about it has come from decompiling
recent games which use the LUCA System engine, where it is referenced as part of
the decoder for CZx files.

]

#v(2em)
#line(length: 100%, stroke: 1.5pt + gray)

= Compression Methods
The two types of compression used in CZx files are Type 1 (used in CZ1, CZ3, and
CZ4 files) and Type 2 (used in CZ2 files). On top of these two types, CZ3 and
CZ4 have extra modifications to the image data to make it possible to compress
them further. Both of these methods are dictionary based compression algorithms.

== Type 1 (CZ1-style)
Type 1 compression is a dictionary based compression algorithm that has a
fixed-length block size. The data MUST be read and written in blocks which are
sized according to the compressed_size value in the compression information
section of the header. When creating compressed data, the block size (which
determines the compressed_size value) SHOULD be set to 0xFEFD, however, it MAY
be set to a smaller value, and MUST NOT be set to a larger value, as this will
break compatibility with existing decoders.

To decode Type 1 compression,

== Type 2 (CZ2-style)
Type 2 compression is a dictionary based compression algorithm that has a
variable-length block size. The data MUST be read in blocks which are sized
according to the compressed_size value in the
#link(<compression-info>)[compression information] section of the header. When
creating compressed data, the block size is dynamic based on the number of
entries that can fit in the 18-bit dictionary.
