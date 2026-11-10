#import "/src/lib.typ": qrcode

#set page(width: auto, height: auto, margin: 6pt)

#let m = 3pt

#let github = rgb("#181717")
#let typst-brand-primary = rgb("#239dad")
#let typst-text-brand = rgb("#11566c")
#let linkedin = rgb("#0a66c2")

#let dots = tiling(size: (m, m), relative: "parent", circle(
  radius: m / 2,
  fill: github,
))
#let fade = gradient.linear(typst-brand-primary, typst-text-brand, angle: 45deg)
#let stripes = (
  gradient
    .linear(linkedin, linkedin.darken(20%), angle: 45deg)
    .sharp(2)
    .repeat(18)
)
#let sweep = gradient.conic(..color.map.rainbow)

#grid(
  columns: 2,
  gutter: 12pt,
  qrcode(
    "https://github.com/munr0/dazzle#readme",
    graphic: pad(m, image("logos/github.svg")),
    module-size: m,
    fill: dots,
  ),
  qrcode(
    "https://typst.app/universe/package/dazzle",
    graphic: pad(m, image("logos/typst.svg")),
    module-size: m,
    fill: fade,
  ),

  qrcode(
    "https://www.linkedin.com/company/typst",
    graphic: pad(m, image("logos/linkedin.svg")),
    module-size: m,
    fill: stripes,
  ),
  qrcode(
    "https://typst.app/docs/reference/math/",
    graphic: pad(m, circle(
      stroke: 1pt,
      text(13pt, $ bold(f(x)) $),
    )),
    module-size: m,
    fill: sweep,
  ),
)
