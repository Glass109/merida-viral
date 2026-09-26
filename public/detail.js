const el = document.querySelector("#detail-map");
const center = [Number(el.dataset.lng), Number(el.dataset.lat)];
const map = new maplibregl.Map({
  container: el,
  style: mvMap.styleFor(),
  center,
  zoom: 15,
  attributionControl: false,
});
map.addControl(new maplibregl.AttributionControl({ compact: true }));
mvMap.setup(map);
const pin = document.createElement("div");
pin.className = "map-pin map-pin-hot";
const icon = document.createElement("span");
icon.className = "iconify reicon";
icon.dataset.icon = "reicon:location";
pin.append(icon);
new maplibregl.Marker({ element: pin }).setLngLat(center).addTo(map);
