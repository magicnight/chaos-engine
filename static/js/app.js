/* ================================================================
   C.H.A.O.S. — Connected Human-Augmented OSINT Suite
   Gridstack Init, Layout, Settings, Boot, SSE, Globe/Map
   ================================================================ */
var D = null;
var S = null;

var globe = null;
var globeInitialized = false;
var flightsVisible = true;
var lowPerfMode = localStorage.getItem('chaos_low_perf') === 'true';
var isFlat = true;
var currentRegion = 'world';
var flatSvg, flatProjection, flatPath, flatG, flatZoom, flatW, flatH;
var grid = null;

if(lowPerfMode) document.body.classList.add('low-perf');

   MAP
   ================================================================ */
var mapLifecycleBound = false;

function bindMapLifecycleEvents() {
  if (mapLifecycleBound) return;
  mapLifecycleBound = true;
  window.addEventListener('resize', function() { refreshMapViewport(); });
  window.addEventListener('orientationchange', function() { setTimeout(function() { refreshMapViewport(true); }, 150); });
  document.addEventListener('visibilitychange', function() { if (!document.hidden) setTimeout(function() { refreshMapViewport(true); }, 150); });
}

function setMapLoading(show, text) {
  var overlay = document.getElementById('mapLoading');
  var label = document.getElementById('mapLoadingText');
  if (!overlay || !label) return;
  if (text) label.textContent = text;
  overlay.classList.toggle('show', show);
}

function renderMapLegend() {
  var items = [
    { c: '#64f0c8', l: 'Air Traffic' }, { c: '#ff5f63', l: 'Thermal/Fire' },
    { c: 'rgba(255,120,80,0.8)', l: 'Conflict' }, { c: '#44ccff', l: 'SDR Receiver' },
    { c: '#ffe082', l: 'Nuclear Site' }, { c: '#ffb84c', l: 'OSINT Event' },
    { c: '#69f0ae', l: 'Health Alert' }, { c: '#81d4fa', l: 'Earthquake' },
    { c: '#ff9800', l: 'Weather Alert' }
  ];
  var el = document.getElementById('mapLegend');
  if (el) el.innerHTML = items.map(function(x) {
    return '<div class="leg-item"><div class="leg-dot" style="background:' + x.c + '"></div>' + x.l + '</div>';
  }).join('');
}

function renderRegionControls() {
  var bar = document.getElementById('mapRegionBar');
  if (!bar) return;
  if (isMobileLayout()) { bar.textContent = ''; bar.style.display = 'none'; return; }
  bar.innerHTML = getRegionControlsMarkup();
  bar.style.display = 'flex';
}

function initMap() {
  bindMapLifecycleEvents();
  renderMapLegend();
  if (isFlat) {
    if (globe && typeof globe.pauseAnimation === 'function') globe.pauseAnimation();
    var gEl = document.getElementById('globeViz');
    var fEl = document.getElementById('flatMapSvg');
    if (gEl) gEl.style.display = 'none';
    if (fEl) fEl.style.display = 'block';
    var pt = document.getElementById('projToggle');
    if (pt) pt.textContent = 'GLOBE MODE';
    var mh = document.getElementById('mapHint');
    if (mh) mh.textContent = 'SCROLL TO ZOOM \u00B7 DRAG TO PAN';
    if (!flatSvg) initFlatMap();
    else { flatG.selectAll('*').remove(); drawFlatMap(); }
    setMapLoading(false);
    return;
  }
  setMapLoading(true, 'Initializing 3D Globe');
  requestAnimationFrame(function() {
    try {
      initGlobe();
      setMapLoading(false);
    } catch (err) {
      isFlat = true;
      var gv = document.getElementById('globeViz');
      var fs = document.getElementById('flatMapSvg');
      if (gv) gv.style.display = 'none';
      if (fs) fs.style.display = 'block';
      var pt2 = document.getElementById('projToggle');
      if (pt2) pt2.textContent = 'GLOBE MODE';
      var mh2 = document.getElementById('mapHint');
      if (mh2) mh2.textContent = '3D LOAD FAILED \u00B7 FLAT MODE';
      if (!flatSvg) initFlatMap();
      else { flatG.selectAll('*').remove(); drawFlatMap(); }
      setMapLoading(false);
    }
  });
}

