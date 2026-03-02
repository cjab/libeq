# PFS Archive Format

## Structural Overview
```
                           ╔═════════════════════════╗
                  ┌────────║       Header            ║
                  │        ╠═════════════════════════╣
                  │        ║   ┌──── File ──────────┐║<─┐
                  │        ║   │  Block 1  (≤8KB)   │║  │
                  │        ║   │--------------------│║  │
                  │        ║   │  Block 2  (≤8KB)   │║  │ index_entry
                  │        ║ B │--------------------│║  │  .data_offset
                  │        ║ l │       ...          │║  │
                  │        ║ o │--------------------│║  │
                  │        ║ c │  Block N  (≤8KB)   │║  │
                  │        ║ k └────────────────────┘║  │
                  │        ║ s        ...            ║  │
                  │        ║          ...            ║  │
                  │        ║   ┌──── File ──────────┐║<─┼──┐
                  │        ║   │  Directory Blocks  │║  │  │
                  │        ║   └────────────────────┘║  │  │
  header          └───────>╠═════════════════════════╣  │  │
    .index_offset          ║      Entry Count        ║  │  │
                           ║─────────────────────────║  │  │
                           ║   ┌────────────────────┐║  │  │
                           ║   │  Index Entry 1     │║──┘  │
                           ║ I │--------------------│║     │
                           ║ n │  Index Entry 2     │║     │
                           ║ d │--------------------│║     │
                           ║ e │       ...          │║     │ directory_entry
                           ║ x │--------------------│║     │  .data_offset
                           ║   │  Directory Entry   │║─────┘
                           ║   └────────────────────┘║
                           ╠═════════════════════════╣
                           ║       Footer            ║
                           ╚═════════════════════════╝
```


PFS (also known as .s3d, .eqg, .pfs) is an archive file format used by the
EverQuest client to store zlib compressed game assets.

These files were likely only ever written by asset pipelines and written
from scratch every time. So little attention was paid in the design to random
modification of existing files.

## Data

### Header (12 bytes)
```
┌────────┬───────────────────┬──────────┬─────────┐
│ Offset │ Field             │ Type     │ Size    │
├────────┼───────────────────┼──────────┼─────────┤
│ 0x00   │ index_offset      │ u32 LE   │ 4 bytes │
│ 0x04   │ magic_number      │ u32 LE   │ 4 bytes │
│ 0x08   │ version           │ u32 LE   │ 4 bytes │
└────────┴───────────────────┴──────────┴─────────┘
```

The `index_offset` field points to the beginning of the index section
of the file. The file format was designed in such a way to make appending
data easy. Random writes were not prioritized at all. This is the one field
that does require a seek, however. It must be updated after all file blocks
have been written to point at the beginning of the index.

The `magic_number` field always contains `PFS `. That's the string PFS
followed by a space. And this is why the format is called PFS.

The `version` field always contains `0x00020000`. Or at least I haven't seen
other versions myself in the wild.

### Entry Count (4 bytes)
```
┌────────┬───────────────────┬──────────┬─────────┐
│ Offset │ Field             │ Type     │ Size    │
├────────┼───────────────────┼──────────┼─────────┤
│ 0x00   │ entry_count       │ u32 LE   │ 4 bytes │
└────────┴───────────────────┴──────────┴─────────┘
```

`entry_count` is a simple count of the number of index entries that follow in
the index section. This count includes the directory entry.

### Index Entry (12 bytes)
```
┌────────┬───────────────────┬──────────┬─────────┐
│ Offset │ Field             │ Type     │ Size    │
├────────┼───────────────────┼──────────┼─────────┤
│ 0x00   │ filename_crc      │ u32 LE   │ 4 bytes │
│ 0x04   │ data_offset       │ u32 LE   │ 4 bytes │
│ 0x08   │ uncompressed_size │ u32 LE   │ 4 bytes │
└────────┴───────────────────┴──────────┴─────────┘
```

`filename_crc` is computed using CRC-32/MPEG-2 (polynomial 0x04C11DB7, initial
value 0x00000000, no input/output reflection, no final XOR). The input is
the filename as bytes with a null terminator appended.

