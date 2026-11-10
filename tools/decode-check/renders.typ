// Codes with graphics across versions and error correction levels, one per page, for
// `just check-renders`: every page must decode to the payload stored in its <payload> metadata.

#import "/src/lib.typ": qrcode

#set page(width: auto, height: auto, margin: 0pt, fill: white)

// Reproducible pseudo-random printable ASCII
#let payload(length, seed) = {
  let x = seed
  let chars = ()
  for _ in range(length) {
    x = calc.rem(x * 1103515245 + 12345, 2147483648)
    chars.push(str.from-unicode(32 + calc.rem(calc.quo(x, 65536), 95)))
  }
  chars.join()
}

#let graphics = (
  rect(width: 100%, height: 100%, fill: black),
  rect(width: 100%, height: 100%, fill: white),
  circle(width: 100%, fill: red),
  [],
)

// Longest payload for each level, around version 27: beyond about version 28, zebra's path
// decoding runs into Typst's loop limit
#let longest = (l: 1300, m: 1050, q: 800, h: 600)
// Version 1 at level L has no room for a graphic, so start beyond it (17 bytes)
#let lengths = (5, 20, 40, 70, 110, 160, 230, 320, 440, 600, 800, 1050, 1300)

#let codes = ()
#for level in ("l", "m", "q", "h") {
  for length in lengths.filter(n => (
    n <= longest.at(level) and (level != "l" or n > 17)
  )) {
    codes.push((level: level, length: length, size: auto))
  }
}
// Explicit sizes below the safe one
#for (length, size) in ((40, 5), (160, 15%), (320, 21), (600, 25%)) {
  codes.push((level: "h", length: length, size: size))
}

#for (i, code) in codes.enumerate() {
  let data = payload(code.length, i + 1)
  [#metadata(data) <payload>]
  qrcode(
    data,
    graphic: graphics.at(calc.rem(i, graphics.len())),
    graphic-size: code.size,
    options: (ec-level: code.level),
    quiet-zone: true,
    module-size: 4pt,
  )
  pagebreak(weak: true)
}
