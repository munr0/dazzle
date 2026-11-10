#import "/src/lib.typ": qrcode

#set page(width: auto, height: auto, margin: 2pt)

#let logo = rect(width: 100%, height: 100%, fill: red)

// "https://typst.app" is version 3 (29 modules); the safe size at level H is 9
#qrcode("https://typst.app", graphic: logo, module-size: 2pt)
#qrcode("https://typst.app", graphic: logo, module-size: 2pt, quiet-zone: true)
#qrcode("https://typst.app", graphic: logo, width: 60pt, quiet-zone: 2)
// Explicit sizes round down to odd: 6 -> 5, 30% of 29 = 8.7 -> 7
#qrcode("https://typst.app", graphic: logo, graphic-size: 6, module-size: 2pt)
#qrcode("https://typst.app", graphic: logo, graphic-size: 30%, module-size: 2pt)
// Version 4 (33 modules) at level Q: 9
#qrcode(
  "https://typst.app/universe/package/dazzle",
  graphic: logo,
  options: (ec-level: "q"),
  module-size: 2pt,
)
// The caller's clear is kept too
#let see-through = rect(
  width: 100%,
  height: 100%,
  fill: red.transparentize(50%),
)
#qrcode(
  "https://typst.app",
  graphic: see-through,
  module-size: 2pt,
  clear: (width: 3, height: 3, x: 20, y: 20),
)
#qrcode("https://typst.app", graphic: see-through, module-size: 2pt, clear: (
  (width: 2, height: 2, x: 9, y: 20),
  (width: 2, height: 2, x: 20, y: 9),
))
