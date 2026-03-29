/* ================================================================
   C.H.A.O.S. — Connected Human-Augmented OSINT Suite
   Panel Registry & Update Functions
   ================================================================ */
var PANELS = [
  { id: 'globe',     title: L.situationMap,     icon: '\uD83C\uDF10', category: L.catSituational,  w: 8, h: 5, x: 0, y: 0 },
  { id: 'sources',   title: L.sourceHealth,     icon: '\uD83D\uDCE1', category: L.catSystem,       w: 4, h: 4, x: 8, y: 0 },
  { id: 'markets',   title: L.marketData,        icon: '\uD83D\uDCCA', category: L.catFinancial,  w: 4, h: 3, x: 0, y: 5 },
  { id: 'risk',      title: L.riskGauges,        icon: '\u26A1',       category: L.catFinancial,  w: 4, h: 4, x: 4, y: 5 },
  { id: 'energy',    title: L.energyMacro,       icon: '\uD83D\uDEE2', category: L.catFinancial,  w: 4, h: 2, x: 8, y: 4 },
  { id: 'conflicts', title: L.conflicts,          icon: '\u2694',       category: L.catSecurity,   w: 4, h: 3, x: 8, y: 6 },
  { id: 'osint',     title: L.osintStream,        icon: '\uD83D\uDCE1', category: L.catSecurity,   w: 4, h: 4, x: 8, y: 9 },
  { id: 'news',      title: L.newsFeed,           icon: '\uD83D\uDCF0', category: L.catNews,       w: 4, h: 3, x: 0, y: 8 },
  { id: 'quakes',    title: L.seismicMonitor,     icon: '\uD83C\uDF0B', category: L.catNatural,    w: 4, h: 3, x: 4, y: 9 },
  { id: 'nuclear',   title: L.nuclearWatch,       icon: '\u2622',       category: L.catNatural,    w: 4, h: 3, x: 0, y: 11 },
  { id: 'cyber',     title: L.cyberThreats,       icon: '\uD83D\uDD12', category: L.catCyber,      w: 4, h: 3, x: 4, y: 12 },
  { id: 'space',     title: L.spaceWatch,         icon: '\uD83D\uDEF0', category: L.catSpace,      w: 4, h: 2, x: 8, y: 13 },
  { id: 'delta',     title: L.deltaChanges,       icon: '\uD83D\uDCC8', category: L.catSystem,     w: 4, h: 3, x: 0, y: 14 },
  { id: 'analysis',  title: L.aiBrief,            icon: '\uD83E\uDD16', category: L.catAI,         w: 4, h: 4, x: 4, y: 15 },
  { id: 'signals',   title: L.crossSignals,       icon: '\uD83D\uDD17', category: L.catAI,         w: 4, h: 3, x: 8, y: 15 },
  { id: 'sanctions',  title: L.sanctionsWatch,     icon: '\uD83D\uDEAB', category: L.catSecurity,   w: 4, h: 3, x: 0, y: 18 },
  { id: 'economy',    title: L.globalEconomy,      icon: '\uD83C\uDFE6', category: L.catFinancial,  w: 4, h: 3, x: 4, y: 18 },
  { id: 'climate',    title: L.climateEnviron,     icon: '\uD83C\uDF21', category: L.catNatural,    w: 4, h: 3, x: 8, y: 18 },
  { id: 'network',    title: L.networkIntel,       icon: '\uD83C\uDF10', category: L.catCyber,      w: 4, h: 2, x: 0, y: 21 },
  { id: 'transport',  title: L.transportAir,       icon: '\u2708',       category: L.catSituational, w: 4, h: 3, x: 4, y: 21 },
  { id: 'trends',     title: L.trendsInnovation,   icon: '\uD83D\uDCC8', category: L.catNews,        w: 4, h: 3, x: 8, y: 21 },
  { id: 'asteroids',  title: L.neoTracker,         icon: '\u2604',       category: L.catSpace,       w: 4, h: 2, x: 0, y: 24 },
  { id: 'srcmonitor', title: L.sourceMonitor,     icon: '\uD83D\uDCCA', category: L.catSystem,      w: 4, h: 4, x: 4, y: 24 },
  { id: 'live-bloomberg', title: L.liveBloomberg, icon: '\uD83D\uDCFA', category: L.catLive, w: 4, h: 4 },
  { id: 'live-aljazeera', title: L.liveAljazeera, icon: '\uD83D\uDCFA', category: L.catLive, w: 4, h: 4 },
  { id: 'live-france24',  title: L.liveFrance24,  icon: '\uD83D\uDCFA', category: L.catLive, w: 4, h: 4 },
  { id: 'live-dw',        title: L.liveDw,        icon: '\uD83D\uDCFA', category: L.catLive, w: 4, h: 4 },
  { id: 'live-euronews',  title: L.liveEuronews,  icon: '\uD83D\uDCFA', category: L.catLive, w: 4, h: 4 },
  { id: 'live-skynews',   title: L.liveSkynews,   icon: '\uD83D\uDCFA', category: L.catLive, w: 4, h: 4 },
  { id: 'live-cnbc',      title: L.liveCnbc,      icon: '\uD83D\uDCFA', category: L.catLive, w: 4, h: 4 },
  { id: 'live-nhk',       title: L.liveNhk,       icon: '\uD83D\uDCFA', category: L.catLive, w: 4, h: 4 },
  { id: 'live-cctv4',     title: L.liveCctv4,     icon: '\uD83D\uDCFA', category: L.catLive, w: 4, h: 4 },
  { id: 'live-tvbs',      title: L.liveTvbs,      icon: '\uD83D\uDCFA', category: L.catLive, w: 4, h: 4 },
  { id: 'live-cti',       title: L.liveCti,       icon: '\uD83D\uDCFA', category: L.catLive, w: 4, h: 4 },
  { id: 'live-ebc',       title: L.liveEbc,       icon: '\uD83D\uDCFA', category: L.catLive, w: 4, h: 4 },
  { id: 'live-phoenix',   title: L.livePhoenix,   icon: '\uD83D\uDCFA', category: L.catLive, w: 4, h: 4 },
  { id: 'live-cgtn',      title: L.liveCgtn,      icon: '\uD83D\uDCFA', category: L.catLive, w: 4, h: 4 },
  { id: 'live-cctvnews',  title: L.liveCctvnews,  icon: '\uD83D\uDCFA', category: L.catLive, w: 4, h: 4 }
];

var regionPOV = {
  world: { lat: 20, lng: 20, altitude: 1.8 },
  americas: { lat: 35, lng: -95, altitude: 1.0 },
  europe: { lat: 50, lng: 15, altitude: 1.0 },
  middleEast: { lat: 28, lng: 45, altitude: 1.1 },
  asiaPacific: { lat: 25, lng: 110, altitude: 1.2 },
  africa: { lat: 5, lng: 20, altitude: 1.2 }
};

var flatRegionBounds = {
  world:[[-180,-60],[180,80]], americas:[[-130,10],[-60,55]], europe:[[-12,34],[45,72]],
  middleEast:[[24,10],[65,45]], asiaPacific:[[60,-12],[180,55]], africa:[[-20,-36],[55,38]]
};