function initGlobe() {
  if (globeInitialized && globe) {
    if (typeof globe.resumeAnimation === 'function') globe.resumeAnimation();
    var gv = document.getElementById('globeViz');
    var fs = document.getElementById('flatMapSvg');
    if (gv) gv.style.display = 'block';
    if (fs) fs.style.display = 'none';
    var pt = document.getElementById('projToggle');
    if (pt) pt.textContent = 'FLAT MODE';
    var mh = document.getElementById('mapHint');
    if (mh) mh.textContent = 'DRAG TO ROTATE \u00B7 SCROLL TO ZOOM';
    return;
  }
  var container = document.getElementById('mapContainer');
  if (!container) return;
  var w = container.clientWidth;
  var h = container.clientHeight || 560;

  globe = Globe()
    .width(w).height(h)
    .globeImageUrl('//unpkg.com/three-globe@2.33.0/example/img/earth-night.jpg')
    .bumpImageUrl('//unpkg.com/three-globe@2.33.0/example/img/earth-topology.png')
    .backgroundImageUrl('')
    .backgroundColor('rgba(0,0,0,0)')
    .atmosphereColor('#64f0c8')
    .atmosphereAltitude(0.18)
    .showGraticules(true)
    .pointAltitude(function(d) { return d.alt || 0.01; })
    .pointRadius(function(d) { return d.size || 0.3; })
    .pointColor(function(d) { return d.color; })
    .pointLabel(function(d) { return '<b>' + (d.popHead || '') + '</b><br><span style="opacity:0.7">' + (d.popMeta || '') + '</span>'; })
    .onPointClick(function(pt, ev) { showPopup(ev, pt.popHead, pt.popText, pt.popMeta, pt.lat, pt.lng, pt.alt); })
    .onPointHover(function(pt) { var el = document.getElementById('globeViz'); if (el) el.style.cursor = pt ? 'pointer' : 'grab'; })
    .arcColor(function(d) { return d.color; })
    .arcStroke(function(d) { return d.stroke || 0.4; })
    .arcDashLength(0.4)
    .arcDashGap(0.2)
    .arcDashAnimateTime(lowPerfMode ? 0 : 2000)
    .arcAltitudeAutoScale(0.3)
    .arcLabel(function(d) { return d.label || ''; })
    .ringColor(function() { return function(t) { return 'rgba(255,120,80,' + (1 - t) + ')'; }; })
    .ringMaxRadius(function(d) { return d.maxR || 3; })
    .ringPropagationSpeed(function(d) { return d.speed || 2; })
    .ringRepeatPeriod(function(d) { return d.period || 800; })
    .labelText(function(d) { return d.text; })
    .labelSize(function(d) { return d.size || 0.4; })
    .labelColor(function(d) { return d.color || 'rgba(106,138,130,0.9)'; })
    .labelDotRadius(0)
    .labelAltitude(0.012)
    .labelResolution(2)
    (document.getElementById('globeViz'));

  var scene = globe.scene();
  var renderer = globe.renderer();
  renderer.setClearColor(0x000000, 0);
  var starGeom = new THREE.BufferGeometry();
  var starVerts = [];
  for (var i = 0; i < 2000; i++) {
    var r = 800 + Math.random() * 200;
    var theta = Math.random() * Math.PI * 2;
    var phi = Math.acos(2 * Math.random() - 1);
    starVerts.push(r * Math.sin(phi) * Math.cos(theta), r * Math.sin(phi) * Math.sin(theta), r * Math.cos(phi));
  }
  starGeom.setAttribute('position', new THREE.Float32BufferAttribute(starVerts, 3));
  scene.add(new THREE.Points(starGeom, new THREE.PointsMaterial({ color: 0x88bbaa, size: 0.8, transparent: true, opacity: 0.6 })));

  scene.traverse(function(obj) {
    if (obj.material && obj.type === 'Line') {
      obj.material.color.set(0x1a3a2a);
      obj.material.opacity = 0.3;
      obj.material.transparent = true;
    }
  });

  globe.pointOfView(regionPOV.world, 0);
  globe.controls().autoRotate = !lowPerfMode;
  globe.controls().autoRotateSpeed = 0.3;
  globe.controls().enableDamping = true;
  globe.controls().dampingFactor = 0.1;

  var rotateTimeout;
  var el = document.getElementById('globeViz');
  if (el) {
    el.addEventListener('mousedown', function() { globe.controls().autoRotate = false; clearTimeout(rotateTimeout); });
    el.addEventListener('mouseup', function() { rotateTimeout = setTimeout(function() { if (globe && !lowPerfMode) globe.controls().autoRotate = true; }, 10000); });
  }

  plotMarkers();

  if (isFlat) {
    var gv = document.getElementById('globeViz');
    var fs = document.getElementById('flatMapSvg');
    if (gv) gv.style.display = 'none';
    if (fs) fs.style.display = 'block';
    initFlatMap();
  } else {
    var gv2 = document.getElementById('globeViz');
    var fs2 = document.getElementById('flatMapSvg');
    if (gv2) gv2.style.display = 'block';
    if (fs2) fs2.style.display = 'none';
    var pt2 = document.getElementById('projToggle');
    if (pt2) pt2.textContent = 'FLAT MODE';
    var mh2 = document.getElementById('mapHint');
    if (mh2) mh2.textContent = 'DRAG TO ROTATE \u00B7 SCROLL TO ZOOM';
  }
  globeInitialized = true;
}

var airCoords = [
  { lat: 30, lon: 44 }, { lat: 24, lon: 120 }, { lat: 49, lon: 32 }, { lat: 57, lon: 24 },
  { lat: 14, lon: 114 }, { lat: 37, lon: 127 }, { lat: 25, lon: -80 }, { lat: 4, lon: 2 },
  { lat: -34, lon: 18 }, { lat: 10, lon: 51 }
];
var nukeCoords = [
  { lat: 47.5, lon: 34.6 }, { lat: 51.4, lon: 30.1 }, { lat: 28.8, lon: 50.9 },
  { lat: 39.8, lon: 125.8 }, { lat: 37.4, lon: 141 }, { lat: 31.0, lon: 35.1 },
  { lat: 48.1, lon: 16.4 }, { lat: 46.5, lon: 1.3 }, { lat: 52.2, lon: -0.5 },
  { lat: 33.4, lon: 130.3 }
];
var whoGeo = [
  { lat: 0.3, lon: 32.6 }, { lat: -6.2, lon: 106.8 }, { lat: -4.3, lon: 15.3 },
  { lat: 35, lon: 105 }, { lat: 12.5, lon: 105 }, { lat: 28, lon: 84 },
  { lat: 24, lon: 45 }, { lat: 30, lon: 70 }, { lat: -0.8, lon: 11.6 }, { lat: 9, lon: 38 }
];
var osintGeo = [
  { lat: 45, lon: 41 }, { lat: 48, lon: 37 }, { lat: 48.5, lon: 37.5 },
  { lat: 45, lon: 40.2 }, { lat: 50.6, lon: 36.6 }, { lat: 48.5, lon: 35 }
];
var globalHubs = [
  { lat: 40.6, lon: -73.8 }, { lat: 51.5, lon: -0.5 }, { lat: 25.3, lon: 55.4 },
  { lat: 1.4, lon: 103.8 }, { lat: -33.9, lon: 151.2 }, { lat: -23.4, lon: -46.5 }
];

