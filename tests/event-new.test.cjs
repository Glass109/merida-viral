const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const vm = require("node:vm");

test("event forms require a band without forcing the first band", () => {
  const choices = Array.from({ length: 2 }, () => ({
    checked: false,
    setCustomValidity(message) { this.message = message; },
    reportValidity() { this.reported = true; },
    addEventListener(type, callback) { this[type] = callback; },
  }));
  const form = {
    dataset: { bandRequired: "Choose a band" },
    querySelectorAll: () => choices,
    addEventListener(type, callback) { this[type] = callback; },
  };
  vm.runInNewContext(fs.readFileSync(path.join(__dirname, "../public/event-new.js"), "utf8"), {
    document: { querySelector: () => form },
  });
  let prevented = false;
  form.submit({ preventDefault() { prevented = true; } });
  assert.equal(prevented, true);
  assert.equal(choices[0].message, "Choose a band");
  assert.equal(choices[0].reported, true);
  choices[1].checked = true;
  choices[1].change();
  prevented = false;
  form.submit({ preventDefault() { prevented = true; } });
  assert.equal(prevented, false);
  assert.equal(choices[0].message, "");
});
