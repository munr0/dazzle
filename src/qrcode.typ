#import "@local/zebra:0.2.0" as zebra // TODO: switch to @preview before release

// Largest safe graphic per error correction level and version, from tools/safe-sizes.py
#let _sizes = toml("safe-sizes.toml")

/// A QR code with a graphic in the middle.
///
/// The modules under the graphic are removed, and its size is worked out from what the error
/// correction can recover. Without a graphic this is zebra's `qrcode`.
///
/// - data (str, bytes): The data to encode.
/// - graphic (none, content): Content to put in the middle, such as an `image`. Images are scaled
///   to fit the square, keeping their aspect ratio.
/// - graphic-size (auto, int, ratio): Side of the square for the graphic. `auto` is the largest
///   size that still decodes. An `int` is a number of modules and a `ratio` is a fraction of the
///   code's width, not counting the quiet zone. Both are rounded down to an odd number of modules
///   so the square is centred, and aren't checked against the error correction, so a size larger
///   than `auto` may not scan.
/// - options (dictionary): zebra's QR code options. `ec-level` defaults to `"h"` with a graphic.
/// - quiet-zone (int, bool, none): As in zebra.
/// - clear (dictionary, array): As in zebra. With a graphic, these rectangles are cleared as well
///   as the graphic's square, but they aren't accounted for in its size, so they may stop it
///   scanning.
/// - args (any): Passed on to zebra's `qrcode`: `width`, `height`, `module-size`, `fill` and
///   `background-fill`.
/// -> content
#let qrcode(
  data,
  graphic: none,
  graphic-size: auto,
  options: (:),
  quiet-zone: 0,
  clear: (),
  ..args,
) = {
  assert(args.pos() == (), message: "dazzle: unexpected positional arguments")
  if graphic == none {
    return zebra.qrcode(
      data,
      options: options,
      quiet-zone: quiet-zone,
      clear: clear,
      ..args,
    )
  }
  assert(
    options.at("versions", default: none) != "micro",
    message: "dazzle: graphics are not supported in Micro QR codes",
  )
  let options = (ec-level: "h") + options

  context {
    // A render at 1pt per module measures the symbol in modules (zebra also validates the options)
    let probe = zebra.qrcode(data, options: options, module-size: 1pt)
    let modules = calc.round(measure(probe).width / 1pt)
    // zebra couldn't encode it and has reported why, such as data that's too long
    if modules == 0 { return probe }
    let side = if graphic-size == auto {
      let version = calc.quo(modules - 17, 4)
      // zebra drops `none` options, falling back to its default level M
      let level = if options.ec-level == none { "m" } else { options.ec-level }
      let side = _sizes.at(level).at(version - 1)
      assert(
        side > 0,
        message: "dazzle: no graphic fits a version "
          + str(version)
          + " QR code at error correction level "
          + upper(level)
          + "; use a higher level",
      )
      side
    } else {
      assert(
        type(graphic-size) in (int, ratio),
        message: "dazzle: graphic-size must be auto, an int or a ratio",
      )
      let side = if type(graphic-size) == ratio {
        calc.floor(float(graphic-size) * modules)
      } else { graphic-size }
      if calc.even(side) { side -= 1 }
      assert(
        side > 0,
        message: "dazzle: graphic-size is too small for even one module",
      )
      assert(
        side < modules,
        message: "dazzle: graphic-size must be smaller than the code's "
          + str(modules)
          + " modules",
      )
      side
    }

    let clear = if type(clear) == dictionary { (clear,) } else { clear }
    let code = zebra.qrcode(
      data,
      options: options,
      quiet-zone: quiet-zone,
      clear: clear + ((width: side, height: side),),
      ..args,
    )
    let size = measure(code)
    // zebra's reading of quiet-zone, in modules
    let margin = if quiet-zone == true {
      4
    } else if quiet-zone in (false, none) {
      0
    } else {
      calc.max(quiet-zone, 0)
    }
    let module-size = size.width / (modules + 2 * margin)
    block(width: size.width, height: size.height, {
      code
      place(center + horizon, box(
        width: side * module-size,
        height: side * module-size,
        {
          set image(width: 100%, height: 100%, fit: "contain")
          align(center + horizon, graphic)
        },
      ))
    })
  }
}