function buildMapPoints() {
  var points = [];
  var labels = [];

  if (flightsVisible) {
    S.air.forEach(function(a, i) {
      var c = airCoords[i]; if (!c) return;
      points.push({
        lat: c.lat, lng: c.lon, size: 0.25 + a.total / 200, alt: 0.015,
        color: 'rgba(100,240,200,0.8)', type: 'air', priority: 1,
        popHead: a.region, popMeta: 'Air Activity',
        popText: a.total + ' aircraft tracked<br>No callsign: ' + a.noCallsign + '<br>High altitude: ' + a.highAlt
      });
      labels.push({ lat: c.lat, lng: c.lon + 2, text: a.region.replace(' Region', '') + ' ' + a.total, size: 0.35, color: 'rgba(106,138,130,0.8)' });
    });
  }

  S.thermalPoints.forEach(function(f) {
    if (!f.lat || !f.lon) return;
    points.push({
      lat: f.lat, lng: f.lon, size: 0.12 + Math.min(f.frp / 200, 0.3), alt: 0.008,
      color: 'rgba(255,95,99,0.7)', type: 'thermal', priority: 3,
      popHead: 'Thermal Detection', popMeta: 'FIRMS Satellite',
      popText: 'FRP: ' + f.frp.toFixed(1) + ' MW<br>Brightness: ' + f.brightness
    });
  });

  S.nuke.forEach(function(n, i) {
    var c = nukeCoords[i]; if (!c) return;
    points.push({
      lat: c.lat, lng: c.lon, size: 0.3, alt: 0.012,
      color: n.anomaly ? 'rgba(255,95,99,0.9)' : 'rgba(255,224,130,0.8)', type: 'nuke', priority: 2,
      popHead: n.site, popMeta: 'Radiation Monitoring',
      popText: 'Status: ' + (n.anomaly ? 'ANOMALY' : 'Normal') + '<br>Avg CPM: ' + (n.cpm ? n.cpm.toFixed(1) : 'No data')
    });
  });

  S.quakes.forEach(function(q) {
    if (!q.lat || !q.lon) return;
    points.push({
      lat: q.lat, lng: q.lon, size: 0.15 + Math.min(q.mag / 8, 0.4), alt: 0.01,
      color: 'rgba(129,212,250,0.8)', type: 'quake', priority: 2,
      popHead: 'M' + q.mag.toFixed(1) + ' Earthquake', popMeta: 'USGS',
      popText: q.place
    });
  });

  S.weatherAlerts.forEach(function(a) {
    if (!a.lat || !a.lon) return;
    points.push({
      lat: a.lat, lng: a.lon, size: 0.22, alt: 0.01,
      color: 'rgba(255,152,0,0.8)', type: 'weather', priority: 2,
      popHead: a.event, popMeta: 'NOAA/NWS \u00B7 ' + a.severity,
      popText: a.headline || ''
    });
  });

  S.whoAlerts.slice(0, 10).forEach(function(w, i) {
    var c = whoGeo[i]; if (!c) return;
    points.push({
      lat: c.lat, lng: c.lon, size: 0.25, alt: 0.01,
      color: 'rgba(105,240,174,0.7)', type: 'health', priority: 2,
      popHead: w.title, popMeta: 'WHO Outbreak', popText: w.summary || ''
    });
  });

  S.tgUrgent.slice(0, 6).forEach(function(post, i) {
    var c = osintGeo[i]; if (!c) return;
    points.push({
      lat: c.lat, lng: c.lon, size: 0.3, alt: 0.018,
      color: 'rgba(255,184,76,0.8)', type: 'osint', priority: 2,
      popHead: (post.channel || '').toUpperCase(), popMeta: (post.views ? post.views.toLocaleString() : '?') + ' views',
      popText: cleanText((post.text || '').substring(0, 200))
    });
  });

  return { points: points, labels: labels };
}

function plotMarkers() {
  if (!globe) return;
  var mp = buildMapPoints();
  var points = mp.points;
  var labels = mp.labels;

  globe.pointsData(points);
  globe.labelsData(labels);

  var conflictRings = S.acledEvents.slice(0, 20).map(function(e) {
    var logFatal = Math.log2(Math.max(e.fatalities || 1, 1));
    return {
      lat: e.lat, lng: e.lon || e.lng,
      maxR: Math.max(2, Math.min(6, 1 + logFatal)),
      speed: 1.5 + Math.random(),
      period: 600 + Math.random() * 600
    };
  });
  globe.ringsData(conflictRings);

  var arcs = [];
  if (flightsVisible) {
    for (var i = 0; i < S.air.length; i++) {
      for (var j = i + 1; j < S.air.length; j++) {
        var a = S.air[i], b = S.air[j];
        var from = airCoords[i], to = airCoords[j];
        if (!from || !to) continue;
        var traffic = a.total + b.total;
        if (traffic < 30) continue;
        var ncRatio = (a.noCallsign + b.noCallsign) / Math.max(traffic, 1);
        var color = ncRatio > 0.15 ? ['rgba(255,95,99,0.6)', 'rgba(255,95,99,0.15)'] :
                    ncRatio > 0.05 ? ['rgba(255,184,76,0.5)', 'rgba(255,184,76,0.1)'] :
                                     ['rgba(100,240,200,0.4)', 'rgba(100,240,200,0.08)'];
        arcs.push({
          startLat: from.lat, startLng: from.lon, endLat: to.lat, endLng: to.lon,
          color: color, stroke: Math.max(0.3, Math.min(1.2, traffic / 120)),
          label: from.lat + ',' + from.lon + ' - ' + to.lat + ',' + to.lon + ': ' + traffic + ' aircraft'
        });
      }
    }
    S.air.forEach(function(a, i) {
      if (!airCoords[i] || a.total < 25) return;
      globalHubs.forEach(function(hub) {
        var dLat = Math.abs(airCoords[i].lat - hub.lat);
        var dLon = Math.abs(airCoords[i].lon - hub.lon);
        if (dLat + dLon < 20) return;
        arcs.push({
          startLat: airCoords[i].lat, startLng: airCoords[i].lon,
          endLat: hub.lat, endLng: hub.lon,
          color: ['rgba(100,240,200,0.2)', 'rgba(100,240,200,0.05)'],
          stroke: 0.3
        });
      });
    });
  }
  globe.arcsData(arcs);

  if (typeof globe.onZoom === 'function') {
    globe.onZoom(function() {
      var alt = globe.pointOfView().altitude;
      var sf = Math.max(0.6, Math.min(2.5, 1.5 / alt));
      globe.pointRadius(function(d) { return (d.size || 0.3) * sf; });
      globe.labelSize(function(d) { return alt < 1.8 ? (d.size || 0.4) : 0; });
      globe.arcStroke(function(d) { return (d.stroke || 0.4) * Math.max(0.5, Math.min(1.5, 1.2 / alt)); });
      if (alt > 2.0) globe.pointsData(points.filter(function(p) { return (p.priority || 3) <= 1; }));
      else if (alt > 1.2) globe.pointsData(points.filter(function(p) { return (p.priority || 3) <= 2; }));
      else globe.pointsData(points);
    });
  }
}

