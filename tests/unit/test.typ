#import "/src/lib.typ" as dazzle: qrcode
#import "/src/qrcode.typ": _sizes
#import "@local/zebra:0.2.0" as zebra

#assert.eq(dictionary(dazzle).keys().sorted(), ("datamatrix", "qrcode"))
#assert.eq(dazzle.datamatrix, zebra.datamatrix)

// Odd sizes, so the square is centred
#for level in ("l", "m", "q", "h") {
  let sizes = _sizes.at(level)
  assert.eq(sizes.len(), 40)
  assert(sizes.all(size => size == 0 or calc.odd(size)), message: level)
}

#context {
  let data = "https://typst.app"
  let cases = (
    (:),
    (quiet-zone: true),
    (quiet-zone: 2, width: 4cm),
    (width: 3cm),
    (height: 2cm),
    (module-size: 2pt),
  )
  for args in cases {
    let expected = measure(zebra.qrcode(data, options: (ec-level: "h"), ..args))
    assert.eq(
      measure(qrcode(data, graphic: [], ..args)),
      expected,
      message: repr(args),
    )
    assert.eq(
      measure(qrcode(data, graphic: [], graphic-size: 25%, ..args)),
      expected,
      message: repr(args),
    )
  }
}

// Without a graphic, zebra's default EC level is kept
#context assert.eq(
  measure(qrcode("https://typst.app/universe")),
  measure(zebra.qrcode("https://typst.app/universe")),
)
