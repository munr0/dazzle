#import "@local/zebra:0.2.0" as zebra

#set page(width: auto, height: auto, margin: 2pt)

// zebra's code with a square of `side` cleared, and a red square placed over it by offset
#let expected(
  data,
  side,
  modules,
  ec-level: "h",
  quiet-zone: 0,
  module-size: 2pt,
  width: auto,
  clear: (),
  fill: red,
) = {
  let module-size = if width == auto { module-size } else {
    width / (modules + 2 * quiet-zone)
  }
  let offset = (quiet-zone + (modules - side) / 2) * module-size
  block({
    zebra.qrcode(
      data,
      options: (ec-level: ec-level),
      quiet-zone: quiet-zone,
      module-size: module-size,
      clear: clear + ((width: side, height: side),),
    )
    place(top + left, dx: offset, dy: offset, rect(
      width: side * module-size,
      height: side * module-size,
      fill: fill,
    ))
  })
}

#expected("https://typst.app", 9, 29)
#expected("https://typst.app", 9, 29, quiet-zone: 4)
#expected("https://typst.app", 9, 29, quiet-zone: 2, width: 60pt)
#expected("https://typst.app", 5, 29)
#expected("https://typst.app", 7, 29)
#expected("https://typst.app/universe/package/dazzle", 9, 33, ec-level: "q")
#expected(
  "https://typst.app",
  9,
  29,
  clear: ((width: 3, height: 3, x: 20, y: 20),),
  fill: red.transparentize(50%),
)
#expected(
  "https://typst.app",
  9,
  29,
  clear: (
    (width: 2, height: 2, x: 9, y: 20),
    (width: 2, height: 2, x: 20, y: 9),
  ),
  fill: red.transparentize(50%),
)