function showPopup(event, head, text, meta, lat, lng, alt) {
  var popup = document.getElementById('mapPopup');
  var container = document.getElementById('mapContainer');
  if (!popup || !container) return;
  var rect = container.getBoundingClientRect();
  var left, top;
  if (!isFlat && lat != null && globe && typeof globe.getScreenCoords === 'function') {
    var sc = globe.getScreenCoords(lat, lng, alt || 0.01);
    if (!sc || isNaN(sc.x) || isNaN(sc.y) || sc.x < 0 || sc.y < 0 || sc.x > rect.width || sc.y > rect.height) {
      if (event && event.clientX != null) { left = event.clientX - rect.left + 10; top = event.clientY - rect.top - 10; }
      else return;
    } else { left = sc.x + 10; top = sc.y - 10; }
  } else if (event && event.clientX != null) {
    left = event.clientX - rect.left + 10;
    top = event.clientY - rect.top - 10;
  } else {
    left = rect.width / 2 - 140; top = rect.height / 2 - 60;
  }
  if (left + 290 > rect.width) left = left - 300;
  if (top + 150 > rect.height) top = top - 160;
  if (left < 0) left = 10;
  if (top < 0) top = 10;
  popup.style.left = left + 'px'; popup.style.top = top + 'px';
  popup.querySelector('.pp-head').textContent = head || '';
  popup.querySelector('.pp-text').innerHTML = text || '';
  popup.querySelector('.pp-meta').textContent = meta || '';
  popup.classList.add('show');
}
function closePopup() { var p = document.getElementById('mapPopup'); if (p) p.classList.remove('show'); }

function toggleFlights() {
  flightsVisible = !flightsVisible;
  var ft = document.getElementById('flightToggle');
  if (ft) ft.classList.toggle('off', !flightsVisible);
  if (isFlat) {
    if (flatG) { flatG.selectAll('*').remove(); drawFlatMap(); }
    return;
  }
  if (!globe) return;
  if (flightsVisible) plotMarkers();
  else {
    globe.arcsData([]);
    globe.pointsData(globe.pointsData().filter(function(p) { return p.type !== 'air'; }));
  }
}

function toggleMapMode() {
  isFlat = !isFlat;
  var btn = document.getElementById('projToggle');
  var hint = document.getElementById('mapHint');
  if (btn) btn.textContent = isFlat ? 'GLOBE MODE' : 'FLAT MODE';
  if (hint) hint.textContent = isFlat ? 'SCROLL TO ZOOM \u00B7 DRAG TO PAN' : 'DRAG TO ROTATE \u00B7 SCROLL TO ZOOM';
  var globeEl = document.getElementById('globeViz');
  var flatEl = document.getElementById('flatMapSvg');
  if (isFlat) {
    if (globe && typeof globe.pauseAnimation === 'function') globe.pauseAnimation();
    if (globeEl) globeEl.style.display = 'none';
    if (flatEl) flatEl.style.display = 'block';
    setMapLoading(false);
    if (!flatSvg) initFlatMap();
    else { flatG.selectAll('*').remove(); drawFlatMap(); }
  } else {
    if (flatEl) flatEl.style.display = 'none';
    setMapLoading(true, 'Initializing 3D Globe');
    requestAnimationFrame(function() {
      try {
        initGlobe();
        if (globe && typeof globe.resumeAnimation === 'function') globe.resumeAnimation();
        if (globeEl) globeEl.style.display = 'block';
        setMapLoading(false);
      } catch (err) {
        isFlat = true;
        if (globeEl) globeEl.style.display = 'none';
        if (flatEl) flatEl.style.display = 'block';
        if (btn) btn.textContent = 'GLOBE MODE';
        if (hint) hint.textContent = '3D LOAD FAILED \u00B7 FLAT MODE';
        if (!flatSvg) initFlatMap();
        else { flatG.selectAll('*').remove(); drawFlatMap(); }
        setMapLoading(false);
      }
    });
  }
}

function initFlatMap() {
  var container = document.getElementById('mapContainer');
  if (!container) return;
  flatW = container.clientWidth; flatH = container.clientHeight || 560;
  flatSvg = d3.select('#flatMapSvg').attr('viewBox', '0 0 ' + flatW + ' ' + flatH).attr('preserveAspectRatio', 'xMidYMid meet');
  flatProjection = d3.geoNaturalEarth1().fitSize([flatW - 20, flatH - 20], { type: 'Sphere' }).translate([flatW / 2, flatH / 2]);
  flatPath = d3.geoPath(flatProjection);
  flatG = flatSvg.append('g');
  flatZoom = d3.zoom().scaleExtent([1, 12]).on('zoom', function(event) {
    flatG.attr('transform', event.transform);
    var k = event.transform.k;
    flatG.selectAll('.marker-circle').attr('r', function() { return +this.dataset.baseR / Math.sqrt(k); });
    flatG.selectAll('.marker-label').style('font-size', Math.max(7, 9 / Math.sqrt(k)) + 'px')
      .style('display', k >= 2.5 ? 'block' : 'none');
  });
  flatSvg.call(flatZoom);
  drawFlatMap();
}

function drawFlatMap() {
  flatG.append('path').datum(d3.geoGraticule()()).attr('class', 'graticule').attr('d', flatPath);
  fetch('https://cdn.jsdelivr.net/npm/world-atlas@2/countries-110m.json')
    .then(function(r) { return r.json(); }).then(function(world) {
      var countries = topojson.feature(world, world.objects.countries);
      flatG.selectAll('path.land').data(countries.features).enter().append('path').attr('class', 'land').attr('d', flatPath);
      flatG.append('path').datum(topojson.mesh(world, world.objects.countries, function(a, b) { return a !== b; })).attr('class', 'border').attr('d', flatPath);
      plotFlatMarkers();
    }).catch(function() { plotFlatMarkers(); });
}

