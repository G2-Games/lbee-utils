#set document(
    title: "The CZ# Image Formats",
    author: "G2",
    date: auto,
)
#set text(font: "Roboto", lang: "en", size: 9.3pt)
#show link: underline
#show link: set text(blue)
#set page(
    numbering: "1",
    margin: 0.5in,
    paper: "ansi-a",
)

#text(size: 2.2em, weight: "bold")[The CZx Image Formats]
#v(1em, weak: true)
#text(size: 1.1em)[Specification #strong[Version 0.3] — Sept. , 2024]

#line(length: 100%, stroke: 1.5pt + gray)

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED",  "MAY", and "OPTIONAL" in this document are to be
interpreted as described in IETF #link("https://datatracker.ietf.org/doc/html/rfc2119")[RFC2119].

The CZx family of image formats (CZ0, CZ1, CZ2, CZ3, CZ4, and CZ5) are used in
the LUCA System visual novel engine developed by Prototype Ltd. These image
formats can be used for storing lossless compressed and uncompressed pixel data
over a wide range of bit depths and with accompanying metadata useful for a
visual novel. All bytes in CZx files MUST be stored in little-endian format.

= Header
#columns(2)[
== Block 1 — Basic Header
All CZx files MUST begin with this header block. The header contains information about basic parameters of the image, such as the bitmap dimensions. The common part of the header is as follows:

#box(stroke: 1pt, width: 100%, inset: 5pt, radius: 3pt)[
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
]

This common header MAY be followed by an extended header, which contains metadata such as cropping parameters and positioning on the screen. This part of the header exists if the header_length value is greater than 15 bytes, with the exception of the CZ2 format. An example of the extended header is as follows:

#box(stroke: 1pt, width: 100%, inset: 5pt, radius: 3pt)[
```rust
ExtendedHeader {
    unknown: [u8; 5],   // Set to 0

    crop_width: u16,    // width of image crop
    crop_height: u16,   // height of image crop

    bounds_width: u16,  // width of crop bounds
    bounds_height: u16, // height of crop bounds
}
```
]

The extended header MAY be followed by image offset information, used for positioning the image. This information only exists if the header_length value is greater than 28 bytes. An example of the offset header is as follows:


== Block 2 — Indexed Color Information
If the depth of the image is 8 bits per pixel, the header MUST be followed by a palette block containing the information required to properly decode an image which is encoded in indexed color.
The palette is an ordered list of colors. The color in the first position MUST correspond to an index value of 0 in the image, the second corresponding to a value of 1, and so on. These colors are stored in 8 bit RGBA format.

The length of the palette corresponds to the bit depth of the image. Therefore, the color list MUST be 256 colors long.

#box(stroke: 1pt, width: 100%, inset: 5pt, radius: 3pt)[
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
]

]
