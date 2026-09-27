(function () {
  const STYLES = {
    light: "https://tiles.openfreemap.org/styles/positron",
    dark: "https://tiles.openfreemap.org/styles/dark",
  };

  function currentMapColorTheme() {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }

  function styleUrlForTheme(theme) {
    return STYLES[theme || currentMapColorTheme()] || STYLES.dark;
  }

  // Keep the base map in the same tonal family as the surrounding UI. The
  // streets, not a contrasting block of land, should define the street grid.
  const MAP_PALETTES = {
    light: {
      background: "#ffffff",
      residential: "#fafafa",
      building: "#f3f3f3",
      buildingOutline: "#e5e5e5",
      park: "#f3f6f3",
      water: "#eaf1f3",
      path: "#e5e5e5",
      minorRoad: "#dcdcdc",
      majorRoad: "#d3d3d3",
      roadCasing: "#c4c4c4",
      roadLabel: "#666666",
      roadHalo: "#ffffff",
    },
    dark: {
      background: "#101010",
      residential: "#161616",
      building: "#1c1c1c",
      buildingOutline: "#262626",
      park: "#1b211d",
      water: "#1a252b",
      path: "#292929",
      minorRoad: "#303030",
      majorRoad: "#383838",
      roadCasing: "#464646",
      roadLabel: "#898989",
      roadHalo: "#101010",
    },
  };

  function applyMapPalette(map) {
    const colors = MAP_PALETTES[currentMapColorTheme()];
    const paint = (id, property, value) => {
      if (map.getLayer(id)) map.setPaintProperty(id, property, value);
    };
    paint("background", "background-color", colors.background);
    paint("landuse_residential", "fill-color", colors.residential);
    paint("building", "fill-color", colors.building);
    paint("building", "fill-outline-color", colors.buildingOutline);
    for (const id of ["park", "landuse_park", "landcover_wood"]) {
      paint(id, "fill-color", colors.park);
    }
    paint("water", "fill-color", colors.water);
    paint("waterway", "line-color", colors.water);
    for (const id of ["highway_path", "road_pier"]) {
      paint(id, "line-color", colors.path);
    }
    paint("highway_minor", "line-color", colors.minorRoad);
    for (const id of ["highway_major_inner", "highway_major_subtle", "highway_motorway_inner", "highway_motorway_subtle", "tunnel_motorway_inner", "highway_motorway_bridge_inner"]) {
      paint(id, "line-color", colors.majorRoad);
    }
    for (const id of ["highway_major_casing", "highway_motorway_casing", "tunnel_motorway_casing", "highway_motorway_bridge_casing"]) {
      paint(id, "line-color", colors.roadCasing);
    }
    for (const id of ["highway-name-path", "highway-name-minor", "highway-name-major", "highway_name_other", "highway_name_motorway"]) {
      paint(id, "text-color", colors.roadLabel);
      paint(id, "text-halo-color", colors.roadHalo);
    }
  }

  // OpenMapTiles ships named landmarks in the `poi` source-layer (shops,
  // including malls, plus hospitals, museums, markets…). Positron and Dark
  // don't render that layer, so we draw text-only labels, tiered by importance.
  // Transit/utility POIs are skipped so real destinations aren't drowned out.
  const NOISY_CLASSES = [
    "bus",
    "railway",
    "aerialway",
    "parking",
    "toilets",
    "drinking_water",
    "waste_basket",
    "atm",
    "bicycle",
    "car",
    "entrance",
    "post",
    "information",
  ];

  function landmarkLabelLayersForTheme(theme) {
    const dark = (theme || currentMapColorTheme()) === "dark";
    const text = dark ? "#cfcfcf" : "#4a4a4a";
    const halo = dark ? "#0c0c0c" : "#ffffff";
    const nameField = ["coalesce", ["get", "name:latin"], ["get", "name"], ["get", "name_en"]];
    const base = {
      type: "symbol",
      source: "openmaptiles",
      "source-layer": "poi",
      layout: {
        "text-field": nameField,
        "text-font": ["Noto Sans Regular"],
        "text-size": 12,
        "text-max-width": 9,
        "text-anchor": "top",
        "text-offset": [0, 0.5],
        "symbol-sort-key": ["get", "rank"],
      },
      paint: {
        "text-color": text,
        "text-halo-color": halo,
        "text-halo-width": 1,
        "text-halo-blur": 0.5,
      },
    };
    const notNoisy = ["!", ["in", ["get", "class"], ["literal", NOISY_CLASSES]]];
    return [
      // Malls are the most common video backdrop, so give them their own
      // higher-contrast layer that wins label collisions.
      Object.assign({}, base, {
        id: "mv-poi-mall",
        minzoom: 14,
        filter: ["all", ["has", "name"], ["==", ["get", "subclass"], "mall"]],
        layout: Object.assign({}, base.layout, {
          "text-font": ["Noto Sans Bold"],
          "text-size": 13,
          "text-allow-overlap": true,
          "symbol-sort-key": 0,
        }),
      }),
      Object.assign({}, base, {
        id: "mv-poi-major",
        minzoom: 14,
        filter: ["all", ["has", "rank"], ["<", ["get", "rank"], 7], notNoisy],
      }),
      Object.assign({}, base, {
        id: "mv-poi-minor",
        minzoom: 16,
        filter: ["all", ["has", "rank"], [">=", ["get", "rank"], 7], notNoisy],
      }),
    ];
  }

  function addLandmarkLabelsToMap(map) {
    for (const layer of landmarkLabelLayersForTheme()) {
      if (!map.getLayer(layer.id)) map.addLayer(layer);
    }
  }

  // Wire a map to theme changes and keep landmark labels across style swaps.
  function setupMapThemeAndLandmarks(map) {
    const systemTheme = window.matchMedia("(prefers-color-scheme: dark)");
    map.on("style.load", () => {
      applyMapPalette(map);
      addLandmarkLabelsToMap(map);
    });
    systemTheme.addEventListener("change", (event) => {
      // Rebuild the map when the OS preference changes; UI colors remain CSS-only.
      map.setStyle(styleUrlForTheme(event.matches ? "dark" : "light"), { diff: false });
    });
  }

  const CENTER = [-89.62, 20.975];

  function createMap(container, center = CENTER, zoom = 14) {
    const map = new maplibregl.Map({
      container,
      style: styleUrlForTheme(),
      center,
      zoom,
      attributionControl: false,
    });
    map.addControl(new maplibregl.AttributionControl({ compact: true }));
    setupMapThemeAndLandmarks(map);
    return map;
  }

  // HTML and stored records use lat/lng; MapLibre expects [lng, lat].
  function coordinatesFromElement(element) {
    if (element.dataset.lat == null || element.dataset.lng == null) return null;
    const lat = Number(element.dataset.lat);
    const lng = Number(element.dataset.lng);
    if (!Number.isFinite(lat) || !Number.isFinite(lng) || Math.abs(lat) > 90 || Math.abs(lng) > 180) return null;
    return [lng, lat];
  }

  function addMarker(map, coordinates, element, options = {}) {
    return new maplibregl.Marker({ ...options, element }).setLngLat(coordinates).addTo(map);
  }

  // Place markup is supplied by the server as an inert <template>, not HTML
  // parsed from a data attribute. Default marker content stays untouched.
  function addPlacePin(map, source, pin, options = {}) {
    const coordinates = coordinatesFromElement(source);
    if (!coordinates) return null;
    const template = source.querySelector("template.map-pin-content")
      || (source.nextElementSibling?.matches("template.map-pin-content") ? source.nextElementSibling : null);
    if (template) {
      pin.replaceChildren(template.content.cloneNode(true));
      if (pin.classList) pin.classList.add("map-photo-marker");
    }
    pin.title = source.dataset.title || "";
    return addMarker(map, coordinates, pin, options);
  }

  // Reusable for any list of places (videos, venues, bands) rendered as rows
  // with data-lat/data-lng and a .place-number button.
  function addNumberedPlaceMarkers(map, list, onSelect) {
    const markers = [];
    list.querySelectorAll(".place-row[data-lat][data-lng]").forEach((row, index) => {
      const number = row.querySelector(".place-number");
      if (!number) return;
      const pin = document.createElement("button");
      pin.type = "button";
      pin.className = "map-pin" + (index === 0 ? " map-pin-hot" : "");
      pin.setAttribute("aria-label", number.getAttribute("aria-label"));
      pin.textContent = String(index + 1);
      pin.addEventListener("click", () => onSelect(row));
      number.addEventListener("click", () => onSelect(row));
      const marker = addPlacePin(map, row, pin, { anchor: "bottom" });
      if (marker) markers.push(marker);
    });
    return markers;
  }

  const POI_LAYERS = ["mv-poi-mall", "mv-poi-major", "mv-poi-minor"];

  // Snap taps near a named landmark label; open-space taps remain exact.
  function nearestLandmark(map, point, radius = 24) {
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
    return nearestDistance <= radius ? nearest : null;
  }

  window.mvMap = {
    CENTER,
    createMap,
    coordinatesFromElement,
    addMarker,
    addPlacePin,
    addNumberedPlaceMarkers,
    nearestLandmark,
  };
})();