function plotFlatMarkers() {
  var mg = flatG.append('g').attr('class', 'markers');
  var proj = flatProjection;
  function addPt(lat, lon, r, fill, stroke, onClick, priority) {
    var p = proj([lon, lat]); if (!p || !p[0] || !p[1]) return null;
    var g = mg.append('g').attr('transform', 'translate(' + p[0] + ',' + p[1] + ')').style('cursor', 'pointer').attr('data-priority', priority || 3);
    if (onClick) g.on('click', function(ev) { ev.stopPropagation(); onClick(ev); });
    g.append('circle').attr('class', 'marker-circle').attr('r', r).attr('data-base-r', r).attr('fill', fill).attr('stroke', stroke).attr('stroke-width', 0.8);
    return g;
  }

  if (flightsVisible) {
    S.air.forEach(function(a, i) {
      var c = airCoords[i]; if (!c) return;
      var g = addPt(c.lat, c.lon, 4 + a.total / 40, 'rgba(100,240,200,0.7)', 'rgba(100,240,200,0.3)',
        function(ev) { showPopup(ev, a.region, a.total + ' aircraft<br>No callsign: ' + a.noCallsign + '<br>High alt: ' + a.highAlt, 'Air Activity'); }, 1);
      if (g) g.append('text').attr('class', 'marker-label').attr('x', 10).attr('y', 3).attr('fill', 'var(--dim)').attr('font-size', '9px').attr('font-family', 'var(--mono)').text(a.region.replace(' Region', '') + ' ' + a.total);
    });
  }

  S.thermalPoints.forEach(function(f) {
    if (!f.lat || !f.lon) return;
    addPt(f.lat, f.lon, 2 + Math.min(f.frp / 50, 5), 'rgba(255,95,99,0.6)', 'rgba(255,95,99,0.2)',
      function(ev) { showPopup(ev, 'Thermal', 'FRP: ' + f.frp.toFixed(1) + ' MW', 'FIRMS'); }, 3);
  });

  S.nuke.forEach(function(n, i) {
    var c = nukeCoords[i]; if (!c) return;
    addPt(c.lat, c.lon, 4, 'rgba(255,224,130,0.7)', 'rgba(255,224,130,0.3)',
      function(ev) { showPopup(ev, n.site, 'CPM: ' + (n.cpm ? n.cpm.toFixed(1) : '--'), 'Radiation'); }, 2);
  });

  S.quakes.forEach(function(q) {
    if (!q.lat || !q.lon) return;
    addPt(q.lat, q.lon, 3 + Math.min(q.mag, 6), 'rgba(129,212,250,0.7)', 'rgba(129,212,250,0.3)',
      function(ev) { showPopup(ev, 'M' + q.mag.toFixed(1), q.place, 'USGS Earthquake'); }, 2);
  });

  S.weatherAlerts.forEach(function(a) {
    if (!a.lat || !a.lon) return;
    addPt(a.lat, a.lon, 4, 'rgba(255,152,0,0.7)', 'rgba(255,152,0,0.3)',
      function(ev) { showPopup(ev, a.event, a.headline || '', 'NOAA/NWS'); }, 2);
  });

  S.whoAlerts.slice(0, 10).forEach(function(w, i) {
    var c = whoGeo[i]; if (!c) return;
    addPt(c.lat, c.lon, 3.5, 'rgba(105,240,174,0.6)', 'rgba(105,240,174,0.2)',
      function(ev) { showPopup(ev, w.title, w.summary || '', 'WHO'); }, 2);
  });

  S.tgUrgent.slice(0, 6).forEach(function(p, i) {
    var c = osintGeo[i]; if (!c) return;
    addPt(c.lat, c.lon, 4, 'rgba(255,184,76,0.7)', 'rgba(255,184,76,0.3)',
      function(ev) { showPopup(ev, (p.channel || '').toUpperCase(), cleanText((p.text || '').substring(0, 200)), (p.views || '?') + ' views'); }, 2);
  });

  S.acledEvents.slice(0, 20).forEach(function(e) {
    var eLon = e.lon || e.lng;
    var p = proj([eLon, e.lat]); if (!p || !p[0] || !p[1]) return;
    var r = Math.max(4, Math.min(14, 2 + Math.log2(Math.max(e.fatalities || 1, 1)) * 1.5));
    var g = mg.append('g').attr('transform', 'translate(' + p[0] + ',' + p[1] + ')').style('cursor', 'pointer').attr('data-priority', 1)
      .on('click', function(ev) { ev.stopPropagation(); showPopup(ev, e.type || 'CONFLICT', (e.fatalities || 0) + ' fatalities<br>' + (e.location || '') + ', ' + (e.country || ''), 'ACLED'); });
    g.append('circle').attr('class', 'conflict-ring marker-circle').attr('r', r).attr('data-base-r', r).attr('fill', 'none').attr('stroke', 'rgba(255,120,80,0.7)').attr('stroke-width', 1.5);
    g.append('circle').attr('r', r * 0.4).attr('fill', 'rgba(255,120,80,0.3)');
  });

  if (flightsVisible) {
    var cG = flatG.append('g').attr('class', 'corridors-layer');
    for (var i = 0; i < S.air.length; i++) {
      for (var j = i + 1; j < S.air.length; j++) {
        var a = S.air[i], b = S.air[j], from = airCoords[i], to = airCoords[j];
        if (!from || !to) continue;
        var traffic = a.total + b.total; if (traffic < 30) continue;
        var ncR = (a.noCallsign + b.noCallsign) / Math.max(traffic, 1);
        var clr = ncR > 0.15 ? 'rgba(255,95,99,0.4)' : ncR > 0.05 ? 'rgba(255,184,76,0.35)' : 'rgba(100,240,200,0.25)';
        var interp = d3.geoInterpolate([from.lon, from.lat], [to.lon, to.lat]);
        var coords = []; for (var k = 0; k <= 40; k++) coords.push(interp(k / 40));
        cG.append('path').datum({ type: 'Feature', geometry: { type: 'LineString', coordinates: coords } }).attr('d', flatPath).attr('fill', 'none').attr('stroke', clr).attr('stroke-width', Math.max(0.8, Math.min(3, traffic / 80)));
      }
    }
    S.air.forEach(function(a, idx) {
      if (!airCoords[idx] || a.total < 25) return;
      globalHubs.forEach(function(hub) {
        if (Math.abs(airCoords[idx].lat - hub.lat) + Math.abs(airCoords[idx].lon - hub.lon) < 20) return;
        var interp = d3.geoInterpolate([airCoords[idx].lon, airCoords[idx].lat], [hub.lon, hub.lat]);
        var coords = []; for (var k = 0; k <= 40; k++) coords.push(interp(k / 40));
        cG.append('path').datum({ type: 'Feature', geometry: { type: 'LineString', coordinates: coords } }).attr('d', flatPath).attr('fill', 'none').attr('stroke', 'rgba(100,240,200,0.15)').attr('stroke-width', 0.6);
      });
    });
  }
}

function setRegion(r) {
  currentRegion = r;
  document.querySelectorAll('.region-btn').forEach(function(b) { b.classList.toggle('active', b.dataset.region === r); });
  closePopup();
  if (isFlat && flatSvg && flatZoom) {
    if (r === 'world') { flatSvg.transition().duration(750).call(flatZoom.transform, d3.zoomIdentity); return; }
    var bounds = flatRegionBounds[r];
    var p0 = flatProjection(bounds[0]), p1 = flatProjection(bounds[1]); if (!p0 || !p1) return;
    var dx = Math.abs(p1[0] - p0[0]), dy = Math.abs(p1[1] - p0[1]);
    var cx = (p0[0] + p1[0]) / 2, cy = (p0[1] + p1[1]) / 2;
    var scale = Math.min(flatW / dx, flatH / dy) * 0.85;
    flatSvg.transition().duration(750).call(flatZoom.transform, d3.zoomIdentity.translate(flatW / 2 - scale * cx, flatH / 2 - scale * cy).scale(scale));
  } else {
    var pov = regionPOV[r] || regionPOV.world;
    if (globe) globe.pointOfView(pov, 1000);
  }
}