var signalGuideItems = [
  {term:'VIX',category:'Macro',meaning:'The CBOE Volatility Index, commonly used as a market-implied fear or volatility gauge.',matters:'Higher VIX often means more expected equity volatility and more defensive market positioning.',notMeaning:'Not a direct forecast of a crash and not a geopolitical indicator by itself.',example:'VIX above 20 with widening HY spreads is a stronger stress pattern than VIX alone.'},
  {term:'HY Spread',category:'Macro',meaning:'High-yield credit spread, shown here as a stress proxy from FRED credit data.',matters:'When spreads widen, markets are usually pricing more credit stress and tighter financial conditions.',notMeaning:'Not a recession call by itself. It is one stress signal among many.',example:'A rising HY Spread alongside higher VIX and weaker equities is a stronger risk-off pattern than HY alone.'},
  {term:'CPM',category:'Radiation',meaning:'Counts per minute from a radiation monitoring source, used here for relative radiation status at a site.',matters:'Useful for spotting anomalies against a sites normal range or comparing consecutive readings.',notMeaning:'Not a direct safety verdict on its own. Interpretation depends on local baseline and trend.',example:'A site reading 33 CPM can be normal if that locations usual background level is in the same range.'},
  {term:'FRP',category:'Thermal',meaning:'Fire Radiative Power. Intensity of a FIRMS hotspot, measured in megawatts.',matters:'Higher FRP usually means a hotter, larger, or more energetic fire event at that exact point.',notMeaning:'Not the intensity of the whole region and not automatic proof of conflict activity.',example:'FRP 92.3 MW describes one hotspot, while Total 1,451 describes the entire regional detection count.'},
  {term:'GSCPI',category:'Macro',meaning:'The Global Supply Chain Pressure Index, a broad indicator of global supply-chain strain.',matters:'Helps translate geopolitical or weather disruptions into likely pressure on shipping, inventory, and pricing.',notMeaning:'Not a live market price and not a company-specific supply-chain score.',example:'A higher GSCPI makes route or energy shocks more likely to spill into broader cost pressure.'},
  {term:'WHO Alert',category:'Health',meaning:'A WHO Disease Outbreak News item surfaced in the health layer.',matters:'Useful for watching outbreaks that could affect travel, supply chains, or regional operating conditions.',notMeaning:'Not a pandemic declaration and not automatically high severity.',example:'A WHO alert in a port-heavy region matters more if it overlaps shipping, border controls, or local instability.'},
  {term:'Sweep Delta',category:'Platform',meaning:'The change summary between the current sweep and the previous one.',matters:'Useful for spotting what changed recently instead of re-reading the full dashboard.',notMeaning:'Not a full risk model. It is a directional change layer on top of the raw signals.',example:'A delta marked risk-off with several new and escalated items means the latest sweep materially worsened the signal mix.'},
  {term:'SDR Receiver',category:'Signals',meaning:'A publicly reachable software-defined radio receiver in or near a region of interest.',matters:'Dense receiver coverage can give you more ability to monitor communications or signal activity.',notMeaning:'Not evidence of hostile emissions or a threat by itself.',example:'South China Sea SDR count means publicly accessible KiwiSDR receivers are available in or near that zone.'},
  {term:'No Callsign',category:'Air',meaning:'OpenSky received an aircraft track without a usable callsign or flight ID.',matters:'Useful as an opacity signal. A cluster of missing callsigns can indicate incomplete transponder metadata.',notMeaning:'Not proof of military, covert, or hostile activity on its own.',example:'No Callsign 6 of 152 means 6 tracks in that theater had no usable callsign in the feed.'}
];
function synthesize(raw) {
  var src = raw.sources || {};
  var v = {};

  var c = raw.chaos || {};
  v.meta = {
    version: c.version || '0.1.0',
    timestamp: c.timestamp || new Date().toISOString(),
    sourcesQueried: c.sourcesQueried || 0,
    sourcesOk: c.sourcesOk || 0,
    sourcesFailed: c.sourcesFailed || 0,
    totalDurationMs: c.totalDurationMs || 0
  };

  var osky = src.OpenSky || {};
  var oskyRegions = osky.regions || [];
  v.air = oskyRegions.map(function(r) { return {
    region: r.name || 'Unknown', total: r.total || 0,
    noCallsign: r.noCallsign || 0, highAlt: r.highAlt || 0,
    top: r.topCountries || []
  };});
  v.totalAir = v.air.reduce(function(s, a) { return s + a.total; }, 0);

  var firms = src.FIRMS || {};
  var hotspots = firms.hotspots || [];
  v.thermalCount = hotspots.length;
  v.thermalPoints = hotspots.map(function(h) { return {
    lat: h.lat || h.latitude || 0, lon: h.lon || h.longitude || 0,
    frp: h.frp || 0, brightness: h.brightness || 0
  };});

  var kiwi = src.KiwiSDR || {};
  v.sdrTotal = kiwi.totalReceivers || 0;
  v.sdrOnline = kiwi.onlineCount || 0;

  var sc = src.Safecast || {};
  v.nuke = (sc.sites || []).map(function(s) { return {
    site: s.site || s.name || 'Unknown', cpm: s.cpm || s.value || 0,
    anomaly: s.anomaly || false, n: s.measurements || s.n || 1
  };});

  var acled = src.ACLED || {};
  v.conflictEvents = acled.totalEvents || (acled.events || []).length;
  v.conflictFatalities = acled.totalFatalities || 0;
  v.acledEvents = (acled.events || acled.deadliestEvents || []).filter(function(e) { return e.lat && (e.lon || e.lng); });

  var who = src.WHO || {};
  v.whoAlerts = (who.alerts || []).map(function(a) { return {
    title: a.title || '', date: a.date || '', summary: a.summary || a.description || ''
  };});

  var promed = src.ProMED || {};
  v.promedAlerts = (promed.alerts || []).map(function(a) { return {
    title: a.title || '', date: a.pubDate || '', description: a.description || '', link: a.link || ''
  };});
  v.promedDiseases = promed.diseaseMentions || {};

  var gdelt = src.GDELT || {};
  v.newsArticles = (gdelt.allArticles || []).map(function(a) { return {
    title: a.title || '', url: a.url || '',
    domain: a.domain || a.source || 'GDELT', date: a.date || a.seendate || ''
  };});

  var wn = src.WorldNews || {};
  var wnArticles = (wn.articles || []).map(function(a) { return {
    title: a.title || '', url: a.url || '',
    domain: 'WorldNews', date: a.publishDate || ''
  };});
  v.newsArticles = v.newsArticles.concat(wnArticles);

  var rw = src.ReliefWeb || {};
  var rwReports = (rw.latestReports || []).map(function(r) { return {
    title: r.title || '', url: r.url || '',
    domain: 'ReliefWeb', date: r.date || ''
  };});
  v.newsArticles = v.newsArticles.concat(rwReports);

  v.newsCount = v.newsArticles.length;

  var tg = src.Telegram || {};
  v.tgPosts = tg.recentPosts || [];
  v.tgUrgent = tg.urgent || [];
  v.tgPostCount = v.tgPosts.length;

  var bsky = src.Bluesky || {};
  var bskyTopics = bsky.topics || {};
  v.bskyPosts = [];
  for (var bt in bskyTopics) {
    if (!bskyTopics.hasOwnProperty(bt)) continue;
    (bskyTopics[bt] || []).forEach(function(p) {
      v.bskyPosts.push({ channel: 'BSKY/' + bt.toUpperCase(), text: p.text || '', date: p.createdAt || '', views: p.likes || 0 });
    });
  }
  v.bskyPosts = v.bskyPosts.slice(0, 10);

  var reddit = src.Reddit || {};
  var redditSubs = reddit.subreddits || {};
  v.redditPosts = [];
  for (var rs in redditSubs) {
    if (!redditSubs.hasOwnProperty(rs)) continue;
    (redditSubs[rs] || []).forEach(function(p) {
      v.redditPosts.push({ channel: 'r/' + rs, text: p.title || '', date: p.created || '', views: p.score || 0 });
    });
  }
  v.redditPosts = v.redditPosts.slice(0, 10);

  var sat = src.CelesTrak || {};
  v.space = {
    recentLaunches: sat.recentLaunches || [],
    militarySats: sat.militarySats || 0,
    starlink: sat.starlink || sat.starlinkCount || 0,
    totalNewObjects: sat.totalNewObjects || (sat.recentLaunches || []).length,
    oneweb: sat.oneweb || 0
  };

  var yf = src.YFinance || {};
  var quotes = yf.quotes || {};
  v.markets = {};
  var qMap = {
    'SPY':'SPY', 'QQQ':'NASDAQ', '^DJI':'DOW', '^RUT':'RUSSELL',
    'BTC-USD':'BTC', 'ETH-USD':'ETH', '^VIX':'VIX', '^GSPC':'S&P 500'
  };
  for (var sym in quotes) {
    if (!quotes.hasOwnProperty(sym)) continue;
    var data = quotes[sym];
    var label = qMap[sym] || sym;
    v.markets[label] = {
      symbol: sym, name: label,
      price: data.price || 0, change: data.change || 0, changePct: data.changePct || 0
    };
  }

  var fred = src.FRED || {};
  var fredInd = fred.indicators || [];
  v.fred = {};
  for (var fi = 0; fi < fredInd.length; fi++) {
    var ind = fredInd[fi];
    v.fred[ind.id] = {
      id: ind.id, label: ind.label || ind.id,
      value: ind.value, date: ind.date || '', recent: ind.recent || []
    };
  }

  var eia = src.EIA || {};
  var eiaData = eia.data || {};
  var natgasEntry = eiaData.natgas || eiaData.henryHub || eiaData.naturalGas;
  var wtiRecent = eiaData.wti ? (eiaData.wti.recent || []) : [];
  // recent items may be {value, period} objects — extract numeric values
  var wtiRecentVals = wtiRecent.map(function(r) { return typeof r === 'number' ? r : (r && r.value ? r.value : 0); });
  v.energy = {
    wti: eiaData.wti ? eiaData.wti.value : null,
    brent: eiaData.brent ? eiaData.brent.value : null,
    natgas: natgasEntry ? natgasEntry.value : null,
    wtiRecent: wtiRecentVals
  };

  var usgs = src.USGS || {};
  v.quakeCount = usgs.totalQuakes || 0;
  v.maxMagnitude = usgs.maxMagnitude || 0;
  v.quakes = (usgs.quakes || []).map(function(q) { return {
    mag: q.mag || q.magnitude || 0, place: q.place || q.location || '',
    lat: q.lat || q.latitude || 0, lon: q.lon || q.longitude || 0
  };});

  var gdacs = src.GDACS || {};
  v.gdacsAlerts = (gdacs.alerts || []).map(function(a) { return {
    title: a.title || '', alertLevel: a.alertLevel || '', eventType: a.eventType || '',
    country: a.country || '', lat: a.lat || 0, lon: a.lon || 0, date: a.pubDate || ''
  };});
  v.gdacsSummary = gdacs.summary || {};

  var tsunami = src.Tsunami || {};
  v.tsunamiAlerts = (tsunami.alerts || []).map(function(a) { return {
    title: a.title || '', date: a.updated || '', summary: a.summary || ''
  };});
  v.tsunamiWarnings = tsunami.warnings || 0;
  v.tsunamiWatches = tsunami.watches || 0;

  var noaa = src.NOAA || {};
  v.weatherAlerts = (noaa.topAlerts || noaa.alerts || []).map(function(a) { return {
    event: a.event || '', severity: a.severity || '',
    lat: a.lat || 0, lon: a.lon || 0, headline: a.headline || a.description || ''
  };});

  var swpc = src.SWPC || {};
  v.spaceWeather = swpc.current || {};

  v.shipsError = (src.Ships || {}).error || null;

  var nvd = src['NVD-CVE'] || {};
  v.cveCount = nvd.criticalCount || 0;
  v.cveTop = nvd.topVulnerabilities || [];

  var kev = src['CISA-KEV'] || {};
  v.kevRecent = (kev.vulnerabilities || []).map(function(k) { return {
    cveId: k.cveID || '', vendor: k.vendorProject || '', product: k.product || '',
    name: k.vulnerabilityName || '', dateAdded: k.dateAdded || '',
    ransomware: k.knownRansomware === 'Known'
  };});
  v.kevCount = kev.recentAdditions || 0;
  v.kevRansomware = kev.ransomwareLinked || 0;

  var cg = src.CoinGecko || {};
  v.cgCoins = (cg.coins || []).map(function(c) { return {
    name: c.name || '', symbol: c.symbol || '', price: c.price || 0,
    change24h: c.change24h || 0, marketCap: c.marketCap || 0
  };});
  v.cgMarketCap = cg.totalMarketCap || 0;

  v.bls = src.BLS || {};
  v.treasury = src['US Treasury'] || {};
  v.gscpi = src.GSCPI || {};

  // --- Sanctions ---
  var ofac = src.Sanctions || {};
  v.ofac = {
    publishDate: (ofac.ofac || {}).publishDate || '',
    entryCount: (ofac.ofac || {}).entryCount || 0,
    dataAvailable: (ofac.ofac || {}).dataAvailable || false,
    monitoringTargets: ((ofac.openSanctions || {}).monitoringTargets || []).length
  };
  var euSanc = src.EUSanctions || {};
  v.euSanctions = (euSanc.regimes || []).map(function(r) { return {
    name: r.name || '', country: r.country || '', adopted: r.adopted || '', url: r.url || ''
  };});
  v.euSanctionsCount = euSanc.totalRegimes || 0;

  // --- Global Economy ---
  var ecb = src.ECB || {};
  v.ecb = {
    eurUsd: (ecb.eurUsd || {}).rate || 0,
    eurUsdDate: (ecb.eurUsd || {}).date || '',
    euribor: (ecb.euribor3m || {}).rate || 0,
    euriborDate: (ecb.euribor3m || {}).date || ''
  };
  var comtrade = src['UN Comtrade'] || {};
  v.comtrade = (comtrade.tradeFlows || []).map(function(f) { return {
    reporter: f.reporter || '', commodity: f.commodity || '',
    topPartners: f.topPartners || [], totalRecords: f.totalRecords || 0
  };});
  var spending = src.USAspending || {};
  v.spending = {
    contracts: (spending.recentDefenseContracts || []).map(function(c) { return {
      recipient: c.recipient || '', amount: c.amount || 0,
      description: (c.description || '').substring(0, 60), agency: c.agency || '', date: c.date || ''
    };}),
    agencies: (spending.topAgencies || []).map(function(a) { return {
      name: a.name || '', budget: a.budget || 0, obligations: a.obligations || 0
    };})
  };

  // --- Climate & Environment ---
  var copernicus = src.Copernicus || {};
  v.climate = {
    title: copernicus.title || '', period: copernicus.period || '',
    tempAnomaly: copernicus.globalTemperatureAnomaly || null,
    summary: (copernicus.summary || '').substring(0, 200)
  };
  var epaRad = src['EPA-RadNet'] || {};
  v.epaRadnet = {
    totalReadings: epaRad.totalReadings || 0,
    readings: (epaRad.readings || []).slice(0, 10).map(function(r) { return {
      location: r.location || '', state: r.state || '',
      result: r.result || 0, unit: r.unit || '', date: r.collectDate || ''
    };})
  };

  // --- Network Intel ---
  var isc = src['ISC-SANS'] || {};
  v.iscInfocon = {
    status: (isc.infocon || {}).status || 'unknown',
    severity: (isc.infocon || {}).severity || ''
  };
  var ripe = src.RIPEAtlas || {};
  v.ripeAtlas = {
    activeProbes: ripe.activeProbes || 0,
    measurements: (ripe.recentMeasurements || []).slice(0, 5).map(function(m) { return {
      type: m.type || '', description: m.description || '', participants: m.participantCount || 0
    };})
  };

  // --- Transport & Airspace ---
  var ntsb = src.NTSB || {};
  v.ntsb = {
    totalIncidents: ntsb.totalIncidents || 0,
    fatalInjuries: ntsb.fatalInjuries || 0,
    incidents: (ntsb.incidents || []).slice(0, 8).map(function(inc) { return {
      date: inc.date || '', location: inc.location || '', severity: inc.severity || '',
      aircraft: inc.aircraft || '', fatalities: inc.fatalInjuries || 0
    };})
  };
  var adsb = src['ADS-B'] || {};
  v.adsb = {
    totalMilitary: adsb.totalMilitary || 0,
    categories: adsb.categories || {},
    aircraft: (adsb.militaryAircraft || []).slice(0, 8)
  };

  // --- Trends & Innovation ---
  var gtrends = src.GoogleTrends || {};
  v.gtrends = (gtrends.trends || []).slice(0, 10).map(function(t) { return {
    query: t.query || '', traffic: t.traffic || '', date: t.date || ''
  };});
  var patents = src['USPTO-Patents'] || {};
  v.patents = {
    totalFound: patents.totalFound || 0,
    domains: patents.domains || {}
  };

  // --- NEO Tracker ---
  var neo = src['NASA-NEO'] || {};
  v.neo = {
    hazardousCount: neo.hazardousCount || 0,
    totalObjects: neo.elementCount || 0,
    objects: (neo.objects || []).slice(0, 8).map(function(o) { return {
      name: o.name || '', hazardous: o.hazardous || false,
      distanceKm: o.missDistanceKm || 0, velocityKmh: o.velocityKmh || 0,
      diameterMax: o.diameterMaxKm || 0, date: o.closeApproachDate || ''
    };})
  };

  v.delta = raw.delta || {};
  v.correlations = raw.correlations || [];
  v.analysis = (raw.analysis && typeof raw.analysis === 'object' && raw.analysis.text) ? raw.analysis.text : (raw.analysis || null);
  v.errors = raw.errors || [];
  v.timing = raw.timing || {};

  return v;
}
   ================================================================ */
