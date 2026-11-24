<!-- help-ids: ppi.curveOffset, curve.convert, bezier.tangent ; template: concept ; since: 0.1.0 -->
# Curve offsets, conversions and Bezier Tangent

## Offsets

The four offset commands make an offset copy of the selected curves in the
same item, a set distance away, like a parallel line. Each opens a dialog
for the distance first.

| Button | Works on |
|---|---|
| **Open Curve Offset…** | open curves, one side |
| **Open Curve Offset Trio…** | open curves, both sides |
| **Closed Curve Offset…** | closed curves, one side (in or out) |
| **Closed Curve Offset Trio…** | closed curves, both sides |

## Conversions

**Curve to B-Spline**, **Curve to Bezier**, **Bezier to Curve**, **Bezier
to B-Spline**, **B-Spline to Bezier** and **B-Spline to Curve** turn the
selected curves into the other kind. The result goes into a new item named
*Converted*; the source stays as it was.

## Bezier Tangent

**Bezier Tangent…** sharpens, smooths or evens out the handles of the
selected Bezier knots.

## Related

- [Curve tab](index.md)
- [Bezier](../modeling/bezier.md)
