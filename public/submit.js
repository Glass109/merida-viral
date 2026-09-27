const center = mvMap.CENTER;
const form = document.querySelector("#submit-form");
const button = document.querySelector("#get-location");
const status = document.querySelector("#location-status");
const mapWrap = document.querySelector(".submit-map-wrap");
const mapToggle = document.querySelector("#submit-map-toggle");
const urlInput = document.querySelector("#video-url");
const titleInput = document.querySelector("#video-title");
const preview = document.querySelector("#embed-preview");
const map = mvMap.createMap("submit-map");
const pin = document.createElement("div");
pin.className = "submit-pin";
pin.textContent = "✳";
const marker = mvMap.addMarker(map, center, pin, { draggable: true });

function updateSelectedLocationDisplay(lngLat, place = "") {
  form.elements.lng.value = lngLat.lng.toFixed(6);
  form.elements.lat.value = lngLat.lat.toFixed(6);
  const coords = `${lngLat.lat.toFixed(4)}, ${lngLat.lng.toFixed(4)}`;
  status.textContent = place
    ? `${button.dataset.pinned} ${place} · ${coords}`
    : `${button.dataset.pinned} ${coords} · ${button.dataset.adjust}`;
}

function renderVideoEmbedPreview(data) {
  preview.replaceChildren();
  if (data.thumbnail) {
    const image = document.createElement("img");
    image.src = data.thumbnail;
    image.alt = "";
    image.loading = "lazy";
    image.referrerPolicy = "no-referrer";
    preview.append(image);
  }
  const byline = [data.creator, data.platform].filter(Boolean).join(" · ");
  if (byline) {
    const text = document.createElement("p");
    text.className = "embed-byline";
    text.textContent = byline;
    preview.append(text);
  }
  if (!preview.hasChildNodes()) preview.textContent = preview.dataset.unavailable;
}

async function loadVideoEmbedPreview() {
  const url = urlInput.value.trim();
  if (!url) {
    preview.replaceChildren();
    return;
  }
  if (!/^https:\/\//i.test(url)) {
    preview.textContent = preview.dataset.unavailable;
    return;
  }
  preview.textContent = preview.dataset.reading;
  try {
    const response = await fetch("/embed", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ url }),
    });
    if (!response.ok) throw new Error("embed lookup failed");
    const data = await response.json();
    if (!titleInput.value.trim() && data.title) {
      titleInput.value = [...data.title].slice(0, 90).join("");
    }
    renderVideoEmbedPreview(data);
  } catch {
    preview.textContent = preview.dataset.unavailable;
  }
}

let embedTimer;
urlInput.addEventListener("input", () => {
  clearTimeout(embedTimer);
  embedTimer = setTimeout(loadVideoEmbedPreview, 600);
});
urlInput.addEventListener("change", loadVideoEmbedPreview);

function setSubmissionMapExpanded(expanded) {
  mapWrap.classList.toggle("is-expanded", expanded);
  document.body.classList.toggle("map-expanded", expanded);
  mapToggle.setAttribute("aria-expanded", String(expanded));
  mapToggle.querySelector(".map-expand-label").textContent = expanded
    ? mapToggle.dataset.collapse
    : mapToggle.dataset.expand;
  mapToggle.querySelector(".map-expand-icon").textContent = expanded ? "⤡" : "⤢";
  requestAnimationFrame(() => map.resize());
}

mapToggle.addEventListener("click", () => {
  setSubmissionMapExpanded(!mapWrap.classList.contains("is-expanded"));
});
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && mapWrap.classList.contains("is-expanded")) {
    setSubmissionMapExpanded(false);
    mapToggle.focus();
  }
});

marker.on("dragend", () => updateSelectedLocationDisplay(marker.getLngLat()));
map.on("click", (event) => {
  const landmark = mvMap.nearestLandmark(map, event.point);
  if (landmark) {
    const [lng, lat] = landmark.geometry.coordinates;
    const ll = { lng, lat };
    marker.setLngLat(ll);
    updateSelectedLocationDisplay(ll, landmark.properties.name);
    return;
  }
  marker.setLngLat(event.lngLat);
  updateSelectedLocationDisplay(event.lngLat);
});
button.onclick = () => {
  if (!navigator.geolocation) return;
  button.textContent = button.dataset.finding;
  navigator.geolocation.getCurrentPosition(
    (position) => {
      const ll = { lng: position.coords.longitude, lat: position.coords.latitude };
      map.flyTo({ center: [ll.lng, ll.lat], zoom: 16 });
      marker.setLngLat(ll);
      updateSelectedLocationDisplay(ll);
      button.textContent = button.dataset.set;
    },
    () => {
      button.textContent = button.dataset.retry;
      status.textContent = button.dataset.unavailable;
    },
    { enableHighAccuracy: true, timeout: 10000 },
  );
};