function getAge(d) {
  if (!d) return '';
  var ms = Date.now() - new Date(d).getTime();
  var h = Math.floor(ms / 3600000);
  if (h < 1) return 'just now';
  if (h < 24) return h + 'h ago';
  return Math.floor(h / 24) + 'd ago';
}
function cleanText(t) {
  if (!t) return '';
  return t.replace(/&#39;/g, "'").replace(/&#33;/g, "!").replace(/&amp;/g, "&").replace(/<[^>]+>/g, '');
}
function safeNum(v, fb) {
  var n = parseFloat(v);
  return isNaN(n) ? fb : n;
}
function fmtNum(v, dec) {
  var n = safeNum(v, null);
  if (n === null) return '--';
  return dec !== undefined ? n.toFixed(dec) : n.toLocaleString();
}
function safeExternalUrl(raw) {
  try { var u = new URL(raw, location.href); return (u.protocol === 'http:' || u.protocol === 'https:') ? u.toString() : null; } catch(e) { return null; }
}
function isMobileLayout() { return window.innerWidth <= 1100; }
function fredVal(id) { return S && S.fred && S.fred[id] ? S.fred[id] : null; }
/* ================================================================
   PERFORMANCE TOGGLE
   ================================================================ */
function togglePerfMode() {
  lowPerfMode = !lowPerfMode;
  localStorage.setItem('chaos_low_perf', String(lowPerfMode));
  document.body.classList.toggle('low-perf', lowPerfMode);
  var ps = document.getElementById('perfStatus');
  if (ps) ps.textContent = lowPerfMode ? 'LITE' : 'FULL';
  if (globe) {
    globe.controls().autoRotate = !lowPerfMode;
    globe.arcDashAnimateTime(lowPerfMode ? 0 : 2000);
  }
  updateAllPanels();
}
/* ================================================================
   TOPBAR
   ================================================================ */
function getRegionControlsMarkup() {
  return ['world','americas','europe','middleEast','asiaPacific','africa'].map(function(r) {
    var label = r === 'middleEast' ? 'MIDDLE EAST' : r === 'asiaPacific' ? 'ASIA PACIFIC' : r.toUpperCase();
    return '<button class="region-btn ' + (r === currentRegion ? 'active' : '') + '" data-region="' + r + '" onclick="setRegion(\'' + r + '\')">' + label + '</button>';
  }).join('');
}

function renderTopbar() {
  var mobile = isMobileLayout();
  var ts = new Date(S.meta.timestamp);
  var d = ts.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' }).toUpperCase();
  var timeStr = ts.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: true });

  var vixF = fredVal('VIXCLS');
  var vixVal = (S.markets.VIX ? S.markets.VIX.price : 0) || (vixF ? vixF.value : 0);
  var riskLevel = vixVal > 30 ? 'EXTREME RISK' : vixVal > 20 ? 'HIGH ALERT' : 'ELEVATED';

  var deltaDir = S.delta && S.delta.summary ? S.delta.summary.direction : null;
  var deltaPill = '';
  if (deltaDir) {
    var sym = deltaDir === 'risk-off' ? '&#x25B2; RISK-OFF' : deltaDir === 'risk-on' ? '&#x25BC; RISK-ON' : '&#x25C6; MIXED';
    deltaPill = '<span class="meta-pill">DELTA <span class="v">' + sym + '</span></span>';
  }

  document.getElementById('topbar').innerHTML =
    '<div class="top-left">' +
      '<span class="brand">CHAOS MONITOR</span>' +
      '<span class="regime-chip"><span class="blink"></span>' + riskLevel + '</span>' +
    '</div>' +
    (mobile ? '<div class="top-center">' + getRegionControlsMarkup() + '</div>' : '') +
    '<div class="top-right">' +
      '<button class="meta-pill perf-pill" onclick="togglePerfMode()" title="Toggle visual effects">VISUALS <span class="v" id="perfStatus">' + (lowPerfMode ? 'LITE' : 'FULL') + '</span></button>' +
      '<span class="meta-pill">SWEEP <span class="v">' + (S.meta.totalDurationMs / 1000).toFixed(1) + 's</span></span>' +
      '<span class="meta-pill">' + d + ' <span class="v">' + timeStr + '</span></span>' +
      '<span class="meta-pill">SOURCES <span class="v">' + S.meta.sourcesOk + '/' + S.meta.sourcesQueried + '</span></span>' +
      deltaPill +
      '<button class="guide-btn" onclick="openGlossary()">What Signals Mean</button>' +
      '<span class="alert-badge">' + riskLevel + '</span>' +
      '<button class="gear-btn" onclick="openSettings()" title="Dashboard Settings">\u2699</button>' +
    '</div>';
}
/* ================================================================
   PANEL UPDATE FUNCTIONS
   ================================================================ */
