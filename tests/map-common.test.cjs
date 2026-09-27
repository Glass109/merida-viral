const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const vm = require("node:vm");

function loadMapHelpers() {
  const created = [];
  class Marker {
    constructor(options) { this.options = options; created.push(this); }
    setLngLat(coordinates) { this.coordinates = coordinates; return this; }
    addTo(map) { this.map = map; return this; }
  }
  const window = { matchMedia: () => ({ matches: false, addEventListener() {} }) };
  const document = { createElement: () => ({
    setAttribute(name, value) { this[name] = value; },
    addEventListener(name, callback) { this[name] = callback; },
    replaceChildren(content) { this.content = content; },
  }) };
  const maplibregl = { Marker };
  vm.runInNewContext(
    fs.readFileSync(path.join(__dirname, "../public/map-common.js"), "utf8"),
    { window, document, maplibregl },
  );
  return { helpers: window.mvMap, created };
}

test("coordinates use MapLibre longitude-first order and reject invalid data", () => {
  const { helpers } = loadMapHelpers();
  assert.deepEqual(Array.from(helpers.coordinatesFromElement({ dataset: { lat: "20.975", lng: "-89.62" } })), [-89.62, 20.975]);
  for (const dataset of [{ lat: "91", lng: "0" }, { lat: "NaN", lng: "0" }, { lat: "0" }]) {
    assert.equal(helpers.coordinatesFromElement({ dataset }), null);
  }
});

test("numbered markers reuse rows from any place list and skip invalid locations", () => {
  const { helpers, created } = loadMapHelpers();
  const selections = [];
  function row(lat, lng, label) {
    const number = {
      getAttribute: () => label,
      addEventListener(name, callback) { this[name] = callback; },
    };
    return {
      dataset: { lat, lng },
      querySelector: (selector) => selector === ".place-number" ? number : null,
      number,
    };
  }
  const first = row("20.975", "-89.62", "Show venue");
  const invalid = row("100", "-89.62", "Bad place");
  const list = { querySelectorAll: () => [first, invalid] };
  const map = {};
  const markers = helpers.addNumberedPlaceMarkers(map, list, (selected) => selections.push(selected));
  assert.equal(markers.length, 1);
  assert.equal(created[0].map, map);
  assert.deepEqual(Array.from(created[0].coordinates), [-89.62, 20.975]);
  assert.equal(created[0].options.element["aria-label"], "Show venue");
  created[0].options.element.click();
  first.number.click();
  assert.deepEqual(selections, [first, first]);
});

test("custom marker content is cloned from a template and uses the place name", () => {
  const { helpers, created } = loadMapHelpers();
  const content = { cloneNode: () => ({ custom: true }) };
  const source = {
    dataset: { lat: "20.975", lng: "-89.62", title: "Venue name" },
    querySelector: () => ({ content }),
  };
  const pin = { replaceChildren(value) { this.content = value; } };
  assert.equal(helpers.addPlacePin({}, source, pin), created[0]);
  assert.deepEqual(pin.content, { custom: true });
  assert.equal(pin.title, "Venue name");
  assert.deepEqual(Array.from(created[0].coordinates), [-89.62, 20.975]);
});

test("detail pins can use a template next to the map without picking another row's template", () => {
  const { helpers, created } = loadMapHelpers();
  const template = { matches: () => true, content: { cloneNode: () => "detail icon" } };
  const source = {
    dataset: { lat: "20", lng: "-89", title: "Band" },
    querySelector: () => null,
    nextElementSibling: template,
  };
  const pin = { replaceChildren(value) { this.content = value; } };
  helpers.addPlacePin({}, source, pin);
  assert.equal(pin.content, "detail icon");
  assert.equal(pin.title, "Band");
  assert.equal(created.length, 1);
});