function mapZoom(factor) {
  if (isFlat && flatSvg && flatZoom) {
    flatSvg.transition().duration(300).call(flatZoom.scaleBy, factor);
  } else if (globe) {
    var pov = globe.pointOfView();
    globe.pointOfView({ altitude: pov.altitude / factor }, 300);
  }
}
/* ================================================================
   GLOSSARY
   ================================================================ */
function renderGlossary() {
  var body = document.getElementById('glossaryBody');
  if (!body) return;
  body.innerHTML = signalGuideItems.map(function(item) {
    return '<div class="glossary-card">' +
      '<div class="glossary-term"><strong>' + item.term + '</strong><span class="glossary-tag">' + item.category + '</span></div>' +
      '<div class="glossary-line"><span class="glossary-label">Meaning</span>' + item.meaning + '</div>' +
      '<div class="glossary-line"><span class="glossary-label">Why it matters</span>' + item.matters + '</div>' +
      '<div class="glossary-line"><span class="glossary-label">Not proof of</span>' + item.notMeaning + '</div>' +
      '<div class="glossary-line"><span class="glossary-label">Example</span>' + item.example + '</div>' +
    '</div>';
  }).join('');
}

function openGlossary() {
  var overlay = document.getElementById('glossaryOverlay');
  if (!overlay) return;
  overlay.classList.add('show');
  document.body.style.overflow = 'hidden';
}

function closeGlossary() {
  var overlay = document.getElementById('glossaryOverlay');
  if (!overlay) return;
  overlay.classList.remove('show');
  document.body.style.overflow = '';
}
/* ================================================================
   SETTINGS PANEL
   ================================================================ */
function openSettings() {
  var overlay = document.getElementById('settingsOverlay');
  overlay.classList.add('show');
  document.body.style.overflow = 'hidden';
  var html = '<h3>Dashboard Settings</h3>';
  var cats = {};
  PANELS.forEach(function(p) {
    if (!cats[p.category]) cats[p.category] = [];
    cats[p.category].push(p);
  });
  Object.keys(cats).forEach(function(cat) {
    html += '<div class="settings-cat">' + cat + '</div>';
    cats[cat].forEach(function(p) {
      var visible = !!document.querySelector('[gs-id="' + p.id + '"]');
      html += '<label class="settings-toggle"><input type="checkbox" ' + (visible ? 'checked' : '') +
        ' onchange="togglePanel(\'' + p.id + '\', this.checked)"> ' + p.icon + ' ' + p.title + '</label>';
    });
  });
  html += '<div style="margin-top:16px;display:flex;gap:8px">';
  html += '<button onclick="resetLayout()" class="settings-btn">Reset Layout</button>';
  html += '<button onclick="togglePerfMode()" class="settings-btn">Toggle Visuals</button>';
  html += '<button onclick="closeSettings()" class="settings-btn primary">Close</button>';
  html += '</div>';
  document.getElementById('settingsContent').innerHTML = html;
}

function closeSettings() {
  var overlay = document.getElementById('settingsOverlay');
  overlay.classList.remove('show');
  document.body.style.overflow = '';
}

function togglePanel(id, show) {
  if (show) {
    var panel = PANELS.find(function(p) { return p.id === id; });
    if (!panel) return;
    var widgetHtml = buildPanelHtml(panel);
    grid.addWidget(widgetHtml, { id: panel.id, x: panel.x, y: panel.y, w: panel.w, h: panel.h });
    bindPanelControls();
    updatePanelById(id);
    if (id === 'globe') {
      updateGlobe();
      setTimeout(function() { initMap(); }, 100);
    }
  } else {
    var el = document.querySelector('[gs-id="' + id + '"]');
    if (el) grid.removeWidget(el);
  }
  saveLayout();
}
/* ================================================================
   LAYOUT PERSISTENCE
   ================================================================ */
function saveLayout() {
  if (!grid) return;
  var items = grid.getGridItems().map(function(el) {
    var n = el.gridstackNode;
    return { id: n.id, x: n.x, y: n.y, w: n.w, h: n.h };
  });
  localStorage.setItem('chaos_panel_layout', JSON.stringify(items));
}

function loadLayout() {
  var saved = localStorage.getItem('chaos_panel_layout');
  if (!saved) return false;
  try {
    var items = JSON.parse(saved);
    items.forEach(function(item) {
      var el = document.querySelector('[gs-id="' + item.id + '"]');
      if (el) grid.update(el, { x: item.x, y: item.y, w: item.w, h: item.h });
    });
    PANELS.forEach(function(p) {
      if (!items.find(function(i) { return i.id === p.id; })) {
        var el = document.querySelector('[gs-id="' + p.id + '"]');
        if (el) grid.removeWidget(el);
      }
    });
    return true;
  } catch(e) { return false; }
}

function resetLayout() {
  localStorage.removeItem('chaos_panel_layout');
  location.reload();
}
/* ================================================================
   GRIDSTACK PANEL BUILDER
   ================================================================ */
function buildPanelHtml(p) {
  var inner = '';
  if (p.id === 'globe') {
    inner = '<div class="panel-body" id="panel-globe" style="padding:0;position:relative"></div>';
  } else {
    inner = '<div class="panel-body" id="panel-' + p.id + '"></div>';
  }
  return '<div class="grid-stack-item" gs-id="' + p.id + '" gs-w="' + p.w + '" gs-h="' + p.h + '" gs-x="' + p.x + '" gs-y="' + p.y + '">' +
    '<div class="grid-stack-item-content">' +
      '<div class="panel-header">' +
        '<span class="panel-icon">' + p.icon + '</span>' +
        '<span class="panel-title">' + p.title + '</span>' +
        '<span class="panel-category">' + p.category + '</span>' +
        '<div class="panel-controls">' +
          '<button class="panel-btn panel-btn-min" title="Minimize">\u2212</button>' +
          '<button class="panel-btn panel-btn-max" title="Maximize">\u25A1</button>' +
          '<button class="panel-btn panel-btn-hide" title="Hide">\u00D7</button>' +
        '</div>' +
      '</div>' +
      inner +
    '</div>' +
  '</div>';
}