function updateSources() {
  var el = document.getElementById('panel-sources');
  if (!el) return;
  var layers = [
    { name: 'Air Activity', count: S.totalAir, dot: 'air', sub: S.air.length + ' theaters' },
    { name: 'Thermal Spikes', count: S.thermalCount.toLocaleString(), dot: 'thermal', sub: 'FIRMS detections' },
    { name: 'SDR Coverage', count: S.sdrTotal, dot: 'sdr', sub: S.sdrOnline + ' online' },
    { name: 'Maritime Watch', count: S.shipsError ? 0 : '--', dot: 'maritime', sub: 'chokepoints' },
    { name: 'Nuclear Sites', count: S.nuke.length, dot: 'nuke', sub: 'monitors' },
    { name: 'Conflict Events', count: S.conflictEvents, dot: 'incident', sub: S.conflictFatalities + ' fatalities' },
    { name: 'Health Watch', count: S.whoAlerts.length + (S.promedAlerts || []).length, dot: 'health', sub: 'WHO + ProMED alerts' },
    { name: 'World News', count: S.newsCount, dot: 'news', sub: 'GDELT + WorldNews + ReliefWeb' },
    { name: 'OSINT Feed', count: S.tgPostCount + (S.bskyPosts || []).length + (S.redditPosts || []).length, dot: 'osint', sub: 'TG + Bsky + Reddit' },
    { name: 'Satellites', count: S.space.militarySats, dot: 'space', sub: S.space.totalNewObjects + ' new objects' }
  ];
  el.innerHTML = layers.map(function(l) {
    return '<div class="layer-item"><div class="layer-left"><div class="ldot ' + l.dot + '"></div><div><div class="layer-name">' + l.name + '</div><div class="layer-sub">' + l.sub + '</div></div></div><div class="layer-count">' + l.count + '</div></div>';
  }).join('');
}

var noDataHtml = '<div style="display:flex;align-items:center;justify-content:center;height:100%;min-height:60px;color:var(--dim);font-family:var(--mono);font-size:10px;letter-spacing:0.08em;text-transform:uppercase;opacity:0.6">' + L.noData + '</div>';

function updateMarkets() {
  var el = document.getElementById('panel-markets');
  if (!el) return;
  function mktCard(label) {
    var q = S.markets[label];
    if (!q || !q.price) return '';
    var clr = q.changePct >= 0 ? 'var(--gain)' : 'var(--loss)';
    var arrow = q.changePct >= 0 ? '&#9650;' : '&#9660;';
    var priceFmt = q.price >= 1000 ? '$' + q.price.toLocaleString(undefined, { maximumFractionDigits: 0 }) : '$' + q.price.toLocaleString(undefined, { maximumFractionDigits: 2 });
    return '<div class="mc"><div class="ml">' + label + '</div><span class="mv" style="color:' + clr + '">' + priceFmt + '</span><span class="ms" style="color:' + clr + '">' + arrow + ' ' + (q.changePct >= 0 ? '+' : '') + q.changePct.toFixed(2) + '%</span></div>';
  }
  var indexCards = ['SPY', 'NASDAQ', 'DOW', 'RUSSELL', 'S&P 500'].map(mktCard).filter(Boolean).join('');
  var cryptoCards = ['BTC', 'ETH'].map(mktCard).filter(Boolean).join('');
  // CoinGecko top coins (skip BTC/ETH if already shown via YFinance)
  var cgCards = (S.cgCoins || []).filter(function(c) {
    return c.symbol !== 'BTC' && c.symbol !== 'ETH' && c.price > 0;
  }).slice(0, 4).map(function(c) {
    var clr = c.change24h >= 0 ? 'var(--gain)' : 'var(--loss)';
    var arrow = c.change24h >= 0 ? '&#9650;' : '&#9660;';
    var pFmt = c.price >= 1000 ? '$' + c.price.toLocaleString(undefined, { maximumFractionDigits: 0 }) : '$' + c.price.toLocaleString(undefined, { maximumFractionDigits: 2 });
    return '<div class="mc"><div class="ml">' + c.symbol.toUpperCase() + '</div><span class="mv" style="color:' + clr + '">' + pFmt + '</span><span class="ms" style="color:' + clr + '">' + arrow + ' ' + (c.change24h >= 0 ? '+' : '') + c.change24h.toFixed(2) + '%</span></div>';
  }).join('');
  var allCrypto = cryptoCards + cgCards;
  if (!indexCards && !allCrypto) { el.innerHTML = noDataHtml; return; }
  el.innerHTML =
    (indexCards ? '<div style="margin-bottom:8px"><div style="font-family:var(--mono);font-size:9px;color:var(--dim);margin-bottom:4px;letter-spacing:1px">INDEXES</div><div class="metrics-row">' + indexCards + '</div></div>' : '') +
    (allCrypto ? '<div><div style="font-family:var(--mono);font-size:9px;color:var(--dim);margin-bottom:4px;letter-spacing:1px">CRYPTO</div><div class="metrics-row">' + allCrypto + '</div></div>' : '');
}

function updateRiskGauges() {
  var el = document.getElementById('panel-risk');
  if (!el) return;
  var vix = fredVal('VIXCLS');
  var hy = fredVal('BAMLH0A0HYM2');
  var usd = fredVal('DTWEXBGS');
  var claims = fredVal('ICSA');
  var mort = fredVal('MORTGAGE30US');
  var m2 = fredVal('M2SL');
  var natDebt = fredVal('GFDEBTN');
  // Risk-level coloring: low=green, high=red (inverted — higher is worse)
  var vixColor = vix && vix.value > 30 ? 'var(--danger)' : vix && vix.value > 20 ? 'var(--warn)' : 'var(--accent)';
  var hyColor = hy && hy.value > 5 ? 'var(--danger)' : hy && hy.value > 3.5 ? 'var(--warn)' : 'var(--accent)';

  var signalMetrics = [
    { l: 'Incident Tempo', v: S.tgUrgent.length, p: Math.min(S.tgUrgent.length * 10, 100) },
    { l: 'Air Theaters', v: S.air.length, p: Math.min(S.air.length * 10, 100) },
    { l: 'Thermal Spikes', v: S.thermalCount, p: Math.min(S.thermalCount / 5, 100) },
    { l: 'SDR Nodes', v: S.sdrOnline, p: S.sdrTotal > 0 ? Math.round(S.sdrOnline / S.sdrTotal * 100) : 0 },
    { l: 'Quake Events', v: S.quakeCount, p: Math.min(S.quakeCount * 2, 100) },
    { l: 'WHO Alerts', v: S.whoAlerts.length, p: Math.min(S.whoAlerts.length * 10, 100) }
  ];

  el.innerHTML =
    '<div class="econ-row"><span class="elabel">VIX (Fear)</span><span class="eval" style="color:' + vixColor + '">' + fmtNum(vix ? vix.value : null) + '</span></div>' +
    '<div class="econ-row"><span class="elabel">HY Spread</span><span class="eval" style="color:' + hyColor + '">' + fmtNum(hy ? hy.value : null) + '</span></div>' +
    '<div class="econ-row"><span class="elabel">USD Index</span><span class="eval">' + fmtNum(usd ? usd.value : null, 1) + '</span></div>' +
    '<div class="econ-row"><span class="elabel">Jobless Claims</span><span class="eval">' + (claims && claims.value ? claims.value.toLocaleString() : '--') + '</span></div>' +
    '<div class="econ-row"><span class="elabel">30Y Mortgage</span><span class="eval">' + fmtNum(mort ? mort.value : null) + '%</span></div>' +
    '<div class="econ-row"><span class="elabel">M2 Supply</span><span class="eval">$' + (m2 && m2.value ? (m2.value / 1000).toFixed(1) : '--') + 'T</span></div>' +
    '<div class="econ-row"><span class="elabel">Nat. Debt</span><span class="eval">' + (natDebt && natDebt.value ? '$' + (natDebt.value / 1e6).toFixed(2) + 'T' : '--') + '</span></div>' +
    '<div style="margin-top:8px;border-top:1px solid rgba(255,255,255,0.06);padding-top:8px">' +
    signalMetrics.map(function(s) {
      return '<div class="sm"><span class="sml">' + s.l + '</span><div class="smb"><span style="width:' + s.p + '%"></span></div><span class="smv">' + s.v + '</span></div>';
    }).join('') +
    '</div>';
}

