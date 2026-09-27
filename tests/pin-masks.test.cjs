const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const publicDir = path.join(__dirname, '../public');
const css = fs.readFileSync(path.join(publicDir, 'app.css'), 'utf8');

test('every pin shape uses an editable external SVG mask and the supplied noise texture', () => {
  for (const style of ['guitar-pick', 'skull', 'flame', 'star', 'lightning', 'coffin', 'heart', 'burst']) {
    const svg = fs.readFileSync(path.join(publicDir, 'pin-masks', `${style}.svg`), 'utf8');
    assert.match(svg, /<svg[^>]+viewBox="0 0 100 118"/);
    assert.match(svg, /<path\b[\s\S]*?fill="(?:white|#ffffff)"/i);
    assert.match(css, new RegExp(`pin-masks/${style}\\.svg`));
    assert.doesNotMatch(svg, /<script|onload=|<image|<foreignObject/i);
  }
  assert.match(css, /background: url\("\/noise\.jpg"\)/);
  assert.match(css, /-webkit-mask: var\(--pin-mask\)/);
  assert.match(css, /mask: var\(--pin-mask\)/);
  assert.match(css, /\.photo-pin::before\s*\{[^}]*background: #0a0a0a/s);
  assert.match(css, /\.photo-pin-outline\s*\{[^}]*background: #050505/s);
  assert.match(css, /\.photo-pin-rim img\s*\{[^}]*blur\(3px\) brightness\(\.38\)/s);
  assert.match(css, /\.map-photo-marker \.photo-pin\s*\{[^}]*drop-shadow\(0 0 2px/s);
  assert.match(css, /\.map-photo-marker\s*\{[^}]*width: 116px;[^}]*height: 138px/s);
  assert.doesNotMatch(css, /\.map-photo-marker::after/);
});
