<a href="https://github.com/open-flash/open-flash">
    <img src="https://raw.githubusercontent.com/open-flash/open-flash/master/logo.png"
    alt="Open Flash logo" title="Open Flash" align="right" width="64" height="64" />
</a>

# `ofl` - Open Flash CLI Application for SWF files

[![crates.io](https://img.shields.io/crates/v/ofl.svg)](https://crates.io/crates/ofl)
[![GitHub repository](https://img.shields.io/badge/Github-open--flash%2Fofl-blue.svg)](https://github.com/open-flash/ofl)
[![Build status](https://img.shields.io/travis/com/open-flash/ofl/master.svg)](https://travis-ci.com/open-flash/ofl)

`ofl` is a command line application to process SWF files using the libraries of the Open Flash project.
The current subcommands allow to parse an SWF file, extract its content and analyze its AVM1 buffers.

This project is part of the [Open Flash][ofl] project.

## Commands

### `dump`

```
ofl dump [FLAGS] <swf> [output]
```

Extract all the data from the provided SWF file into an output directory.

This command generates the following files inside the `output` directory:
- `movie.json`: full [Movie](https://docs.rs/swf-types/0.11.0/swf_types/struct.Movie.html)
- `header.json`: [Header](https://docs.rs/swf-types/0.11.0/swf_types/struct.Header.html)
- `<tagIndex>/tag.json`: [Tag](https://docs.rs/swf-types/0.11.0/swf_types/enum.Tag.html)

For tags containing AVM1 buffers (such as `DoAction`), it also generates the following files:
- `main.avm1`: AVM1 buffer
- `main.cfg.json`: Parsed [Control Flow Graph](https://docs.rs/avm1-types/0.10.0/avm1_types/cfg/struct.Cfg.html)

This is the recommended command to quickly analyze a SWF file.

### `parse`

```
ofl parse movie.swf
```

Parses an SWF file and into an [`swf-types` Movie][swf-types] and prints it as JSON.

Example output:

```
{
  "header": {
    "swf_version": 34,
    "frame_size": {
      "x_min": 0,
      "x_max": 11000,
      "y_min": 0,
      "y_max": 8000
    },
    "frame_rate": 6144,
    "frame_count": 1
  },
  "tags": [
    {
      "type": "file-attributes",
      "use_network": false,
      "use_relative_urls": false,
      "no_cross_domain_caching": false,
      "use_as3": true,
      "has_metadata": false,
      "use_gpu": false,
      "use_direct_blit": false
    },
    {
      "type": "set-background-color",
      "color": {
        "r": 255,
        "g": 255,
        "b": 255
      }
    },
    {
      "type": "define-scene-and-frame-label-data",
      "scenes": [
        {
          "offset": 0,
          "name": "Scene 1"
        }
      ],
      "labels": []
    },
    {
      "type": "show-frame"
    }
  ]
}
```

# License

[AGPL 3.0 or later](./LICENSE.md)

[ofl]: https://github.com/open-flash/open-flash
[swf-types]: https://github.com/open-flash/swf-types