function updateEnergy() {
  var el = document.getElementById('panel-energy');
  if (!el) return;
  if (!S.energy.wti && !S.energy.brent && !S.energy.natgas) { el.innerHTML = noDataHtml; return; }
  var ff = fredVal('DFF');
  var vixF = fredVal('VIXCLS');
  var vixLive = S.markets.VIX;
  var vixVal = (vixLive ? vixLive.price : 0) || (vixF ? vixF.value : 0);
  var gscpi = S.gscpi;

  var wtiH = S.energy.wtiRecent || [];
  var wtiMax = wtiH.length > 0 ? Math.max.apply(null, wtiH) : 1;
  var wtiMin = wtiH.length > 0 ? Math.min.apply(null, wtiH) : 0;
  var sparkHtml = wtiH.map(function(v) {
    var pct = wtiMax === wtiMin ? 50 : ((v - wtiMin) / (wtiMax - wtiMin)) * 100;
    return '<div class="spark-bar" style="height:' + Math.max(pct, 8) + '%"></div>';
  }).join('');

  // VIX risk coloring: low=green, high=red (higher is worse)
  var vixColor = vixVal > 30 ? 'var(--danger)' : vixVal > 20 ? 'var(--warn)' : 'var(--accent)';
  var metrics = [
    { l: 'WTI Crude', v: S.energy.wti ? '$' + S.energy.wti : '--', s: '$/bbl', p: 70, c: '' },
    { l: 'Brent', v: S.energy.brent ? '$' + S.energy.brent : '--', s: '$/bbl', p: 75, c: '' },
    { l: 'Nat Gas', v: S.energy.natgas ? '$' + S.energy.natgas : '--', s: '$/MMBtu', p: 30, c: '' },
    { l: 'VIX', v: vixVal ? vixVal.toFixed(1) : '--', s: 'volatility index', p: vixVal ? Math.min(vixVal * 2.5, 100) : 30, c: vixColor },
    { l: 'Fed Funds', v: ff ? ff.value + '%' : '--', s: ff ? ff.date : '', p: 36, c: '' },
    { l: 'GSCPI', v: gscpi && gscpi.value !== undefined ? Number(gscpi.value).toFixed(2) : '--', s: gscpi && gscpi.interpretation ? gscpi.interpretation : '', p: 49, c: '' }
  ];

  el.innerHTML =
    '<div class="metrics-row">' +
    metrics.map(function(m) { return '<div class="mc"><div class="ml">' + m.l + '</div><span class="mv"' + (m.c ? ' style="color:' + m.c + '"' : '') + '>' + m.v + '</span><span class="ms">' + m.s + '</span><div class="mbar"><span style="width:' + m.p + '%"></span></div></div>'; }).join('') +
    '</div>' +
    (wtiH.length > 0 ? '<div style="margin-top:6px"><div style="font-family:var(--mono);font-size:10px;color:var(--dim);margin-bottom:4px">WTI RECENT</div><div class="spark">' + sparkHtml + '</div></div>' : '');
}

