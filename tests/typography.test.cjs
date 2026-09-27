const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const css = fs.readFileSync(path.join(__dirname, '../public/app.css'), 'utf8');
const layout = fs.readFileSync(path.join(__dirname, '../src/ui/layout.rs'), 'utf8');

test('site loads the chosen fonts and assigns them by role', () => {
  for (const family of ['Space+Grotesk', 'Bebas+Neue', 'Permanent+Marker', 'Archivo+Black']) {
    assert.ok(layout.includes(family), `Google Fonts request should include ${family}`);
  }
  assert.match(css, /--type:\s*"Space Grotesk"/);
  assert.match(css, /--type-display:\s*"Bebas Neue"/);
  assert.match(css, /--type-handwritten:\s*"Permanent Marker"/);
  assert.match(css, /--type-heavy:\s*"Archivo Black"/);
  assert.match(css, /\.metro-title,[\s\S]*?\.detail-copy h1\s*\{[^}]*font-family:\s*var\(--type-display\)/);
  assert.match(css, /\.handwritten-annotation\s*\{[^}]*font-family:\s*var\(--type-handwritten\)/);
  assert.match(css, /\.button-dark,[\s\S]*?\.event-locate\s*\{[^}]*font-family:\s*var\(--type-heavy\)/);
});
