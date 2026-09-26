const center = [-89.62, 20.975];
const form = document.querySelector("#submit-form");
const button = document.querySelector("#get-location");
const status = document.querySelector("#location-status");
const mapWrap = document.querySelector(".submit-map-wrap");
const mapToggle = document.querySelector("#submit-map-toggle");
const urlInput = document.querySelector("#video-url");
const titleInput = document.querySelector("#video-title");
const preview = document.querySelector("#embed-preview");
const POI_LAYERS = ["mv-poi-mall", "mv-poi-major", "mv-poi-minor"];
// Screen-space tolerance for snapping a tap to a nearby landmark label.
const SNAP_RADIUS = 24;
const map = new maplibregl.Map({
  container: "submit-map",
  style: mvMap.styleFor(),
  center,
  zoom: 14,
  attributionControl: false,
});
map.addControl(new maplibregl.AttributionControl({ compact: true }));
mvMap.setup(map);
const pin = document.createElement("div");
pin.className = "submit-pin";
pin.textContent = "✳";
const marker = new maplibregl.Marker({ element: pin, draggable: true })
  .setLngLat(center)
  .addTo(map);

function updatePin(lngLat, place = "") {
  form.elements.lng.value = lngLat.lng.toFixed(6);
  form.elements.lat.value = lngLat.lat.toFixed(6);
  const coords = `${lngLat.lat.toFixed(4)}, ${lngLat.lng.toFixed(4)}`;
  status.textContent = place
    ? `${button.dataset.pinned} ${place} · ${coords}`
    : `${button.dataset.pinned} ${coords} · ${button.dataset.adjust}`;
}

// Snapping to a landmark gives everyone the same coordinates for the same
// place. Taps near a label snap to it; taps in open space stay exact.
function landmarkAt(point) {
  const layers = POI_LAYERS.filter((id) => map.getLayer(id));
  if (!layers.length) return null;
  let nearest = null;
  let nearestDistance = Infinity;
  for (const feature of map.queryRenderedFeatures({ layers })) {
    if (!(feature.geometry?.type === "Point" && feature.properties?.name)) continue;
    const projected = map.project(feature.geometry.coordinates);
    const distance = Math.hypot(projected.x - point.x, projected.y - point.y);
    if (distance < nearestDistance) {
      nearestDistance = distance;
      nearest = feature;
    }
  }
  return nearestDistance <= SNAP_RADIUS ? nearest : null;
}

function renderPreview(data) {
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

async function loadEmbed() {
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
    renderPreview(data);
  } catch {
    preview.textContent = preview.dataset.unavailable;
  }
}

let embedTimer;
urlInput.addEventListener("input", () => {
  clearTimeout(embedTimer);
  embedTimer = setTimeout(loadEmbed, 600);
});
urlInput.addEventListener("change", loadEmbed);

function setExpanded(expanded) {
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
  setExpanded(!mapWrap.classList.contains("is-expanded"));
});
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && mapWrap.classList.contains("is-expanded")) {
    setExpanded(false);
    mapToggle.focus();
  }
});

marker.on("dragend", () => updatePin(marker.getLngLat()));
map.on("click", (event) => {
  const landmark = landmarkAt(event.point);
  if (landmark) {
    const [lng, lat] = landmark.geometry.coordinates;
    const ll = { lng, lat };
    marker.setLngLat(ll);
    updatePin(ll, landmark.properties.name);
    return;
  }
  marker.setLngLat(event.lngLat);
  updatePin(event.lngLat);
});
button.onclick = () => {
  if (!navigator.geolocation) return;
  button.textContent = button.dataset.finding;
  navigator.geolocation.getCurrentPosition(
    (position) => {
      const ll = { lng: position.coords.longitude, lat: position.coords.latitude };
      map.flyTo({ center: [ll.lng, ll.lat], zoom: 16 });
      marker.setLngLat(ll);
      updatePin(ll);
      button.textContent = button.dataset.set;
    },
    () => {
      button.textContent = button.dataset.retry;
      status.textContent = button.dataset.unavailable;
    },
    { enableHighAccuracy: true, timeout: 10000 },
  );
};