function updateNewsTicker() {
  var el = document.getElementById('panel-news');
  if (!el) return;
  var feed = S.newsArticles.slice(0, 20);
  if (feed.length === 0) { el.innerHTML = noDataHtml; return; }
  var tickerCards = feed.map(function(n) {
    var urlAttr = n.url ? ' data-url="' + String(n.url).replace(/&/g, '&amp;').replace(/"/g, '&quot;') + '"' : '';
    var age = n.date ? getAge(n.date) : '';
    return '<div class="tk-card ' + (n.url ? 'clickable' : '') + '"' + urlAttr + '><span class="tk-src gdelt">' + (n.domain || 'GDELT').substring(0, 12) + '</span><span class="tk-time">' + age + '</span><div class="tk-head">' + cleanText(n.title) + '</div>' + (n.url ? '<span class="tk-link">&#8599;</span>' : '') + '</div>';
  }).join('');
  var tickerDuration = Math.max(20, feed.length * 2.5);
  el.innerHTML =
    '<div class="ticker-wrap" style="--ticker-duration:' + tickerDuration + 's">' +
      '<div class="ticker-track">' + tickerCards + (lowPerfMode ? '' : tickerCards) + '</div>' +
    '</div>';
}

function updateOsint() {
  var el = document.getElementById('panel-osint');
  if (!el) return;
  var allPosts = S.tgUrgent.concat(S.tgPosts).concat(S.bskyPosts || []).concat(S.redditPosts || []).slice(0, 15);
  if (allPosts.length === 0 && S.whoAlerts.length === 0 && (S.promedAlerts || []).length === 0) { el.innerHTML = noDataHtml; return; }
  var whoItems = S.whoAlerts.slice(0, 4).map(function(w) { return { channel: 'WHO ALERT', text: w.title, date: w.date, isWho: true }; });
  var promedItems = (S.promedAlerts || []).slice(0, 4).map(function(p) { return { channel: 'PROMED', text: p.title, date: p.date, isWho: true }; });
  var osintItems = allPosts.concat(whoItems).concat(promedItems);
  var osintCards = osintItems.map(function(p) {
    var views = p.views ? (p.views >= 1000 ? (p.views / 1000).toFixed(0) + 'K' : '' + p.views) : '';
    var age = p.date ? getAge(p.date) : '';
    var srcCls = p.isWho ? 'style="color:#69f0ae;border-color:rgba(105,240,174,0.4)"' : 'class="tk-src tg"';
    var isU = p.urgentFlags && p.urgentFlags.length > 0;
    return '<div class="tk-card ' + (isU ? 'urgent' : '') + '"><span ' + srcCls + '>' + (p.channel || 'OSINT').toUpperCase().substring(0, 14) + '</span>' + (views ? '<span class="tk-src other">' + views + '</span>' : '') + '<span class="tk-time">' + age + '</span><div class="tk-head">' + cleanText((p.text || '').substring(0, 160)) + '</div></div>';
  }).join('');
  var osintDuration = Math.max(25, osintItems.length * 3);
  el.innerHTML =
    '<div class="ticker-wrap" style="--ticker-duration:' + osintDuration + 's;max-height:100%">' +
      '<div class="ticker-track">' + osintCards + (lowPerfMode ? '' : osintCards) + '</div>' +
    '</div>';
}

function updateDelta() {
  var el = document.getElementById('panel-delta');
  if (!el) return;
  var delta = S.delta || {};
  var ds = delta.summary || {};
  var hasDelta = (ds.totalChanges || 0) > 0;
  var dirMap = { 'risk-off': '&#9650;', 'risk-on': '&#9660;', 'mixed': '&#9670;' };
  var dirEmoji = dirMap[ds.direction] || '&#9670;';
  var dirClassMap = { 'risk-off': 'up', 'risk-on': 'down', 'mixed': '' };
  var dirClass = dirClassMap[ds.direction] || '';
  var escalated = (delta.signals && delta.signals.escalated || []).slice(0, 6);
  var deescalated = (delta.signals && delta.signals.deescalated || []).slice(0, 4);
  var newSigs = (delta.signals && delta.signals['new'] || []).slice(0, 4);
  var deltaRows = [];
  newSigs.forEach(function(s) {
    deltaRows.push('<div class="delta-row new"><span class="delta-badge new">NEW</span><span class="delta-label">' + (s.reason || s.label || s.key || '') + '</span></div>');
  });
  escalated.forEach(function(s) {
    var sev = s.severity === 'critical' ? 'style="color:var(--warn);font-weight:600"' : s.severity === 'high' ? 'style="color:#ffab40"' : '';
    var val = s.pctChange !== undefined ? (s.pctChange > 0 ? '+' : '') + s.pctChange + '%' : (s.change !== undefined ? ((s.change > 0 ? '+' : '') + s.change) : '');
    deltaRows.push('<div class="delta-row"><span class="delta-badge up">&#9650;</span><span class="delta-label" ' + sev + '>' + (s.label || '') + '</span><span class="delta-val">' + (s.from || '') + (s.to ? ' -> ' + s.to : '') + (val ? ' (' + val + ')' : '') + '</span></div>');
  });
  deescalated.forEach(function(s) {
    var val = s.pctChange !== undefined ? s.pctChange + '%' : (s.change !== undefined ? '' + s.change : '');
    deltaRows.push('<div class="delta-row"><span class="delta-badge down">&#9660;</span><span class="delta-label">' + (s.label || s.key || '') + '</span><span class="delta-val">' + (s.from || '') + (s.to ? ' -> ' + s.to : '') + (val ? ' (' + val + ')' : '') + '</span></div>');
  });
  var deltaHtml = hasDelta ? deltaRows.join('') : '<div style="padding:12px;text-align:center;color:var(--dim);font-family:var(--mono);font-size:10px">No changes since last sweep</div>';

  el.innerHTML =
    (hasDelta ? '<div style="display:flex;gap:12px;margin-bottom:6px;font-family:var(--mono);font-size:10px">' +
      '<span style="color:var(--dim)">Direction: <span class="delta-badge ' + dirClass + '">' + dirEmoji + ' ' + (ds.direction ? ds.direction.toUpperCase() : 'BASELINE') + '</span></span>' +
      '<span style="color:var(--dim)">Changes: <span style="color:var(--accent)">' + ds.totalChanges + '</span></span>' +
      '<span style="color:var(--dim)">Critical: <span style="color:' + ((ds.criticalChanges || 0) > 0 ? 'var(--warn)' : 'var(--dim)') + '">' + (ds.criticalChanges || 0) + '</span></span>' +
    '</div>' : '') +
    '<div class="delta-list">' + deltaHtml + '</div>';
}

function updateAnalysis() {
  var el = document.getElementById('panel-analysis');
  if (!el) return;
  var hasAnalysis = S.analysis && typeof S.analysis === 'string' && S.analysis.length > 10;
  if (hasAnalysis) {
    var paragraphs = S.analysis.split('\n').filter(function(l) { return l.trim(); }).map(function(l) { return '<div class="idea-text" style="margin-bottom:6px">' + cleanText(l) + '</div>'; }).join('');
    el.innerHTML = '<div class="idea-card"><span class="idea-type watch">AI ANALYSIS</span>' + paragraphs + '</div>' +
      '<div class="disclosure">FOR INFORMATIONAL PURPOSES ONLY. This is not financial advice, a recommendation to buy or sell any security, or a solicitation of any kind. All signal-based observations are derived from publicly available OSINT data and should not be relied upon for investment decisions.</div>';
  } else {
    el.innerHTML = '<div style="padding:20px;text-align:center;color:var(--dim);font-family:var(--mono);font-size:11px">' +
      '<div style="font-size:24px;margin-bottom:8px;opacity:0.3">&#9888;</div>' +
      '<div>LLM NOT CONFIGURED</div>' +
      '<div style="font-size:9px;margin-top:6px;opacity:0.6">Set LLM_PROVIDER + credentials in .env to enable AI-powered analysis</div>' +
    '</div>';
  }
}

function updateSignals() {
  var el = document.getElementById('panel-signals');
  if (!el) return;
  var signals = (S.correlations || []).slice(0, 8).map(function(sig, i) {
    var sevClass = sig.severity === 'critical' ? 'critical' : sig.severity === 'high' ? 'high' : sig.severity === 'medium' ? 'medium' : '';
    return '<div class="signal-row ' + sevClass + '"><strong>' + (sig.name || 'Signal ' + (i + 1)) + '</strong><p>' + (sig.description || '') + '</p></div>';
  }).join('');
  el.innerHTML = signals || '<div style="padding:12px;text-align:center;color:var(--dim);font-family:var(--mono);font-size:10px">No cross-source correlations detected</div>';
}

function updateQuakes() {
  var el = document.getElementById('panel-quakes');
  if (!el) return;
  var tsunamiTag = (S.tsunamiWarnings > 0) ? ' | <span style="color:var(--danger)">TSUNAMI WARNING: ' + S.tsunamiWarnings + '</span>' : (S.tsunamiWatches > 0 ? ' | <span style="color:var(--warn)">TSUNAMI WATCH: ' + S.tsunamiWatches + '</span>' : '');
  var html = '<div style="font-family:var(--mono);font-size:10px;color:var(--dim);margin-bottom:6px">QUAKES: <span style="color:var(--accent)">' + S.quakeCount + '</span> | MAX: <span style="color:var(--accent)">' + S.maxMagnitude.toFixed(1) + '</span>' + tsunamiTag + '</div>';
  html += S.quakes.slice(0, 6).map(function(q) {
    var magColor = q.mag >= 6 ? 'var(--danger)' : q.mag >= 4.5 ? 'var(--warn)' : 'var(--accent2)';
    return '<div class="econ-row"><span class="elabel">' + q.place.substring(0, 35) + '</span><span class="eval" style="color:' + magColor + '">M' + q.mag.toFixed(1) + '</span></div>';
  }).join('');
  var gdacsRed = (S.gdacsAlerts || []).filter(function(a) { return (a.alertLevel || '').toLowerCase() === 'red'; });
  var gdacsOrange = (S.gdacsAlerts || []).filter(function(a) { return (a.alertLevel || '').toLowerCase() === 'orange'; });
  var gdacsShow = gdacsRed.concat(gdacsOrange).slice(0, 4);
  if (gdacsShow.length > 0) {
    html += '<div style="font-family:var(--mono);font-size:9px;color:var(--dim);margin:6px 0 4px;letter-spacing:1px">GDACS ALERTS</div>';
    html += gdacsShow.map(function(a) {
      var lvlColor = (a.alertLevel || '').toLowerCase() === 'red' ? 'var(--danger)' : 'var(--warn)';
      return '<div class="econ-row"><span class="elabel">' + (a.eventType || 'Event') + ' - ' + (a.country || '').substring(0, 20) + '</span><span class="eval" style="color:' + lvlColor + '">' + (a.alertLevel || '').toUpperCase() + '</span></div>';
    }).join('');
  }
  el.innerHTML = html;
}

function updateNuclear() {
  var el = document.getElementById('panel-nuclear');
  if (!el) return;
  var allNormal = S.nuke.every(function(s) { return !s.anomaly; });
  var nukeHtml = '<div class="' + (allNormal ? 'nuke-ok' : 'nuke-warn') + '">' + (allNormal ? '&#9679; ALL SITES NORMAL' : '&#9888; ANOMALY DETECTED') + '</div>';
  nukeHtml += S.nuke.map(function(s) {
    return '<div class="site-row"><span>' + s.site + '</span><span class="site-val">' + (s.cpm ? s.cpm.toFixed(1) + ' CPM' : 'No data') + '</span></div>';
  }).join('');
  el.innerHTML = nukeHtml;
}

function updateCyber() {
  var el = document.getElementById('panel-cyber');
  if (!el) return;
  var html = '<div style="font-family:var(--mono);font-size:10px;color:var(--dim);margin-bottom:6px">CRITICAL CVEs: <span style="color:' + (S.cveCount > 0 ? 'var(--danger)' : 'var(--accent)') + '">' + S.cveCount + '</span>' +
    (S.kevCount > 0 ? ' | KEV: <span style="color:var(--warn)">' + S.kevCount + '</span>' : '') +
    (S.kevRansomware > 0 ? ' | RANSOMWARE: <span style="color:var(--danger)">' + S.kevRansomware + '</span>' : '') +
    '</div>';
  html += S.cveTop.slice(0, 5).map(function(c) {
    var sevColor = (c.severity || '').toLowerCase() === 'critical' ? 'var(--danger)' : 'var(--warn)';
    return '<div class="econ-row"><span class="elabel" style="flex:1;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">' + (c.id || c.cveId || '') + '</span><span class="eval" style="color:' + sevColor + '">' + (c.severity || c.score || '') + '</span></div>';
  }).join('');
  if (S.kevRecent && S.kevRecent.length > 0) {
    html += '<div style="font-family:var(--mono);font-size:9px;color:var(--dim);margin:6px 0 4px;letter-spacing:1px">CISA KEV (7d)</div>';
    html += S.kevRecent.slice(0, 4).map(function(k) {
      var rw = k.ransomware ? ' style="color:var(--danger)"' : '';
      return '<div class="econ-row"><span class="elabel" style="flex:1;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">' + k.cveId + ' ' + k.vendor + '</span><span class="eval"' + rw + '>' + (k.ransomware ? 'RANSOM' : k.product) + '</span></div>';
    }).join('');
  }
  if (S.cveTop.length === 0 && (!S.kevRecent || S.kevRecent.length === 0)) html += '<div style="text-align:center;color:var(--dim);font-family:var(--mono);font-size:10px;padding:8px">No critical vulnerabilities</div>';
  el.innerHTML = html;
}

function updateSpace() {
  var el = document.getElementById('panel-space');
  if (!el) return;
  if (!S.space.totalNewObjects && !S.space.militarySats) { el.innerHTML = noDataHtml; return; }
  var swR = S.spaceWeather.R || {};
  var swS = S.spaceWeather.S || {};
  var swG = S.spaceWeather.G || {};
  el.innerHTML =
    '<div class="econ-row"><span class="elabel">New Objects (30d)</span><span class="eval" style="color:var(--accent2)">' + S.space.totalNewObjects + '</span></div>' +
    '<div class="econ-row"><span class="elabel">Military Sats</span><span class="eval">' + S.space.militarySats + '</span></div>' +
    '<div class="econ-row"><span class="elabel">Starlink</span><span class="eval">' + S.space.starlink + '</span></div>' +
    '<div class="econ-row"><span class="elabel">OneWeb</span><span class="eval">' + S.space.oneweb + '</span></div>' +
    (swR.Scale !== undefined ? '<div class="econ-row"><span class="elabel">Radio Blackout (R)</span><span class="eval">' + (swR.Scale || 'R0') + '</span></div>' : '') +
    (swS.Scale !== undefined ? '<div class="econ-row"><span class="elabel">Solar Radiation (S)</span><span class="eval">' + (swS.Scale || 'S0') + '</span></div>' : '') +
    (swG.Scale !== undefined ? '<div class="econ-row"><span class="elabel">Geomagnetic (G)</span><span class="eval">' + (swG.Scale || 'G0') + '</span></div>' : '');
}

function updateConflicts() {
  var el = document.getElementById('panel-conflicts');
  if (!el) return;
  var html = '<div style="font-family:var(--mono);font-size:10px;color:var(--dim);margin-bottom:6px">EVENTS: <span style="color:var(--danger)">' + S.conflictEvents + '</span> | FATALITIES: <span style="color:var(--danger)">' + S.conflictFatalities + '</span></div>';
  html += S.acledEvents.slice(0, 10).map(function(e) {
    var fatalities = e.fatalities || 0;
    var fColor = fatalities > 10 ? 'var(--danger)' : fatalities > 0 ? 'var(--warn)' : 'var(--dim)';
    return '<div class="econ-row"><span class="elabel" style="flex:1;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">' + (e.type || e.event_type || 'Conflict') + ' - ' + (e.country || e.location || '') + '</span><span class="eval" style="color:' + fColor + '">' + fatalities + ' KIA</span></div>';
  }).join('');
  if (S.acledEvents.length === 0) html += '<div style="text-align:center;color:var(--dim);font-family:var(--mono);font-size:10px;padding:8px">No conflict events</div>';
  el.innerHTML = html;
}

function updateSanctions() {
  var el = document.getElementById('panel-sanctions');
  if (!el) return;
  var html = '<div style="font-family:var(--mono);font-size:10px;color:var(--dim);margin-bottom:6px">OFAC ENTRIES: <span style="color:var(--accent)">' + (S.ofac.entryCount || 0).toLocaleString() + '</span> | EU REGIMES: <span style="color:var(--accent)">' + S.euSanctionsCount + '</span></div>';
  if (S.ofac.publishDate) html += '<div class="econ-row"><span class="elabel">OFAC Last Updated</span><span class="eval">' + S.ofac.publishDate + '</span></div>';
  if (S.ofac.monitoringTargets) html += '<div class="econ-row"><span class="elabel">OpenSanctions Targets</span><span class="eval">' + S.ofac.monitoringTargets + '</span></div>';
  html += (S.euSanctions || []).slice(0, 6).map(function(r) {
    return '<div class="econ-row"><span class="elabel" style="flex:1;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">' + r.name.substring(0, 40) + '</span><span class="eval">' + (r.country || r.adopted || '') + '</span></div>';
  }).join('');
  if (!S.ofac.entryCount && !S.euSanctionsCount) html = noDataHtml;
  el.innerHTML = html;
}

function updateEconomy() {
  var el = document.getElementById('panel-economy');
  if (!el) return;
  var html = '';
  if (S.ecb.eurUsd) html += '<div class="econ-row"><span class="elabel">EUR/USD</span><span class="eval" style="color:var(--accent)">' + S.ecb.eurUsd.toFixed(4) + '</span></div>';
  if (S.ecb.euribor) html += '<div class="econ-row"><span class="elabel">EURIBOR 3M</span><span class="eval">' + S.ecb.euribor.toFixed(3) + '%</span></div>';
  if (S.spending.agencies && S.spending.agencies.length > 0) {
    html += '<div style="font-family:var(--mono);font-size:9px;color:var(--dim);margin:6px 0 4px;letter-spacing:1px">TOP AGENCIES</div>';
    html += S.spending.agencies.slice(0, 4).map(function(a) {
      var b = a.budget >= 1e9 ? '$' + (a.budget / 1e9).toFixed(0) + 'B' : a.budget >= 1e6 ? '$' + (a.budget / 1e6).toFixed(0) + 'M' : '$' + a.budget;
      return '<div class="econ-row"><span class="elabel" style="flex:1;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">' + a.name.substring(0, 30) + '</span><span class="eval">' + b + '</span></div>';
    }).join('');
  }
  if (S.comtrade && S.comtrade.length > 0) {
    html += '<div style="font-family:var(--mono);font-size:9px;color:var(--dim);margin:6px 0 4px;letter-spacing:1px">TRADE FLOWS</div>';
    html += S.comtrade.slice(0, 3).map(function(f) {
      return '<div class="econ-row"><span class="elabel" style="flex:1;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">' + f.commodity.substring(0, 30) + '</span><span class="eval">' + f.totalRecords + ' rec</span></div>';
    }).join('');
  }
  if (!html) html = noDataHtml;
  el.innerHTML = html;
}

function updateClimate() {
  var el = document.getElementById('panel-climate');
  if (!el) return;
  var html = '';
  if (S.climate.tempAnomaly !== null && S.climate.tempAnomaly !== undefined) {
    var aColor = S.climate.tempAnomaly > 1.5 ? 'var(--danger)' : S.climate.tempAnomaly > 1.0 ? 'var(--warn)' : 'var(--accent)';
    html += '<div style="font-family:var(--mono);font-size:10px;color:var(--dim);margin-bottom:6px">TEMP ANOMALY: <span style="color:' + aColor + '">+' + S.climate.tempAnomaly.toFixed(2) + '\u00B0C</span></div>';
  }
  if (S.climate.period) html += '<div class="econ-row"><span class="elabel">Period</span><span class="eval">' + S.climate.period + '</span></div>';
  if (S.climate.summary) html += '<div style="font-family:var(--mono);font-size:9px;color:var(--dim);margin:4px 0;line-height:1.4">' + S.climate.summary.substring(0, 150) + '</div>';
  if (S.epaRadnet.totalReadings > 0) {
    html += '<div style="font-family:var(--mono);font-size:9px;color:var(--dim);margin:6px 0 4px;letter-spacing:1px">EPA RADNET (' + S.epaRadnet.totalReadings + ' readings)</div>';
    html += S.epaRadnet.readings.slice(0, 4).map(function(r) {
      return '<div class="econ-row"><span class="elabel">' + r.location.substring(0, 20) + ', ' + r.state + '</span><span class="eval">' + r.result + ' ' + r.unit + '</span></div>';
    }).join('');
  }
  if (!html) html = noDataHtml;
  el.innerHTML = html;
}

function updateNetwork() {
  var el = document.getElementById('panel-network');
  if (!el) return;
  var infoconColors = { green: 'var(--accent)', yellow: 'var(--warn)', orange: '#ffab40', red: 'var(--danger)' };
  var ic = S.iscInfocon || {};
  var icColor = infoconColors[(ic.status || '').toLowerCase()] || 'var(--dim)';
  var html = '<div style="font-family:var(--mono);font-size:10px;color:var(--dim);margin-bottom:6px">INFOCON: <span style="color:' + icColor + '">' + (ic.status || 'N/A').toUpperCase() + '</span>' +
    (S.ripeAtlas.activeProbes ? ' | RIPE PROBES: <span style="color:var(--accent)">' + S.ripeAtlas.activeProbes.toLocaleString() + '</span>' : '') + '</div>';
  html += (S.ripeAtlas.measurements || []).map(function(m) {
    return '<div class="econ-row"><span class="elabel" style="flex:1;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">' + (m.type || '') + ': ' + m.description.substring(0, 30) + '</span><span class="eval">' + m.participants + '</span></div>';
  }).join('');
  if (!ic.status && !S.ripeAtlas.activeProbes) html = noDataHtml;
  el.innerHTML = html;
}

function updateTransport() {
  var el = document.getElementById('panel-transport');
  if (!el) return;
  var html = '<div style="font-family:var(--mono);font-size:10px;color:var(--dim);margin-bottom:6px">';
  if (S.ntsb.totalIncidents) html += 'INCIDENTS: <span style="color:var(--warn)">' + S.ntsb.totalIncidents + '</span>';
  if (S.ntsb.fatalInjuries) html += ' | FATAL: <span style="color:var(--danger)">' + S.ntsb.fatalInjuries + '</span>';
  if (S.adsb.totalMilitary) html += ' | MIL AIRCRAFT: <span style="color:var(--accent)">' + S.adsb.totalMilitary + '</span>';
  html += '</div>';
  html += S.ntsb.incidents.slice(0, 4).map(function(i) {
    var sColor = i.fatalities > 0 ? 'var(--danger)' : 'var(--warn)';
    return '<div class="econ-row"><span class="elabel" style="flex:1;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">' + (i.aircraft || 'Aircraft') + ' - ' + i.location.substring(0, 20) + '</span><span class="eval" style="color:' + sColor + '">' + (i.fatalities > 0 ? i.fatalities + ' KIA' : i.severity || 'Incident') + '</span></div>';
  }).join('');
  var cats = S.adsb.categories || {};
  var catKeys = Object.keys(cats);
  if (catKeys.length > 0) {
    html += '<div style="font-family:var(--mono);font-size:9px;color:var(--dim);margin:6px 0 4px;letter-spacing:1px">MIL AIRCRAFT</div>';
    html += catKeys.slice(0, 4).map(function(k) {
      return '<div class="econ-row"><span class="elabel">' + k + '</span><span class="eval">' + (typeof cats[k] === 'number' ? cats[k] : (cats[k] || []).length) + '</span></div>';
    }).join('');
  }
  if (!S.ntsb.totalIncidents && !S.adsb.totalMilitary) html = noDataHtml;
  el.innerHTML = html;
}

function updateTrends() {
  var el = document.getElementById('panel-trends');
  if (!el) return;
  var html = '';
  if (S.gtrends && S.gtrends.length > 0) {
    html += '<div style="font-family:var(--mono);font-size:9px;color:var(--dim);margin-bottom:4px;letter-spacing:1px">GOOGLE TRENDS</div>';
    html += S.gtrends.slice(0, 6).map(function(t) {
      return '<div class="econ-row"><span class="elabel" style="flex:1;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">' + t.query.substring(0, 30) + '</span><span class="eval">' + (t.traffic || '') + '</span></div>';
    }).join('');
  }
  var domains = S.patents.domains || {};
  var domainKeys = Object.keys(domains);
  if (domainKeys.length > 0) {
    html += '<div style="font-family:var(--mono);font-size:9px;color:var(--dim);margin:6px 0 4px;letter-spacing:1px">PATENT DOMAINS (' + S.patents.totalFound + ' total)</div>';
    html += domainKeys.slice(0, 5).map(function(k) {
      return '<div class="econ-row"><span class="elabel">' + k.toUpperCase() + '</span><span class="eval">' + domains[k] + '</span></div>';
    }).join('');
  }
  if (!html) html = noDataHtml;
  el.innerHTML = html;
}

function updateAsteroids() {
  var el = document.getElementById('panel-asteroids');
  if (!el) return;
  var html = '<div style="font-family:var(--mono);font-size:10px;color:var(--dim);margin-bottom:6px">NEAR EARTH OBJECTS: <span style="color:var(--accent)">' + S.neo.totalObjects + '</span> | HAZARDOUS: <span style="color:' + (S.neo.hazardousCount > 0 ? 'var(--danger)' : 'var(--accent)') + '">' + S.neo.hazardousCount + '</span></div>';
  html += S.neo.objects.slice(0, 6).map(function(o) {
    var distFmt = o.distanceKm >= 1e6 ? (o.distanceKm / 1e6).toFixed(1) + 'M km' : (o.distanceKm / 1e3).toFixed(0) + 'K km';
    var hColor = o.hazardous ? 'var(--danger)' : 'var(--dim)';
    return '<div class="econ-row"><span class="elabel" style="flex:1;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">' + o.name.substring(0, 25) + '</span><span class="eval" style="color:' + hColor + '">' + distFmt + '</span></div>';
  }).join('');
  if (!S.neo.totalObjects) html = noDataHtml;
  el.innerHTML = html;
}

function updateGlobe() {
  var el = document.getElementById('panel-globe');
  if (!el) return;
  if (!el.querySelector('#mapContainer')) {
    el.innerHTML =
      '<div class="map-region-bar" id="mapRegionBar"></div>' +
      '<div class="map-container" id="mapContainer">' +
        '<div id="globeViz"></div>' +
        '<svg id="flatMapSvg" style="display:none;width:100%;height:100%;position:absolute;top:0;left:0;cursor:grab"></svg>' +
        '<div class="map-loading" id="mapLoading"><div class="map-loading-card"><div class="map-loading-ring"></div><div class="map-loading-text" id="mapLoadingText">Initializing 3D Globe</div></div></div>' +
        '<div class="map-legend" id="mapLegend"></div>' +
        '<div class="map-hint" id="mapHint">SCROLL TO ZOOM &middot; DRAG TO PAN</div>' +
        '<div class="map-controls">' +
          '<button class="map-ctrl-btn" onclick="mapZoom(1.5)" title="Zoom in">+</button>' +
          '<button class="map-ctrl-btn" onclick="mapZoom(0.67)" title="Zoom out">&minus;</button>' +
          '<button class="map-ctrl-btn map-toggle" id="flightToggle" onclick="toggleFlights()" title="Toggle flight routes">&#9992;</button>' +
        '</div>' +
        '<button class="proj-toggle" id="projToggle" onclick="toggleMapMode()">GLOBE MODE</button>' +
        '<div class="map-popup" id="mapPopup"><button class="pp-close" onclick="closePopup()">&times;</button><div class="pp-head"></div><div class="pp-text"></div><div class="pp-meta"></div></div>' +
      '</div>';
  }
  renderRegionControls();
}

function updateAllPanels() {
  if (!S) return;
  updateSources();
  updateMarkets();
  updateRiskGauges();
  updateEnergy();
  updateNewsTicker();
  updateOsint();
  updateDelta();
  updateAnalysis();
  updateSignals();
  updateQuakes();
  updateNuclear();
  updateCyber();
  updateSpace();
  updateConflicts();
  updateSanctions();
  updateEconomy();
  updateClimate();
  updateNetwork();
  updateTransport();
  updateTrends();
  updateAsteroids();
}

/* ================================================================
   SPARKLINE SVG GENERATOR
   ================================================================ */
function mkSparkSvg(values, isGood) {
  if (!values || values.length < 2) return '';
  var w = 52, h = 18, pad = 2;
  var min = Math.min.apply(null, values), max = Math.max.apply(null, values);
  var range = max - min || 1;
  var pts = values.map(function(v, i) {
    var x = pad + (i / (values.length - 1)) * (w - pad * 2);
    var y = pad + ((max - v) / range) * (h - pad * 2);
    return x.toFixed(1) + ',' + y.toFixed(1);
  });
  var cls = isGood ? 'spark-good' : 'spark-bad';
  var last = pts[pts.length - 1];
  var lastParts = last.split(',');
  return '<svg class="spark-svg" viewBox="0 0 ' + w + ' ' + h + '"><polyline class="spark-line ' + cls + '" points="' + pts.join(' ') + '"/><circle class="' + cls + ' spark-dot" cx="' + lastParts[0] + '" cy="' + lastParts[1] + '" r="2" fill="' + (isGood ? 'var(--accent)' : 'var(--danger)') + '"/></svg>';
}
var _srcMonitorCache = null;
var _srcMonitorTs = 0;

function updateSourceMonitor() {
  var el = document.getElementById('panel-srcmonitor');
  if (!el) return;
  var now = Date.now();
  // Fetch source health from API (cache 60s)
  if (!_srcMonitorCache || now - _srcMonitorTs > 60000) {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', '/api/v1/sources', false); // sync for simplicity in panel update
    try {
      xhr.send();
      if (xhr.status === 200) {
        _srcMonitorCache = JSON.parse(xhr.responseText);
        _srcMonitorTs = now;
      }
    } catch(e) { /* ignore */ }
  }
  if (!_srcMonitorCache || !Array.isArray(_srcMonitorCache)) { el.innerHTML = noDataHtml; return; }
  var sources = _srcMonitorCache.slice().sort(function(a, b) { return (a.reliability || 0) - (b.reliability || 0); });
  var html = '<div style="font-family:var(--mono);font-size:9px;color:var(--dim);margin-bottom:6px;display:flex;justify-content:space-between"><span>SOURCE</span><span>RATE / LATENCY</span></div>';
  html += sources.map(function(s) {
    var rate = s.reliability != null ? Math.round(s.reliability * 100) : 0;
    var latency = s.avgDurationMs || 0;
    var barColor = rate >= 90 ? 'var(--gain)' : rate >= 60 ? '#f59e0b' : 'var(--loss)';
    var tierBadge = s.tier ? '<span style="font-size:8px;color:var(--dim);margin-left:4px">T' + s.tier + '</span>' : '';
    var latencyStr = latency > 0 ? (latency > 1000 ? (latency/1000).toFixed(1) + 's' : latency + 'ms') : '--';
    return '<div style="display:flex;align-items:center;gap:6px;margin-bottom:3px;font-size:10px">' +
      '<div style="flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-family:var(--mono)">' + s.name + tierBadge + '</div>' +
      '<div style="width:60px;height:4px;background:rgba(255,255,255,0.08);border-radius:2px;flex-shrink:0">' +
        '<div style="width:' + rate + '%;height:100%;background:' + barColor + ';border-radius:2px"></div>' +
      '</div>' +
      '<div style="width:35px;text-align:right;font-family:var(--mono);font-size:9px;color:' + barColor + '">' + rate + '%</div>' +
      '<div style="width:35px;text-align:right;font-family:var(--mono);font-size:9px;color:var(--dim)">' + latencyStr + '</div>' +
    '</div>';
  }).join('');
  // Summary stats
  var total = sources.length;
  var healthy = sources.filter(function(s) { return (s.reliability || 0) >= 0.8; }).length;
  var avgRate = sources.reduce(function(sum, s) { return sum + (s.reliability || 0); }, 0) / (total || 1);
  html = '<div style="display:flex;gap:12px;margin-bottom:8px;padding-bottom:6px;border-bottom:1px solid rgba(255,255,255,0.06)">' +
    '<div style="text-align:center"><div style="font-size:16px;font-weight:700;font-family:var(--mono);color:var(--gain)">' + healthy + '/' + total + '</div><div style="font-size:8px;color:var(--dim)">HEALTHY</div></div>' +
    '<div style="text-align:center"><div style="font-size:16px;font-weight:700;font-family:var(--mono)">' + Math.round(avgRate * 100) + '%</div><div style="font-size:8px;color:var(--dim)">AVG RATE</div></div>' +
  '</div>' + html;
  el.innerHTML = html;
}

function updatePanelById(id) {
  if (id.indexOf('live-') === 0) { updateVideoPanel(id); return; }
  var fnMap = {
    sources: updateSources, markets: updateMarkets, risk: updateRiskGauges,
    energy: updateEnergy, news: updateNewsTicker, osint: updateOsint,
    delta: updateDelta, analysis: updateAnalysis, signals: updateSignals,
    quakes: updateQuakes, nuclear: updateNuclear, cyber: updateCyber,
    space: updateSpace, conflicts: updateConflicts, globe: updateGlobe,
    sanctions: updateSanctions, economy: updateEconomy, climate: updateClimate,
    network: updateNetwork, transport: updateTransport, trends: updateTrends, asteroids: updateAsteroids,
    srcmonitor: updateSourceMonitor
  };
  if (fnMap[id]) fnMap[id]();
}
