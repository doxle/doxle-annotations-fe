(function(){
  function ready(fn){
    if (document.readyState !== 'loading') fn();
    else document.addEventListener('DOMContentLoaded', fn, { once: true });
  }

  function injectCSS(){
    if (document.getElementById('home-text-sheen-style')) return;
    var css = "\n.home-text-sheen { position: relative; display: inline; overflow: visible; vertical-align: baseline; line-height: inherit; }\n.home-text-sheen::after {\n  content: \"\";\n  position: absolute;\n  top: 0; left: -70%;\n  width: 140%; height: 100%;\n  background: linear-gradient(120deg, transparent 30%, var(--shimmer-color) 50%, transparent 70%);\n  transform: skewX(-20deg);\n  animation: homeSheen 20s ease-in-out infinite;\n  opacity: 0; mix-blend-mode: screen; pointer-events: none; will-change: transform, opacity;\n}\n@keyframes homeSheen {\n  0%, 2% { transform: skewX(-20deg) translateX(-160%); opacity: 0; }\n  8% { opacity: 0.9; }\n  28% { transform: skewX(-20deg) translateX(160%); opacity: 1; }\n  30% { opacity: 0; }\n  34% { transform: skewX(-20deg) translateX(-160%); opacity: 0; }\n  40% { opacity: 0.9; }\n  60% { transform: skewX(-20deg) translateX(160%); opacity: 1; }\n  62% { opacity: 0; }\n  66% { transform: skewX(-20deg) translateX(-160%); opacity: 0; }\n  72% { opacity: 0.9; }\n  92% { transform: skewX(-20deg) translateX(160%); opacity: 1; }\n  94%, 100% { transform: skewX(-20deg) translateX(160%); opacity: 0; }\n}\n";
    var style = document.createElement('style');
    style.id = 'home-text-sheen-style';
    style.textContent = css;
    document.head.appendChild(style);
  }

  function injectLineCSS(){
    var css = "\n.home-line-sheen { position: relative; }\n.home-line-sheen::after {\n  content: \"\"; position: absolute; top: 0; left: var(--sheen-start, 0px); width: 140%; height: 100%;\n  background: linear-gradient(120deg, transparent 30%, var(--shimmer-color) 50%, transparent 70%);\n  transform: skewX(-20deg); opacity: 0; mix-blend-mode: screen; pointer-events: none; will-change: transform, opacity;\n}\n.home-line-sheen-run::after { animation: lineSheen 3.6s ease-in-out 1; }\n@keyframes lineSheen { 0% { transform: skewX(-20deg) translateX(0); opacity: 0; } 5% { opacity: 0.9; } 100% { transform: skewX(-20deg) translateX(160%); opacity: 0; } }\n";
    var style = document.getElementById('home-line-sheen-style');
    if (!style) {
      style = document.createElement('style');
      style.id = 'home-line-sheen-style';
      document.head.appendChild(style);
    }
    if (style.textContent !== css) {
      style.textContent = css;
    }
  }

  // Overlay CSS for animating the entire hero text block with a single stripe
  function injectOverlayCSS(){
var css = "\n.home-sheen-overlay { position: absolute; pointer-events: none; overflow: visible; z-index: 11; }\n.home-sheen-stripe { position: absolute; top: 0; left: var(--stripe-left, 0px); height: 100%; background: linear-gradient(120deg, transparent 30%, var(--shimmer-color) 50%, transparent 70%); transform: translateX(0); opacity: 0; will-change: transform, opacity; animation: overlaySheen 4.2s ease-in-out infinite; }\n@keyframes overlaySheen { 0% { transform: translateX(0); opacity: 0; } 6% { opacity: 0.9; } 86% { transform: translateX(var(--travel, 0px)); opacity: 1; } 100% { transform: translateX(var(--travel, 0px)); opacity: 0; } }\n";
    var style = document.getElementById('home-overlay-sheen-style');
    if (!style) {
      style = document.createElement('style');
      style.id = 'home-overlay-sheen-style';
      document.head.appendChild(style);
    }
    if (style.textContent !== css) {
      style.textContent = css;
    }
  }

  // Apply a single overlay across the union of hero headings
  function applyOverlaySheen(){
    // Find hero container (parent of first h1 containing 'BUILT WITH AI')
    var heroH1 = Array.from(document.querySelectorAll('h1')).find(function(el){
      return ((el.textContent || '').trim().toUpperCase().indexOf('BUILT WITH AI') !== -1);
    });
    var container = heroH1 ? heroH1.parentElement : (document.querySelector('h1') ? document.querySelector('h1').parentElement : null);
    if (!container) return;

    var heads = Array.from(container.querySelectorAll('h1, h2'));
    if (!heads.length) return;

    // Union bounding box of all headings (relative to container)
    var pr = container.getBoundingClientRect();
    var minL = Infinity, minT = Infinity, maxR = -Infinity, maxB = -Infinity;
    for (var i = 0; i < heads.length; i++){
      var r = heads[i].getBoundingClientRect();
      minL = Math.min(minL, r.left);
      minT = Math.min(minT, r.top);
      maxR = Math.max(maxR, r.right);
      maxB = Math.max(maxB, r.bottom);
    }
    if (!isFinite(minL) || !isFinite(minT) || !isFinite(maxR) || !isFinite(maxB)) return;

    // Determine the BUILD line and restrict overlay height to that line only
    var mid = heads.find(function(h){ return h.tagName.toLowerCase() === 'h2' && ((h.textContent || '').toUpperCase().indexOf('BUILD') !== -1); }) || heads[1] || heads[0];
    var midRect = mid.getBoundingClientRect();

    // Find the BUILD span to constrain width
    var buildSpanForWidth = mid ? Array.from(mid.querySelectorAll('span')).find(function(s){ return ((s.textContent || '').trim().toUpperCase() === 'BUILD'); }) : null;
    var buildRect = buildSpanForWidth ? buildSpanForWidth.getBoundingClientRect() : midRect;

    var left = Math.max(0, Math.round(buildRect.left - pr.left));
    var top = Math.max(0, Math.round(midRect.top - pr.top));
    var width = Math.max(1, Math.round(buildRect.width));
    var height = Math.max(1, Math.round(midRect.height));

    var overlay = container.querySelector('#home-sheen-overlay');
    if (!overlay) {
      overlay = document.createElement('div');
      overlay.id = 'home-sheen-overlay';
      overlay.className = 'home-sheen-overlay';
      container.appendChild(overlay);

      var stripe = document.createElement('div');
      stripe.className = 'home-sheen-stripe';
      overlay.appendChild(stripe);
    }

    overlay.style.left = left + 'px';
    overlay.style.top = top + 'px';
    overlay.style.width = width + 'px';
    overlay.style.height = height + 'px';

    var stripe = overlay.firstElementChild;
    var stripeWidth = Math.round(width * 0.6);
    stripe.style.width = stripeWidth + 'px';
    stripe.style.setProperty('--travel', width + 'px');
    // Slightly longer wait before next animation by increasing duration
    stripe.style.animationDuration = '5.2s';

    // Start the stripe just before the BUILD word
    var startPx = 0 - Math.round(stripeWidth * 0.3);
    stripe.style.setProperty('--stripe-left', startPx + 'px');
    stripe.style.left = startPx + 'px';
  }

  function startOverlayObserver(){
    if (window.__homeOverlayObserver) return;
    var obs = new MutationObserver(function(){ applyOverlaySheen(); });
    obs.observe(document.documentElement, { childList: true, subtree: true });
    window.__homeOverlayObserver = obs;
  }

  var __lineSheen = { lines: [], idx: 0, pass: 4200, overlap: 3600, pause: 0, timer: null };

  function applyLineSheen(){
    var ov = document.getElementById('home-sheen-overlay');
    if (ov) ov.remove();
    var heroH1 = Array.from(document.querySelectorAll('h1')).find(function(el){
      return ((el.textContent || '').trim().toUpperCase().indexOf('BUILT WITH AI') !== -1);
    });
    var container = heroH1 ? heroH1.parentElement : (document.querySelector('h1') ? document.querySelector('h1').parentElement : null);
    var heads = container ? Array.from(container.querySelectorAll('h1, h2')) : Array.from(document.querySelectorAll('h1, h2'));
    if (!heads.length) return;
    __lineSheen.lines = heads;
    for (var i = 0; i < heads.length; i++){
      var h = heads[i];
      if (!h.classList.contains('home-line-sheen')) h.classList.add('home-line-sheen');
      if (h.classList.contains('home-line-sheen-run')) h.classList.remove('home-line-sheen-run');
      // Compute start offset (from beginning of text or 'BUILD' span if present)
      var start = 0;
      try {
        var spans = Array.from(h.querySelectorAll('span'));
        var buildSpan = spans.find(function(s){ return ((s.textContent||'').trim().toUpperCase()==='BUILD'); });
        if (buildSpan) {
          var hb = h.getBoundingClientRect();
          var sb = buildSpan.getBoundingClientRect();
          start = Math.max(0, Math.round(sb.left - hb.left));
        }
      } catch(e) { /* noop */ }
      h.style.setProperty('--sheen-start', start + 'px');
    }
  }

  function stopLineCycle(){
    if (__lineSheen.timer) { clearTimeout(__lineSheen.timer); __lineSheen.timer = null; }
    __lineSheen.idx = 0;
    for (var i = 0; i < __lineSheen.lines.length; i++){
      __lineSheen.lines[i].classList.remove('home-line-sheen-run');
    }
  }

  function startLineCycle(){
    stopLineCycle();
    function runNext(){
      if (!__lineSheen.lines.length) return;
      var n = __lineSheen.idx % __lineSheen.lines.length;
      var el = __lineSheen.lines[n];
      el.classList.remove('home-line-sheen-run');
      void el.offsetWidth; // reflow to restart animation
      el.classList.add('home-line-sheen-run');
      __lineSheen.idx++;
      var atEnd = (__lineSheen.idx % __lineSheen.lines.length === 0);
      var overlapDelay = Math.max(0, (__lineSheen.pass || 0) - (__lineSheen.overlap || 0));
      var delay = (atEnd && (__lineSheen.pause || 0) > 0) ? __lineSheen.pause : overlapDelay;
      __lineSheen.timer = setTimeout(runNext, delay);
    }
    runNext();
  }

  function startObserver(){
    if (window.__homeSheenObserver) return;
    var obs = new MutationObserver(function(){ applyLineSheen(); startLineCycle(); });
    obs.observe(document.documentElement, { childList: true, subtree: true });
    window.__homeSheenObserver = obs;
  }

  ready(function(){ injectOverlayCSS(); applyOverlaySheen(); startOverlayObserver(); window.addEventListener('resize', applyOverlaySheen, { passive: true }); });
})();
