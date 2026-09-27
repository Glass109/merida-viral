const venueMap = mvMap.createMap("venue-map", mvMap.CENTER, 14);
const latInput = document.querySelector("#venue-lat");
const lngInput = document.querySelector("#venue-lng");
let marker;

function placePin(lng, lat, updateFields = true) {
  if (!Number.isFinite(lng) || !Number.isFinite(lat) || Math.abs(lng) > 180 || Math.abs(lat) > 90) return;
  if (!marker) {
    const pin = document.createElement("div");
    pin.className = "submit-pin";
    pin.textContent = "✳";
    marker = mvMap.addMarker(venueMap, [lng, lat], pin, { draggable: true });
    marker.on("dragend", () => {
      const position = marker.getLngLat();
      placePin(position.lng, position.lat);
    });
  } else {
    marker.setLngLat([lng, lat]);
  }
  if (updateFields) {
    lngInput.value = lng.toFixed(6);
    latInput.value = lat.toFixed(6);
  }
}

venueMap.on("click", (event) => placePin(event.lngLat.lng, event.lngLat.lat));
if (latInput.value !== "" && lngInput.value !== "") {
  const lat = Number(latInput.value);
  const lng = Number(lngInput.value);
  if (Number.isFinite(lat) && Number.isFinite(lng)) {
    placePin(lng, lat, false);
    venueMap.on("load", () => venueMap.jumpTo({ center: [lng, lat], zoom: 14 }));
  }
}
for (const field of [latInput, lngInput]) {
  field.addEventListener("change", () => {
    if (latInput.value === "" || lngInput.value === "") return;
    const lat = Number(latInput.value);
    const lng = Number(lngInput.value);
    if (Number.isFinite(lat) && Number.isFinite(lng) && Math.abs(lat) <= 90 && Math.abs(lng) <= 180) {
      placePin(lng, lat, false);
      venueMap.flyTo({ center: [lng, lat], animate: !window.matchMedia("(prefers-reduced-motion: reduce)").matches });
    }
  });
}
