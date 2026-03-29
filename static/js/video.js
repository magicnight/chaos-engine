/* ================================================================
   C.H.A.O.S. — Connected Human-Augmented OSINT Suite
   Live Video Channels
   ================================================================ */
var LIVE_CHANNELS = {
  'live-bloomberg':  { ytId: 'iEpJwprxDdk' },
  'live-aljazeera':  { ytId: 'gCNeDWCI0vo' },
  'live-france24':   { ytId: 'Ap-UM1O9RBU' },
  'live-dw':         { ytId: 'LuKwFajn37U' },
  'live-euronews':   { ytId: 'pykpO5kQJ98' },
  'live-skynews':    { ytId: 'YDvsBbKfLPA' },
  'live-cnbc':       { ytId: '-qet9mBbNE4' },
  'live-nhk':        { ytId: 'f0lYkdA-Gtw' },
  'live-cctv4':      { ytId: 'f6Kq93wnaZ8' },
  'live-tvbs':       { ytId: 'awgNKme3kr8' },
  'live-cti':        { ytId: '9i0EtHb93jU' },
  'live-ebc':        { ytId: 'elQJhqvQD6I' },
  'live-phoenix':    { ytId: 'fN9uYWCjQaw' },
  'live-cgtn':       { ytId: 'BOy2xDU1LC8' },
  'live-cctvnews':   { ytId: 'f6Kq93wnaZ8' }
};

function updateVideoPanel(panelId) {
  var el = document.getElementById('panel-' + panelId);
  if (!el) return;
  var ch = LIVE_CHANNELS[panelId];
  if (!ch) return;
  if (el.querySelector('iframe')) return;
  var container = document.createElement('div');
  container.style.cssText = 'position:relative;width:100%;height:100%;min-height:200px;';
  var iframe = document.createElement('iframe');
  iframe.src = 'https://www.youtube.com/embed/' + ch.ytId + '?autoplay=1&mute=1&controls=1&rel=0';
  iframe.style.cssText = 'position:absolute;inset:0;width:100%;height:100%;border:none;border-radius:4px;';
  iframe.loading = 'lazy';
  iframe.allow = 'autoplay; encrypted-media; picture-in-picture';
  iframe.allowFullscreen = true;
  container.appendChild(iframe);
  var badge = document.createElement('span');
  badge.style.cssText = 'position:absolute;top:4px;right:4px;background:rgba(239,68,68,0.9);color:white;font-size:9px;font-weight:bold;padding:2px 6px;border-radius:4px;';
  badge.textContent = 'LIVE';
  container.appendChild(badge);
  el.appendChild(container);
}
