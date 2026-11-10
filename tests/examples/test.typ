#import "/src/lib.typ": qrcode

#set page(width: auto, height: auto, margin: 4pt)

#let url = "https://typst.app/universe/package/dazzle"
#let badge = circle(
  width: 100%,
  fill: gradient.linear(red, orange),
  stroke: 1pt + white,
)
#let wide = image(
  bytes(
    "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 20 10'>"
      + "<rect width='20' height='10' rx='2' fill='teal'/><circle cx='10' cy='5' r='3' fill='white'/></svg>",
  ),
  format: "svg",
)

#grid(
  columns: 3,
  gutter: 6pt,
  qrcode(url, graphic: badge, module-size: 2pt),
  qrcode(url, graphic: wide, module-size: 2pt),
  qrcode(url, graphic: text(7pt, weight: "bold")[dz], module-size: 2pt),

  qrcode(url, graphic: badge, graphic-size: 25%, fill: navy, module-size: 2pt),
  qrcode(
    url,
    graphic: [],
    background-fill: gradient.linear(aqua, fuchsia),
    quiet-zone: 2,
    module-size: 2pt,
  ),
  qrcode(url + "?" + "x" * 150, graphic: badge, module-size: 1pt),
)
