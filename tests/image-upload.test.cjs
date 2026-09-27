const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const vm = require("node:vm");

function setup(file, fetch) {
  const callbacks = {};
  const status = { textContent: "" };
  const urlInput = { value: "" };
  const fileInput = { files: file ? [file] : [], addEventListener(event, callback) { callbacks[event] = callback; } };
  const form = {
    dataset: { imageUpload: "artist", uploading: "Uploading", uploadError: "Upload failed" },
    querySelector(selector) {
      if (selector === 'input[type="file"]') return fileInput;
      if (selector === 'input[name="image_url"]') return urlInput;
      return status;
    },
    addEventListener(event, callback) { callbacks[event] = callback; },
    requestSubmit(submitter) { this.submitted = true; this.submitter = submitter; },
  };
  class FormData { append(name, value) { this.name = name; this.value = value; } }
  vm.runInNewContext(fs.readFileSync(path.join(__dirname, "../public/image-upload.js"), "utf8"), {
    document: { querySelectorAll: () => [form] }, FormData, fetch,
  });
  return { form, fileInput, status, urlInput, callbacks };
}

test("a selected picture is uploaded before the record form submits", async () => {
  const file = { size: 100, type: "image/png" };
  const ui = setup(file, async (url, options) => {
    assert.equal(url, "/images/artist");
    assert.equal(options.body.name, "picture");
    assert.equal(options.body.value, file);
    return { ok: true, json: async () => ({ url: "/images/artist/test.png" }) };
  });
  const submitter = { disabled: false };
  let prevented = false;
  await ui.callbacks.submit({ submitter, preventDefault() { prevented = true; } });
  assert.equal(prevented, true);
  assert.equal(ui.urlInput.value, "/images/artist/test.png");
  assert.equal(ui.form.submitted, true);
  assert.equal(ui.form.submitter, submitter);
  assert.equal(submitter.disabled, false);
});

test("invalid pictures never reach storage or submit the form", async () => {
  const ui = setup({ size: 2 * 1024 * 1024 + 1, type: "image/png" }, async () => { throw Error("unexpected upload"); });
  await ui.callbacks.submit({ preventDefault() {} });
  assert.equal(ui.status.textContent, "Upload failed");
  assert.equal(ui.form.submitted, undefined);
});

test("an already rejected event form does not upload an image", async () => {
  const ui = setup({ size: 100, type: "image/png" }, async () => { throw Error("unexpected upload"); });
  await ui.callbacks.submit({ defaultPrevented: true });
  assert.equal(ui.form.submitted, undefined);
});
