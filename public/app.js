const map = mvMap.createMap("map");

const mapView = document.querySelector("#map-view");
const workspace = document.querySelector(".map-workspace");
const viewToggle = document.querySelector("#view-toggle");
const sortToggle = document.querySelector("#sort-toggle");
const grid = document.querySelector("#video-grid");
const wideLayout = window.matchMedia("(min-width: 900px)");
let markers = [];
let popular = true;

function showPlacesOrMapView(view, push = false) {
  const showingMap = view === "map";
  workspace.dataset.view = view;
  viewToggle.setAttribute("aria-expanded", String(showingMap));
  viewToggle.querySelector(".bar-label").textContent = showingMap
    ? viewToggle.dataset.places
    : viewToggle.dataset.map;
  viewToggle.querySelector(".bar-icon").textContent = showingMap ? "☷" : "▤";
  if (push && !wideLayout.matches) history.pushState({ view }, "", showingMap ? "#map" : location.pathname);
  if (wideLayout.matches || showingMap) requestAnimationFrame(() => map.resize());
}

viewToggle.addEventListener("click", () => {
  showPlacesOrMapView(workspace.dataset.view === "places" ? "map" : "places", true);
});
window.addEventListener("popstate", () => showPlacesOrMapView(location.hash === "#map" ? "map" : "places"));
new ResizeObserver(() => {
  if (mapView.getBoundingClientRect().width) map.resize();
}).observe(document.querySelector("#map"));

function focusMapOnSelectedPlace(row) {
  const coordinates = mvMap.coordinatesFromElement(row);
  if (!coordinates) return;
  showPlacesOrMapView("map", !wideLayout.matches);
  map.flyTo({
    center: coordinates,
    zoom: 16,
    essential: true,
    animate: !window.matchMedia("(prefers-reduced-motion: reduce)").matches,
  });
  document.querySelectorAll(".place-row").forEach((item) => {
    item.classList.toggle("selected", item === row);
  });
}

function syncMapMarkersWithPlaceList() {
  markers.forEach((marker) => marker.remove());
  markers = mvMap.addNumberedPlaceMarkers(map, grid, focusMapOnSelectedPlace);
  document.querySelector("#video-count").textContent = String(markers.length).padStart(2, "0");
}

document.body.addEventListener("htmx:afterSwap", (event) => {
  if (event.detail.target === grid) syncMapMarkersWithPlaceList();
});
document.body.addEventListener("htmx:configRequest", (event) => {
  if (!popular && event.detail.path === "/vote") event.detail.path = "/vote?sort=newest";
});
document.body.addEventListener("htmx:responseError", (event) => {
  if (event.detail.target === grid) grid.textContent = grid.dataset.error;
});

sortToggle.addEventListener("click", () => {
  popular = !popular;
  sortToggle.dataset.sort = popular ? "trending" : "newest";
  sortToggle.querySelector(".bar-label").textContent = popular
    ? sortToggle.dataset.trending
    : sortToggle.dataset.newest;
  sortToggle.setAttribute("aria-label", popular ? sortToggle.dataset.sortTrending : sortToggle.dataset.sortNewest);
  sortToggle.setAttribute("aria-pressed", String(popular));
  htmx.ajax("GET", `/places?sort=${popular ? "popular" : "newest"}`, {
    target: "#video-grid",
    swap: "innerHTML",
  });
});

document.querySelector("#zoom-in").addEventListener("click", () => map.zoomIn());
document.querySelector("#zoom-out").addEventListener("click", () => map.zoomOut());
document.querySelector("#locate").addEventListener("click", () => {
  if (!navigator.geolocation) return;
  navigator.geolocation.getCurrentPosition(
    (position) => map.flyTo({
      center: [position.coords.longitude, position.coords.latitude],
      zoom: 15.5,
      animate: !window.matchMedia("(prefers-reduced-motion: reduce)").matches,
    }),
    () => {},
    { enableHighAccuracy: true, timeout: 9000 },
  );
});

showPlacesOrMapView(location.hash === "#map" ? "map" : "places");
map.on("load", syncMapMarkersWithPlaceList);
