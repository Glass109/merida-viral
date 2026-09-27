const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const vm = require("node:vm");

function load(reducedMotion = false) {
  const listeners = {};
  const window = {
    addEventListener: (name, handler) => { listeners[name] = handler; },
    matchMedia: () => ({ matches: reducedMotion }),
    navigation: {},
  };
  vm.runInNewContext(fs.readFileSync(path.join(__dirname, "../public/page-transitions.js"), "utf8"), { window, URL });
  function transition(from, to, navigationType = "push", fromIndex = 0, toIndex = 1, eventName = "pagereveal") {
    const activation = {
      from: { url: `https://example.com${from}`, index: fromIndex },
      entry: { url: `https://example.com${to}`, index: toIndex },
      navigationType,
    };
    const types = [];
    const viewTransition = { types: { add: (type) => types.push(type) }, skipTransition() { this.skipped = true; } };
    window.navigation.activation = activation;
    listeners[eventName]({ viewTransition, activation });
    return { types, skipped: !!viewTransition.skipped };
  }
  return transition;
}

test("collection detail entry and browser Back/Forward use opposite directions", () => {
  const transition = load();
  assert.deepEqual(transition("/venues", "/venues/2").types, ["metro-forward"]);
  assert.deepEqual(transition("/venues/2", "/venues", "traverse", 1, 0).types, ["metro-backward"]);
  assert.deepEqual(transition("/venues", "/venues/2", "traverse", 0, 1).types, ["metro-forward"]);
  assert.deepEqual(transition("/venues/2", "/venues", "push").types, ["metro-backward"]);
  assert.deepEqual(transition("/venues", "/venues/new", "push", 0, 1, "pageswap").types, ["metro-forward"]);
});

test("unrelated navigation and reduced motion do not animate", () => {
  const transition = load();
  for (const [from, to] of [["/venues", "/bands"], ["/venues/1", "/events/3"], ["/venues", "/venues"]]) {
    assert.equal(transition(from, to).skipped, true);
  }
  assert.equal(load(true)("/venues", "/venues/1").skipped, true);
});
