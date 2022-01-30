+++
title = "Docs"
description = "Docs"
+++

# Overview

MicroW8 loads WebAssembly modules with a maximum size of 256kb. You module needs to export
a function `fn upd()` which will be called once per frame.
After calling `upd` MicroW8 will display the 320x240 8bpp framebuffer located
at offset 120 in memory with the 32bpp palette located at 0x13000.

The memory has to be imported as `env` `memory` and has a maximum size of 256kb (4 pages).

# Memory map

```
00000-00040: user memory
00040-00044: time since module start in ms
00044-0004c: gamepad state
0004c-00078: reserved
00078-12c78: frame buffer
12c78-13000: reserved
13000-13400: palette
13400-13c00: font
13c00-14000: reserved
14000-40000: user memory
```

# API

All API functions are found in the `env` module.

## Math

These all do what you'd expect them to. All angles are in radians.

### fn asin(x: f32) -> f32

Returns the arcsine of `x`.

### fn acos(x: f32) -> f32

Returns the arccosine of `x`.

### fn atan(f32) -> f32

Returns the arctangent of `x`.

### fn atan2(y: f32, y: f32) -> f32

Returns the angle between the point `(x, y)` and the positive x-axis.

### fn sin(angle: f32) -> f32

Returns the sine of `angle`.

### fn tan(angle: f32) -> f32

Returns the tangent of `angle`.

### fn cos(angle: f32) -> f32

Returns the cosine of `angle`.

### fn exp(x: f32) -> f32

Returns `e^x`.

### fn log(x: f32) -> f32

Returns the natural logarithmus of `x`. Ie. `e^log(x) == x`.

### fn pow(x: f32, y: f32) -> f32

Returns `x^y`.

### fn fmod(x: f32, y: f32) -> f32

Returns `x` modulo `y`, ie. `x - floor(x / y) * y`. This means the sign of the result of `fmod` is the same as `y`.

## Random

MicroW8 provides a pretty good PRNG, namely xorshift64*. It is initialized to a constant seed at each startup, so if you
want to vary the random sequence you'll need to provide a seed yourself.

### fn random() -> i32

Returns a (pseudo-)random 32bit integer.

### fn randomf() -> f32

Returns a (pseudo-)random float equally distributed in `[0,1)`.

### fn randomSeed(seed: i32)

Seeds the PRNG with the given seed. The seed function is reasonably strong so that you can use

```
randomSeed(index);
random()
```

as a cheap random-access PRNG (aka noise function).

## Graphics

