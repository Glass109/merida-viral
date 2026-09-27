document.querySelectorAll("form[data-image-upload]").forEach((form) => {
  const fileInput = form.querySelector('input[type="file"]');
  const urlInput = form.querySelector('input[name="image_url"]');
  const status = form.querySelector(".metro-upload-status");
  let uploaded = false;

  fileInput.addEventListener("change", () => {
    uploaded = false;
    urlInput.value = "";
    status.textContent = "";
  });

  form.addEventListener("submit", async (event) => {
    if (event.defaultPrevented) return;
    if (uploaded || !fileInput.files.length) return;
    event.preventDefault();
    const file = fileInput.files[0];
    if (file.size > 2 * 1024 * 1024 || !["image/jpeg", "image/png", "image/webp"].includes(file.type)) {
      status.textContent = form.dataset.uploadError;
      return;
    }
    const submitter = event.submitter;
    if (submitter) submitter.disabled = true;
    status.textContent = form.dataset.uploading;
    try {
      const data = new FormData();
      data.append("picture", file);
      const response = await fetch(`/images/${form.dataset.imageUpload}`, { method: "POST", body: data });
      if (!response.ok) throw new Error("upload failed");
      urlInput.value = (await response.json()).url;
      uploaded = true;
      if (submitter) submitter.disabled = false;
      form.requestSubmit(submitter || undefined);
    } catch {
      status.textContent = form.dataset.uploadError;
    } finally {
      if (submitter) submitter.disabled = false;
    }
  });
});