The filename crc is used as the key in the index to locate the file data.
When looking up a file the CRC is taken of the filename. Then the index entries
are scanned for the CRC and when a match is found the `data_offset` is used to
jump to the first block of the file in the blocks section. This is an offset
from the beginning of the file.

CRC collisions are possible and I'm unaware of any attempt to mitigate them.
It's likely, if they happened, earlier filenames with the matching CRC would
have been overwritten in the index orphaning the original file data. If this
happened it would have been immediately obvious and a new filename was probably
chosen.

`uncompressed_size` is the size of the file _after_ it has been
decompressed. This means that blocks are read until the sum of
the `uncompressed_size` entries in their headers equal this value.

### Block (8 + N bytes)
```
┌────────┬───────────────────┬──────────┬───────────┐
│ Offset │ Field             │ Type     │ Size      │
├────────┼───────────────────┼──────────┼───────────┤
│ 0x00   │ compressed_size   │ u32 LE   │ 4 bytes   │
│ 0x04   │ uncompressed_size │ u32 LE   │ 4 bytes   │
│ 0x08   │ compressed_data   │ [u8]     │ N bytes * │
└────────┴───────────────────┴──────────┴───────────┘
* N = compressed_size
```

Each file in the archive is stored as a contiguous series of blocks.
Each block contains a small header containing the compressed and
uncompressed sizes of the block's data followed by the zlib compressed
data. The maximum size of an **uncompressed** block is 8KB. In practice
this means that the compressed data should be smaller or at least
not much bigger than that (in the catastrophic case).

`uncompressed_size` is the size of the block _after_ it has been
decompressed. This means that blocks are read until the sum of
the `uncompressed_size` entries in their headers equal the
uncompressed size listed in the file's index entry.

### Directory (variable length)
```
┌────────┬───────────────────┬──────────┬───────────┐
│ Offset │ Field             │ Type     │ Size      │
├────────┼───────────────────┼──────────┼───────────┤
│ 0x00   │ file_count        │ u32 LE   │ 4 bytes   │
├────────┼───────────────────┼──────────┼───────────┤
│ 0x04   │ filename_0_len    │ u32 LE   │ 4 bytes   │
│ 0x08   │ filename_0        │ [u8]     │ N bytes * │
├────────┼───────────────────┼──────────┼───────────┤
│  ...   │ filename_M_len    │ u32 LE   │ 4 bytes   │
│  ...   │ filename_M        │ [u8]     │ N bytes * │
└────────┴───────────────────┴──────────┴───────────┘
* N = filename_len (includes null terminator)
  Repeats file_count times.
```

The directory is stored as a series of compressed blocks in the same
way that any file is. Because it must be written after all other
files have been written to the archive the directory appears last
in the block section of the file.

The directory's index entry is given a special CRC: `0x61580ac9`.
It is unknown what the filename is that results in the CRC or if there is one
at all. It is then sorted by CRC before being written into the index section,
just like all other files. So this means that while the directory's data will
always be last in the blocks section the index may not be last in the index
section.

Once decompressed the contents of the directory are displayed in the
table above. The `file_count` field contains the number of filenames
in the directory. The filenames themselves are then stored as length
followed by the string and then terminated by a null character. It's
important to note that the length _includes_ this null character.
These filename entries are packed one after another with no padding.

The filename strings themselves were likely required to be ASCII.
I haven't come across other encodings in the wild.

### Footer (9 bytes, optional)
```
┌────────┬───────────────────┬──────────┬─────────┐
│ Offset │ Field             │ Type     │ Size    │
├────────┼───────────────────┼──────────┼─────────┤
│ 0x00   │ footer_string     │ [u8; 5]  │ 5 bytes │
│ 0x05   │ timestamp         │ u32 BE   │ 4 bytes │
└────────┴───────────────────┴──────────┴─────────┘
```

The footer is optional. Some original files have it, some don't. The
`footer_string` seems to always be `STEVE`. Or at least I have never
seen other examples.

The `timestamp` field is a UNIX timestamp. Interestingly, it is the
only field in the format to be stored in big endian byte order. It appears
to be the time at which the PFS file was created. Many of the original files
have timestamps near the original release date.
