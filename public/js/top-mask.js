(function(){
  function ensureMask(){
    var el = document.getElementById('gpu-top-mask');
    if (el) return el;
    el = document.createElement('div');
    el.id = 'gpu-top-mask';
    el.setAttribute('aria-hidden','true');
    var s = el.style;
    s.position = 'fixed';
    s.top = '0';
    s.left = '0';
    s.right = '0';
    s.height = '1px';
    s.background = 'transparent';
    s.zIndex = '2147483647';
    s.pointerEvents = 'none';
    s.backfaceVisibility = 'hidden';
    s.webkitBackfaceVisibility = 'hidden';
    s.transform = 'translateZ(0)';
    s.willChange = 'transform';
    s.contain = 'paint';
    (document.body || document.documentElement).appendChild(el);
    return el;
  }

  function resolveVar(name, fallback){
    try {
      var cs = getComputedStyle(document.documentElement);
      var v = cs.getPropertyValue(name);
      return (v && v.trim()) || fallback;
    } catch(_) { return fallback; }
  }

  function chooseColor(){
    var path = (location && location.pathname) || '';
    var hasCanvas = !!document.querySelector('.canvas-navbar') || /\/canvas(\b|\W)/.test(path);
    var navbar = resolveVar('--navbar-bg', '#000');
    var primary = resolveVar('--bg-primary', '#000');
    return hasCanvas ? navbar : primary;
  }

  function updateMask(){
    var el = ensureMask();
    // Use at least 1 CSS px; bump to 2 CSS px on problematic DPRs
    var dpr = window.devicePixelRatio || 1;
    var cssPx = (dpr % 1 === 0 ? 1 : 2); // non-integer DPRs get 2px
    el.style.height = cssPx + 'px';
    // Keep exactly at top without subpixel drift
    el.style.top = '0px';
    // Update color depending on page context
    el.style.background = chooseColor();
  }

  function init(){
    updateMask();
    window.addEventListener('resize', updateMask, { passive: true });
    window.addEventListener('orientationchange', updateMask, { passive: true });
    // Reapply after theme changes so vars update
    var mq = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)');
    if (mq && mq.addEventListener) mq.addEventListener('change', updateMask);
    // React to SPA route swaps and DOM changes
    var obs = new MutationObserver(function(){ updateMask(); });
    obs.observe(document.documentElement, { childList: true, subtree: true, attributes: true });
  }

  if (document.readyState === 'loading')
    document.addEventListener('DOMContentLoaded', init, { once: true });
  else
    init();
})();