function bindPanelControls() {
  document.querySelectorAll('.panel-btn-hide').forEach(function(btn) {
    if (btn.dataset.bound) return;
    btn.dataset.bound = '1';
    btn.addEventListener('click', function() {
      var item = this.closest('.grid-stack-item');
      grid.removeWidget(item);
      saveLayout();
    });
  });

  document.querySelectorAll('.panel-btn-max').forEach(function(btn) {
    if (btn.dataset.bound) return;
    btn.dataset.bound = '1';
    btn.addEventListener('click', function() {
      var item = this.closest('.grid-stack-item');
      var node = item.gridstackNode;
      if (item.dataset.preMax) {
        var pre = JSON.parse(item.dataset.preMax);
        grid.update(item, pre);
        delete item.dataset.preMax;
      } else {
        item.dataset.preMax = JSON.stringify({x: node.x, y: node.y, w: node.w, h: node.h});
        grid.update(item, {x: 0, y: 0, w: 12, h: 8});
      }
      saveLayout();
      var gsId = item.getAttribute('gs-id');
      if (gsId === 'globe') {
        setTimeout(function() { refreshMapViewport(true); }, 200);
      }
    });
  });

  document.querySelectorAll('.panel-btn-min').forEach(function(btn) {
    if (btn.dataset.bound) return;
    btn.dataset.bound = '1';
    btn.addEventListener('click', function() {
      var item = this.closest('.grid-stack-item');
      var body = item.querySelector('.panel-body');
      if (body) {
        var isMin = body.style.display === 'none';
        body.style.display = isMin ? '' : 'none';
        this.textContent = isMin ? '\u2212' : '+';
        if (isMin) {
          var node = item.gridstackNode;
          if (item.dataset.preMin) {
            var pre = JSON.parse(item.dataset.preMin);
            grid.update(item, { h: pre.h });
            delete item.dataset.preMin;
          }
        } else {
          var node2 = item.gridstackNode;
          item.dataset.preMin = JSON.stringify({ h: node2.h });
          grid.update(item, { h: 1 });
        }
      }
      saveLayout();
    });
  });
}
/* ================================================================
   GRIDSTACK INITIALIZATION
   ================================================================ */
function initGridStack() {
  var container = document.getElementById('gridContainer');
  var panelsHtml = PANELS.map(function(p) { return buildPanelHtml(p); }).join('');
  container.innerHTML = panelsHtml;

  grid = GridStack.init({
    column: 12,
    cellHeight: 60,
    margin: 4,
    float: false,
    animate: true,
    draggable: { handle: '.panel-header' },
    resizable: { handles: 'se' }
  }, container);

  bindPanelControls();

  var hadLayout = loadLayout();

  // Hide live video panels by default (user enables via settings)
  if (!hadLayout) {
    PANELS.forEach(function(p) {
      if (p.id.indexOf('live-') === 0) {
        var el = document.querySelector('[gs-id="' + p.id + '"]');
        if (el) grid.removeWidget(el);
      }
    });
  }

  grid.on('change', function() { saveLayout(); });

  grid.on('resizestop', function(event, el) {
    saveLayout();
    var gsId = el.getAttribute('gs-id');
    if (gsId === 'globe') {
      var panelGlobe = document.getElementById('panel-globe');
      if (globe && panelGlobe) {
        globe.width(panelGlobe.clientWidth).height(panelGlobe.clientHeight);
      }
      if (flatSvg) {
        var mc = document.getElementById('mapContainer');
        if (mc) {
          flatW = mc.clientWidth; flatH = mc.clientHeight;
          flatSvg.attr('viewBox', '0 0 ' + flatW + ' ' + flatH);
          if (flatProjection && flatG) {
            flatProjection = d3.geoNaturalEarth1().fitSize([flatW - 20, flatH - 20], { type: 'Sphere' }).translate([flatW / 2, flatH / 2]);
            flatPath = d3.geoPath(flatProjection);
            flatG.selectAll('*').remove();
            drawFlatMap();
          }
        }
      }
    }
  });

  handleResize();
  window.addEventListener('resize', handleResize);
}

function handleResize() {
  if (!grid) return;
  var w = window.innerWidth;
  if (w >= 1920) grid.column(12);
  else if (w >= 1280) grid.column(8);
  else if (w >= 768) grid.column(4);
  else grid.column(1);
}
/* ================================================================
   RESPONSIVE + REFRESH
   ================================================================ */
function refreshMapViewport(forceGlobeReflow) {
  var container = document.getElementById('mapContainer');
  if (!container) return;
  var width = container.clientWidth;
  var height = container.clientHeight || (isMobileLayout() ? 420 : 560);
  if (globe) {
    globe.width(width).height(height);
    if (forceGlobeReflow && !isFlat) {
      var globeEl = document.getElementById('globeViz');
      if (globeEl) {
        globeEl.style.display = 'none';
        requestAnimationFrame(function() { globeEl.style.display = 'block'; globe.width(width).height(height); });
      }
    }
  }
  if (flatSvg) {
    flatW = width; flatH = height;
    flatSvg.attr('viewBox', '0 0 ' + flatW + ' ' + flatH).attr('preserveAspectRatio', 'xMidYMid meet');
    if (flatProjection && flatG) {
      flatProjection = d3.geoNaturalEarth1().fitSize([flatW - 20, flatH - 20], { type: 'Sphere' }).translate([flatW / 2, flatH / 2]);
      flatPath = d3.geoPath(flatProjection);
      flatG.selectAll('*').remove();
      drawFlatMap();
    }
  }
}

function reinit() {
  renderTopbar();
  updateAllPanels();
  if (isFlat && flatG) { flatG.selectAll('*').remove(); drawFlatMap(); }
  else plotMarkers();
}
/* ================================================================
   BOOT SEQUENCE
   ================================================================ */
