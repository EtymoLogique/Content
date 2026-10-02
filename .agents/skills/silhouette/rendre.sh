#!/bin/zsh
# Planche de contrôle des silhouettes, en une rangée.
# Usage : rendre.sh sortie.png nom [nom…]   (noms sans .svg, lus dans medias/silhouettes/)
# Bandes rouges : parties rognées sur la mini-carte mobile (y < 41 et y > 359).
# Cadre rouge : zone du mot (x 0–190, y 0–90). Requiert inkscape et magick.
set -e
out=${1:?sortie.png}; shift
D=${0:A:h}/../../../medias/silhouettes
T=$(mktemp -d)
pngs=()
for n in "$@"; do
  xmllint --noout $D/$n.svg
  sed -e 's/currentColor/#333/g' -e 's/var(--art-bg)/#f1e6cc/g' \
      -e 's#</svg>$#<rect x="-50" y="-50" width="400" height="91" fill="\#c0392b" fill-opacity="0.25"/><rect x="-50" y="359" width="400" height="100" fill="\#c0392b" fill-opacity="0.25"/><rect x="0" y="0" width="190" height="90" fill="none" stroke="red" stroke-width="1"/></svg>#' \
      $D/$n.svg > $T/$n.svg
  inkscape $T/$n.svg --export-type=png --export-filename=$T/$n.png --export-width=300 \
      --export-background=#f1e6cc --export-background-opacity=1 >/dev/null 2>&1
  pngs+=($T/$n.png)
done
magick montage -label '%t' $pngs -tile x1 -geometry +6+6 $out
rm -rf $T
echo $out
