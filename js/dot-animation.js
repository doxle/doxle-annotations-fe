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

  // Create SVG background pattern
  function createSVGBackground(spacing) {
    const svg = `
      <svg xmlns="http://www.w3.org/2000/svg" width="${spacing}" height="${spacing}">
        <circle cx="${spacing/2}" cy="${spacing/2}" r="1.5" fill="rgba(149, 128, 255, 0.2)"/>
      </svg>
    `;
    return `url('data:image/svg+xml;utf8,${encodeURIComponent(svg)}')`;
  }

  // Choose grid for interactive dots only (much fewer)
  function chooseGrid(container) {
    const rect = container.getBoundingClientRect();
    const w = Math.max(0, rect.width || window.innerWidth || 0);
    const h = Math.max(0, rect.height || window.innerHeight || 0);

    // Only create 50-80 interactive dots for mouse tracking
    let targetDots = 80;
    if (window.devicePixelRatio > 2) targetDots = Math.min(targetDots, 60);
    if (matchMedia && matchMedia('(pointer:coarse)').matches) targetDots = Math.min(targetDots, 50);
    if (matchMedia && matchMedia('(prefers-reduced-motion: reduce)').matches) targetDots = 0;

    const spacing = 32; // Fixed spacing for background pattern
    const interactiveSpacing = Math.sqrt((w * h) / Math.max(1, targetDots));
    const dotsX = Math.ceil(Math.sqrt(targetDots * (w/h)));
    const dotsY = Math.ceil(Math.sqrt(targetDots * (h/w)));

    return { dotsX, dotsY, spacing, interactiveSpacing, width: w, height: h };
  }

  function initWithContainer(container) {
    let config = chooseGrid(container);
    
    // Set SVG pattern as background
    container.style.backgroundImage = createSVGBackground(config.spacing);
    container.style.backgroundRepeat = 'repeat';
    container.style.backgroundPosition = `${config.spacing/2}px ${config.spacing/2}px`;

    function buildGrid() {
      container.innerHTML = '';
      
      // Only create interactive dots if not reduced motion
      if (config.dotsX === 0 || config.dotsY === 0) return [];
      
      const frag = document.createDocumentFragment();
      const dots = [];
      
      // Distribute interactive dots randomly across viewport
      for (let i = 0; i < config.dotsX * config.dotsY; i++) {
        const d = document.createElement('div');
        d.className = 'dot interactive-dot';
        const bx = Math.random() * config.width;
        const by = Math.random() * config.height;
        d.style.left = bx + 'px';
        d.style.top = by + 'px';
        // store numeric base positions to avoid parsing each frame
        d._bx = bx;
        d._by = by;
        d._t = 'translate3d(0,0,0)';
        d.style.opacity = '0'; // Start invisible, show only when near mouse
        frag.appendChild(d);
        dots.push(d);
      }
      container.appendChild(frag);
      return dots;
    }

    let dots = buildGrid();

    let mx = -9999;
    let my = -9999;
    let scheduled = false;
    let isTabVisible = true;

    let pushRadius = 150; // Smaller radius for better performance
    let r2 = pushRadius * pushRadius;
    let fadeRadius = 200; // Radius for opacity fade
    let fr2 = fadeRadius * fadeRadius;
    
    function refreshPhysics() {
      pushRadius = 150;
      r2 = pushRadius * pushRadius;
      fadeRadius = 200;
      fr2 = fadeRadius * fadeRadius;
    }

    function update() {
      scheduled = false;
      if (!isTabVisible) return; // Skip updates when tab is hidden
      
      for (let k = 0; k < dots.length; k++) {
        const d = dots[k];
        const dx = d._bx - mx;
        const dy = d._by - my;
        const dist2 = dx * dx + dy * dy;
        
        // Fade dots in/out based on distance to cursor
        if (dist2 < fr2) {
          const opacity = Math.max(0.3, 1 - (Math.sqrt(dist2) / fadeRadius));
          d.style.opacity = opacity;
          
          if (dist2 < r2) {
            const dist = Math.sqrt(dist2);
            const strength = ((pushRadius - dist) / pushRadius) * 30; // Reduced strength
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
        } else {
          d.style.opacity = '0';
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
        container.style.backgroundImage = createSVGBackground(config.spacing);
        dots = buildGrid();
        refreshPhysics();
      }, 150);
    }

    // Handle tab visibility changes
    function handleVisibilityChange() {
      isTabVisible = !document.hidden;
      if (!isTabVisible) {
        // Hide all interactive dots when tab is hidden
        dots.forEach(d => d.style.opacity = '0');
      }
    }

    window.addEventListener('mousemove', onMouseMove, { passive: true });
    window.addEventListener('touchmove', onTouchMove, { passive: true });
    window.addEventListener('resize', onResize, { passive: true });
    document.addEventListener('visibilitychange', handleVisibilityChange, { passive: true });

    // Expose cleanup if your router unmounts this page
    window.__dotsCleanup = function () {
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('touchmove', onTouchMove);
      window.removeEventListener('resize', onResize);
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }

  ready(init);
})();