function runBoot() {
  var lines = [
    { text: 'INITIALIZING C.H.A.O.S. ENGINE v' + S.meta.version, delay: 0 },
    { text: 'CONNECTING ' + S.meta.sourcesQueried + ' OSINT SOURCES...', delay: 400 },
    { text: '&#9500;&#9472; OPENSKY &middot; FIRMS &middot; KIWISDR &middot; SAFECAST', delay: 700 },
    { text: '&#9500;&#9472; FRED &middot; BLS &middot; EIA &middot; TREASURY &middot; GSCPI', delay: 900 },
    { text: '&#9500;&#9472; TELEGRAM &middot; GDELT &middot; WHO &middot; ACLED', delay: 1100 },
    { text: '&#9492;&#9472; USGS &middot; NOAA &middot; SWPC &middot; CELESTRAK &middot; NVD', delay: 1300 },
    { text: 'SWEEP COMPLETE &mdash; <span class="count">' + S.meta.sourcesOk + '</span>/' + S.meta.sourcesQueried + ' SOURCES <span class="ok">OK</span>', delay: 1700 },
    { text: 'FLIGHT CORRIDORS: <span class="ok">ACTIVE</span> &middot; DUAL PROJECTION: <span class="ok">READY</span>', delay: 2100 },
    { text: 'INTELLIGENCE SYNTHESIS: <span class="ok">ACTIVE</span>', delay: 2400 }
  ];
  var container = document.getElementById('bootLines');
  document.getElementById('bootFinal').textContent = 'TERMINAL ACTIVE';
  var tl = gsap.timeline();
  tl.to('.logo-ring', { opacity: 1, duration: 0.6, ease: 'power2.out' }, 0);
  tl.to(container, { opacity: 1, duration: 0.3 }, 0.3);
  lines.forEach(function(line) {
    tl.call(function() {
      var div = document.createElement('div'); div.innerHTML = line.text; div.style.opacity = '0';
      container.appendChild(div); gsap.to(div, { opacity: 1, duration: 0.2 });
    }, [], line.delay / 1000 + 0.5);
  });
  tl.to('#bootFinal', { opacity: 1, duration: 0.4 }, 3.1);
  tl.to('#boot', { opacity: 0, duration: 0.5, ease: 'power2.in' }, 3.7);
  tl.set('#boot', { display: 'none' }, 4.2);
  tl.to('#bgRadial', { opacity: 1, duration: 1 }, 3.8);
  tl.to('#bgGrid', { opacity: 1, duration: 1.2 }, 4.0);
  tl.to('#scanline', { opacity: 1, duration: 0.8 }, 4.3);
  tl.to('#main', { opacity: 1, duration: 0.6 }, 3.9);
  tl.call(function() {
    gsap.from('.grid-stack-item-content,.topbar', { opacity: 0, y: 20, scale: 0.97, duration: 0.5, stagger: 0.06, ease: 'power2.out' });
    setTimeout(function() { gsap.from('.layer-item,.site-row,.econ-row', { opacity: 0, x: -12, duration: 0.25, stagger: 0.03, ease: 'power1.out' }); }, 500);
    setTimeout(function() { gsap.from('.ic,.tk-card', { opacity: 0, y: 12, duration: 0.25, stagger: 0.03, ease: 'power1.out' }); }, 600);
    setTimeout(function() { gsap.from('.mc', { opacity: 0, y: 8, duration: 0.25, stagger: 0.04, ease: 'power1.out' }); }, 800);
    setTimeout(function() { gsap.from('.idea-card', { opacity: 0, x: 12, duration: 0.3, stagger: 0.06, ease: 'power1.out' }); }, 900);
    setTimeout(function() {
      document.querySelectorAll('.mbar span,.smb span').forEach(function(bar) { var w = bar.style.width; bar.style.width = '0%'; gsap.to(bar, { width: w, duration: 1, ease: 'power2.out' }); });
      document.querySelectorAll('.spark-bar').forEach(function(bar) { var h = bar.style.height; bar.style.height = '0%'; gsap.to(bar, { height: h, duration: 0.8, ease: 'power2.out' }); });
    }, 1000);
  }, [], 4.0);
}
/* ================================================================
   SSE: Live Updates from Rust API
   ================================================================ */
function connectSSE() {
  if (typeof EventSource === 'undefined') return;
  if (location.protocol === 'file:') return;

  var es = new EventSource('/api/v1/sse');
  es.onmessage = function(e) {
    try {
      var msg = JSON.parse(e.data);
      if (msg.type === 'update') {
        fetch('/api/v1/data')
          .then(function(r) { return r.json(); })
          .then(function(data) {
            D = data;
            S = synthesize(D);
            reinit();
            var topbar = document.querySelector('.topbar');
            if (topbar) {
              topbar.style.borderColor = 'var(--accent)';
              setTimeout(function() { topbar.style.borderColor = ''; }, 1500);
            }
          })
          .catch(function() {});
      } else if (msg.type === 'sweep_start') {
        var badge = document.querySelector('.alert-badge');
        if (badge) { badge.textContent = 'SWEEPING...'; badge.style.borderColor = 'var(--accent)'; }
      }
    } catch (err) {}
  };
  es.onerror = function() {
    es.close();
    setTimeout(connectSSE, 5000);
  };
}
/* ================================================================
   INIT
   ================================================================ */
var booted = false;
function init() {
  initGridStack();
  renderTopbar();
  updateGlobe();
  updateAllPanels();
  renderGlossary();
  setTimeout(function() { initMap(); }, 50);
  if (!booted) { runBoot(); booted = true; }
  document.addEventListener('click', function(e) {
    var mc = document.getElementById('mapContainer');
    if (mc && mc.contains(e.target) && !e.target.closest('.map-popup')) closePopup();
    var card = e.target.closest('.tk-card[data-url]');
    if (card) {
      var url = safeExternalUrl(card.dataset.url);
      if (url) window.open(url, '_blank', 'noopener');
    }
  });
  document.addEventListener('keydown', function(e) {
    if (e.key === 'Escape') { closeGlossary(); closeSettings(); }
  });
}

document.addEventListener('DOMContentLoaded', function() {
  if (location.protocol === 'file:') return;

  fetch('/api/v1/data')
    .then(function(r) { return r.json(); })
    .then(function(data) {
      D = data;
      S = synthesize(D);
      init();
      connectSSE();
    })
    .catch(function(err) {
      document.getElementById('boot').innerHTML = '<div style="text-align:center;color:var(--dim);font-family:var(--mono);padding:40px"><div style="font-size:18px;color:var(--accent);margin-bottom:16px">CHAOS</div><div>Waiting for data...</div><div style="font-size:10px;margin-top:8px;opacity:0.5">API endpoint: /api/v1/data</div><div style="font-size:10px;margin-top:4px;opacity:0.5">Error: ' + (err.message || 'fetch failed') + '</div></div>';
    });
});
