(function () {
  function ready(fn) {
    if (document.readyState !== 'loading') fn();
    else document.addEventListener('DOMContentLoaded', fn, { once: true });
  }

  function init() {
    const existing = document.getElementById('dots-container');
    if (existing) {
      initWithContainer(existing);
      return;
    }
    // If not yet in DOM (SPA route), observe until it appears
    const obs = new MutationObserver((_, observer) => {
      const el = document.getElementById('dots-container');
      if (el) {
        observer.disconnect();
        initWithContainer(el);
      }
    });
    obs.observe(document.documentElement, { childList: true, subtree: true });
  }

  // Choose grid to target ~N dots based on viewport and device characteristics
  function chooseGrid(container) {
    const rect = container.getBoundingClientRect();
    const w = Math.max(0, rect.width || window.innerWidth || 0);
    const h = Math.max(0, rect.height || window.innerHeight || 0);
    const area = w * h;

    let targetDots = 2000;
    if (window.devicePixelRatio > 2) targetDots = Math.min(targetDots, 1500);
    if (matchMedia && matchMedia('(pointer:coarse)').matches) targetDots = Math.min(targetDots, 1200);
    if (matchMedia && matchMedia('(prefers-reduced-motion: reduce)').matches) targetDots = Math.min(targetDots, 900);

    const spacingRaw = Math.sqrt(area / Math.max(1, targetDots));
    const spacing = Math.max(24, Math.min(48, spacingRaw));

    const dotsX = Math.ceil(w / spacing) + 1;
    const dotsY = Math.ceil(h / spacing) + 1;

    return { dotsX, dotsY, spacing, width: w, height: h };
  }

  function initWithContainer(container) {
    let config = chooseGrid(container);

    function buildGrid() {
      container.innerHTML = '';
      const frag = document.createDocumentFragment();
      const dots = new Array(config.dotsX * config.dotsY);
      let i = 0;
      for (let x = 0; x < config.dotsX; x++) {
        for (let y = 0; y < config.dotsY; y++) {
          const d = document.createElement('div');
          d.className = 'dot';
          const bx = x * config.spacing;
          const by = y * config.spacing;
          d.style.left = bx + 'px';
          d.style.top = by + 'px';
          // store numeric base positions to avoid parsing each frame
          d._bx = bx;
          d._by = by;
          d._t = 'translate3d(0,0,0)';
          frag.appendChild(d);
          dots[i++] = d;
        }
      }
      container.appendChild(frag);
      return dots;
    }

    let dots = buildGrid();

    let mx = -9999;
    let my = -9999;
    let scheduled = false;

    let pushRadius = Math.min(240, Math.max(140, config.spacing * 6));
    let r2 = pushRadius * pushRadius;
    function refreshPhysics() {
      pushRadius = Math.min(240, Math.max(140, config.spacing * 6));
      r2 = pushRadius * pushRadius;
    }

    function update() {
      scheduled = false;
      for (let k = 0; k < dots.length; k++) {
        const d = dots[k];
        const dx = d._bx - mx;
        const dy = d._by - my;
        const dist2 = dx * dx + dy * dy;
        if (dist2 < r2) {
          const dist = Math.sqrt(dist2);
          const strength = ((pushRadius - dist) / pushRadius) * 50;
          const inv = dist > 0 ? 1 / dist : 0;
          const ox = dx * inv * strength;
          const oy = dy * inv * strength;
          const tr = `translate3d(${ox}px, ${oy}px, 0)`;
          if (d._t !== tr) {
            d.style.transform = tr;
            d._t = tr;
          }
        } else {
          const tr = 'translate3d(0,0,0)';
          if (d._t !== tr) {
            d.style.transform = tr;
            d._t = tr;
          }
        }
      }
    }

    function onMoveLike(p) {
      mx = p.clientX;
      my = p.clientY;
      if (!scheduled) {
        scheduled = true;
        requestAnimationFrame(update);
      }
    }

    function onMouseMove(e) { onMoveLike(e); }
    function onTouchMove(e) {
      if (e.touches && e.touches[0]) onMoveLike(e.touches[0]);
    }

    let resizeTimer = 0;
    function onResize() {
      clearTimeout(resizeTimer);
      resizeTimer = setTimeout(() => {
        config = chooseGrid(container);
        dots = buildGrid();
        refreshPhysics();
      }, 150);
    }

    window.addEventListener('mousemove', onMouseMove, { passive: true });
    window.addEventListener('touchmove', onTouchMove, { passive: true });
    window.addEventListener('resize', onResize, { passive: true });

    // Expose cleanup if your router unmounts this page
    window.__dotsCleanup = function () {
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('touchmove', onTouchMove);
      window.removeEventListener('resize', onResize);
    };
  }

  ready(init);
})();
