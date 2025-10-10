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

  function initWithContainer(container) {
    const dotsX = 60;
    const dotsY = 35;
    const spacing = 30;
    const pushRadius = 180;

    container.innerHTML = '';

    const frag = document.createDocumentFragment();
    const dots = new Array(dotsX * dotsY);
    let i = 0;
    for (let x = 0; x < dotsX; x++) {
      for (let y = 0; y < dotsY; y++) {
        const d = document.createElement('div');
        d.className = 'dot';
        const bx = x * spacing;
        const by = y * spacing;
        d.style.left = bx + 'px';
        d.style.top = by + 'px';
        // store numeric base positions to avoid parsing each frame
        d._bx = bx;
        d._by = by;
        frag.appendChild(d);
        dots[i++] = d;
      }
    }
    container.appendChild(frag);

    let mx = -9999;
    let my = -9999;
    let scheduled = false;

    function update() {
      scheduled = false;
      for (let k = 0; k < dots.length; k++) {
        const d = dots[k];
        const dx = d._bx - mx;
        const dy = d._by - my;
        const dist = Math.hypot(dx, dy);
        if (dist < pushRadius) {
          const strength = ((pushRadius - dist) / pushRadius) * 50;
          const inv = dist > 0 ? 1 / dist : 0;
          const ox = dx * inv * strength;
          const oy = dy * inv * strength;
          d.style.transform = `translate3d(${ox}px, ${oy}px, 0)`;
        } else {
          d.style.transform = 'translate3d(0,0,0)';
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

    window.addEventListener('mousemove', onMouseMove, { passive: true });
    window.addEventListener('touchmove', onTouchMove, { passive: true });

    // Expose cleanup if your router unmounts this page
    window.__dotsCleanup = function () {
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('touchmove', onTouchMove);
    };
  }

  ready(init);
})();