The default palette can be seen [here](../v0.1pre4#AgKaeeOuwg5gCKvFIeiitEwMpUI2rymEcu+DDB1vMu9uBoufvUxIr4Y5p4Jj2ukoNO4PE7QS5cN1ZyDMCRfSzYIGZxKlN2J6NKEWK7KVPk9wVUgn1Ip+hsMinWgEO8ETKfPuHoIa4kjI+ULFOMad7vd3rt/lh1Vy9w+R2MXG/7T61d3c7C6KY+eQNS0eW3ys4iU8R6SycuWZuuZ2Sg3Qxp826s+Kt+2qBojpzNOSoyFqyrVyYMTKEkSl0BZOj59Cs1hPm5bq0F1MmVhGAzMhW9V4YeAe). (Press Z on the keyboard to switch to palette.)

The palette can be changed by writing 32bit rgba colors to addresses 0x13000-0x13400.

The drawing functions are sub-pixel accurate where applicable (line, circle). Pixel centers lie halfway between integer
coordinates. Ie. the top-left pixel covers the area `0,0 - 1,1`, with `0.5,0.5` being the pixel center.

### fn cls(color: i32)

Clears the screen to the given color index. Also sets the text cursor to `0, 0` and disables graphical text mode.

### fn setPixel(x: i32, y: i32, color: i32)

Sets the pixel at `x, y` to the given color index.

### fn getPixel(x: i32, y: i32) -> i32

Returns the color index at `x, y`. Returns `0` if the given coordinates are outside the screen.

### fn hline(left: i32, right: i32, y: i32, color: i32)

Fills the horizontal line `[left, right), y` with the given color index.

### fn rectangle(x: f32, y: f32, w: f32, h: f32, color: i32)

Fills the rectangle `x,y - x+w,y+h` with the given color index.

(Sets all pixels where the pixel center lies inside the rectangle.)

### fn circle(cx: f32, cy: f32, radius: f32, color: i32)

Fills the circle at `cx, cy` and with `radius` with the given color index.

(Sets all pixels where the pixel center lies inside the circle.)

### fn rectangle_outline(x: f32, y: f32, w: f32, h: f32, color: i32)

Draws a one pixel outline on the inside of the given rectangle.

(Draws the outermost pixels that are still inside the rectangle area.)

### fn circle_outline(cx: f32, cy: f32, radius: f32, color: i32)

Draws a one pixel outline on the inside of the given circle.

(Draws the outermost pixels that are still inside the circle area.)

### fn line(x1: f32, y1: f32, x2: f32, y2: f32, color: i32)

Draws a line from `x1,y1` to `x2,y2` in the given color index.

## Input

MicroW8 provides input from a gamepad with one D-Pad and 4 buttons, or a keyboard emulation thereof.

The buttons are numbered

| Button | Keyboard    | Index |
| ------ | ----------- | ----- |
| Up     | Arrow-Up    | 0     |
| Down   | Arrow-Down  | 1     |
| Left   | Arrow-Left  | 2     |
| Right  | Arrow-Right | 3     |
| A      | Z           | 4     |
| B      | X           | 5     |
| X      | A           | 6     |
| Y      | S           | 7     |

In addition to using the API functions below, the gamepad state can also be read as a bitfield of
pressed buttons at address 0x44. 0x48 holds the buttons that were pressed last frame.

### fn isButtonPressed(btn: i32) -> i32

Returns whether the buttons with the given index is pressed this frame.

### fn isButtonTriggered(btn: i32) -> i32

Returns whether the given button is newly pressed this frame.

### fn time() -> f32

Returns the time in seconds since the start of the cart.

The integer time in milliseconds can also be read at address 0x40.

## Text output

The default font can be seen [here](../v0.1pre4#AgKaeeOuwg5gCKvFIeiitEwMpUI2rymEcu+DDB1vMu9uBoufvUxIr4Y5p4Jj2ukoNO4PE7QS5cN1ZyDMCRfSzYIGZxKlN2J6NKEWK7KVPk9wVUgn1Ip+hsMinWgEO8ETKfPuHoIa4kjI+ULFOMad7vd3rt/lh1Vy9w+R2MXG/7T61d3c7C6KY+eQNS0eW3ys4iU8R6SycuWZuuZ2Sg3Qxp826s+Kt+2qBojpzNOSoyFqyrVyYMTKEkSl0BZOj59Cs1hPm5bq0F1MmVhGAzMhW9V4YeAe).

The font can be changed by writing 1bpp 8x8 characters to addresses 0x13400-0x13c00.

All text printing is done at the cursor position, which is advanced after printing each character.
The cursor is not visible.

Text printing can operate in two modes - normal and graphics. After startup and after `cls()` normal mode is active.

### Normal mode

In normal mode, text printing is constrained to an 8x8 character grid. Setting the cursor position to `2,3` will start printing at pixel coordinates `16,24`.

When printing characters, the full 8x8 pixels are painted with the text and background colors according to the character graphics in the font.

When moving/printing past the left or right border the cursor will automatically wrap to the previous/next line. When moving/printing past the upper/lower border, the screen will be scrolled down/up 8 pixels, filling the fresh line with the background color.

### Graphics mode

In graphics mode, text can be printed to any pixel position, the cursor position is set in pixel coordinates.

When printing characters only the foreground pixels are set, the background is "transparent".

Moving/printing past any border does not cause any special operation, the cursor just goes off-screen.

### Control chars

Characters 0-31 are control characters and don't print by default. They take the next 0-2 following characters as parameters.
Avoid the reserved control chars, they are currently NOPs but their behavior can change in later MicroW8 versions.

| Code  | Parameters | Operation                            |
| ----- | ---------- | ------------------------------------ |
| 0     | -          | Nop                                  |
| 1     | char       | Print char (including control chars) |
| 2-3   | -          | Reserved                             |
| 4     | -          | Switch to normal mode                |
| 5     | -          | Switch to graphics mode              |
| 6-7   | -          | Reserved                             |
| 8     | -          | Move cursor left                     |
| 9     | -          | Move cursor right                    |
| 10    | -          | Move cursor down                     |
| 11    | -          | Move cursor up                       |
| 12    | -          | do `cls(background_color)`           |
| 13    | -          | Move cursor to the left border       |
| 14    | color      | Set the background color             |
| 15    | color      | Set the text color                   |
| 16-23 | -          | Reserved                             |
| 24    | -          | Swap text/background colors          |
| 25-30 | -          | Reserved                             |
| 31    | x, y       | Set cursor position (*)              |

(*) In graphics mode, the x coordinate is doubled when using control char 31 to be able to cover the whole screen with one byte.

### fn printChar(char: i32)

Prints the character in the lower 8 bits of `char`. If the upper 24 bits are non-zero, right-shifts `char` by 8 bits and loops back to the beginning.

### fn printString(ptr: i32)

Prints the zero-terminated string at the given memory address.

### fn printInt(num: i32)

Prints `num` as a signed decimal number.

### fn setTextColor(color: i32)

Sets the text color.

### fn setBackgroundColor(color: i32)

Sets the background color.

### fn setCursorPosition(x: i32, y: i32)

Sets the cursor position. In normal mode `x` and `y` are multiplied by 8 to get the pixel position, in graphics mode they are used as is.

# The `uw8` tool

The `uw8` tool included in the MicroW8 download includes a number of useful tools for developing MicroW8 carts. For small productions written in
wat or CurlyWas you don't need anything apart from `uw8` and a text editor of your choice.

## `uw8 run`

Usage:

`uw8 run [<options>] <file>`

Runs `<file>` which can be a binary WebAssembly module, an `.uw8` cart, a wat (WebAssembly text format) source file or a [CurlyWas](https://github.com/exoticorn/curlywas) source file.

Options:

* `-t FRAMES`, `--timeout FRAMES` : Sets the timeout in frames (1/60s). If the start or update function runs longer than this it is forcibly interupted
and execution of the cart is stopped. Defaults to 30 (0.5s)
* `-w`, `--watch`: Reloads the given file every time it changes on disk.
* `-p`, `--pack`: Pack the file into an `.uw8` cart before running it and print the resulting size.
* `-u`, `--uncompressed`: Use the uncompressed `uw8` format for packing.
* `-l LEVEL`, `--level LEVEL`: Compression level (0-9). Higher compression levels are really slow.
* `-o FILE`, `--output FILE`: Write the loaded and optionally packed cart back to disk.

## `uw8 pack`

Usage:

`uw8 pack [<options>] <infile> <outfile>`

Packs the WebAssembly module or text file, or [CurlyWas](https://github.com/exoticorn/curlywas) source file into a `.uw8` cart.

Options:

* `-u`, `--uncompressed`: Use the uncompressed `uw8` format for packing.
* `-l LEVEL`, `--level LEVEL`: Compression level (0-9). Higher compression levels are really slow.

## `uw8 filter-exports`

Usage:

`uw8 filter-exports <infile> <outfile>`

Reads a binary WebAssembly module, removes all exports not used by the MicroW8 platform + everything that is unreachable without those exports and writes the resulting module to `outfile`.

When compiling C code (or Rust, zig or others) to WebAssembly, you end up with a few exported global variables that are used for managing the heap and C stack, even if the code doesn't actually use those features. You can use this command to automatically remove them and gain a few bytes. See the C, Rust and zig examples in the MicroW8 repository.

# Other useful tools

The [Web Assembly Binary Toolkit](https://github.com/WebAssembly/wabt) includes
a few useful tools, eg. `wat2wasm` to compile the WebAssemby text format to binary
wasm and `wasm2wat` to disassemble wasm binaries.

[Binaryen](https://github.com/WebAssembly/binaryen) includes `wasm-opt` which enable additional optimizations over what LLVM (the backend that is used by most compilers that target WebAssembly) can do.

# Distribution

The classical distribution option is just to put the `.uw8` cart into a zip file, let people run it themselves, either in the `uw8` tool or in the web runtime.

If you want to go this way, you might consider including `microw8.html` in your download. It's specifically designed to be a small (~10KB at the moment), self-contained HTML file for just this reason. That way, anyone who has downloaded you production can run it, even when offline, provided they have a modern web browser at hand. Also, should future versions of MicroW8 ever introduce any kind of incompatibilities, they'd still have a compatible version right there without hunting arround for an old version.

## Base64 encoded link

For small productions (<= 1024 bytes), when you load them in the web runtime, the URL is automatically updated to include the cart as base64 encoded data. You can just give that URL to others for them to run your prod.

## url parameter

Another option is to put the cart on a webserver and add `#url=url/to/the/cart.uw8` to the end of the web runtime URL. ([Like this](../v0.1pre5#url=../uw8/skipahead.uw8))

If the cart and the web runtime are on different domains, you'll have to make sure that [CORS header](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS#the_http_response_headers) are enabled for the cart, otherwise the web runtime won't be able to load it.

Feel free to put the web runtime on your own server if it makes sense to you, its [license](https://unlicense.org/) allows you to do anything you want with it.

## `.html` + `.uw8`

At startup the web runtime will try to load a cart in the same directory as the `.html` file. If the URL of the web runtime ends in `.html` it will try to load a cart with the same name and the extension `.uw8`. If the URL of the web runtime ends in a `/` it will try to load a `cart.uw8` at that location.

So, you could for example serve the web runtime as `https://example.org/mytunnel.html` and the cart as `https://example.org/mytunnel.uw8` and send people to the HTML page to run the cart. Or you could put them up as `https://example.org/mytunnel/index.html` and `https://example.org/mytunnel/cart.uw8` and send people to `https://example.org/mytunnel`.

If a cart is found and loaded in this way, the load button is hidden.

## Itch.io

The above `.html` + `.uw8` option works great on [Itch.io](https://itch.io) as well. Put these two files into a zip archive:

* `index.html`: a copy of the web runtime (`microw8.html` in the MicroW8 download)
* `index.uw8`: Your game cart

Upload the zip file to itch.io and make sure to set the embedded viewport size to exactly (!) 640x480 pixel. At that exact size the web runtime hides everything except for the MicroW8 screen.

If instead you actually *want* to display the border around the screen and the byte size you can try a size of about 720x620.

[See here for an example upload.](https://exoticorn.itch.io/skipahead)

# `.uw8` format

The first byte of the file specifies the format version:

## Format version `00`:

This file is simply a standard WebAssembly module

## Format version `01`:

The rest of this file is the same as a WebAssembly
module with the 8 byte header removed. This module
can leave out sections which are then taken from
a base module provided by MicroW8.

You can generate this base module yourself using
`uw8-tool`. As a quick summary, it provides all function
types with up to 5 parameters (i32 or f32) where the
`f32` parameters always preceed the `i32` parameters.
Then it includes all imports that MicroW8 provides,
a function section with a single function of type
`() -> void` and an export section that exports
the first function in the file under the name `upd`.

## Format version `02`:

Same as version `01` except everything after the first byte is compressed
using a [custom LZ compression scheme](https://github.com/exoticorn/upkr).

# The web runtime

[...]

## Video recording

Press F10 to start recording, press again to stop. Then a download dialog will open for the video file.
The file might miss some metadata needed to load in some video editing tools, in that case you can run
it through ffmpeg like this `ffmpeg -i IN_NAME.webm -c copy -o OUT_NAME.webm to fix it up.

To convert it to 1280x720, for example for a lovebyte upload, you can use:

```
ffmpeg -i IN.webm -vf "scale=960:720:flags=neighbor,pad=1280:720:160:0" -r 60 OUT.mp4
```

## Screenshot

Pressing F9 opens a download dialog with a screenshot.