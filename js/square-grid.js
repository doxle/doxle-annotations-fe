(function () {
  function ready(fn) {
    if (document.readyState !== 'loading') fn();
    else document.addEventListener('DOMContentLoaded', fn, { once: true });
  }

  function init() {
    const existing = document.getElementById('square-grid-container');
    if (existing) {
      initWithContainer(existing);
      return;
    }
    const obs = new MutationObserver((_, observer) => {
      const el = document.getElementById('square-grid-container');
      if (el) {
        observer.disconnect();
        initWithContainer(el);
      }
    });
    obs.observe(document.documentElement, { childList: true, subtree: true });
  }

  function chooseGrid(container) {
    const rect = container.getBoundingClientRect();
    const w = Math.max(0, rect.width || window.innerWidth || 0);
    const h = Math.max(0, rect.height || window.innerHeight || 0);
    const area = w * h;

    let targetSquares = 500;
    if (window.devicePixelRatio > 2) targetSquares = Math.min(targetSquares, 400);
    if (matchMedia && matchMedia('(pointer:coarse)').matches) targetSquares = Math.min(targetSquares, 350);

    const spacingRaw = Math.sqrt(area / Math.max(1, targetSquares));
    const spacing = Math.max(28, Math.min(50, spacingRaw));
const squareSize = spacing * 0.15;

    const gridX = Math.ceil(w / spacing) + 1;
    const gridY = Math.ceil(h / spacing) + 1;

    return { gridX, gridY, spacing, squareSize, width: w, height: h };
  }

  function initWithContainer(container) {
    let config = chooseGrid(container);

    function buildGrid() {
      container.innerHTML = '';
      const frag = document.createDocumentFragment();
      const squares = new Array(config.gridX * config.gridY);
      let i = 0;

      for (let x = 0; x < config.gridX; x++) {
        for (let y = 0; y < config.gridY; y++) {
          const sq = document.createElement('div');
          sq.className = 'square';
          const bx = x * config.spacing;
          const by = y * config.spacing;
          sq.style.left = bx + 'px';
          sq.style.top = by + 'px';
          sq.style.width = config.squareSize + 'px';
          sq.style.height = config.squareSize + 'px';
          sq._bx = bx;
          sq._by = by;
          sq._ox = 0;
          sq._oy = 0;
          frag.appendChild(sq);
          squares[i++] = sq;
        }
      }

      container.appendChild(frag);
      return squares;
    }

    let squares = buildGrid();

    let mx = -9999;
    let my = -9999;
    let scheduled = false;
    let isTabVisible = true;
    let frameSkip = 0;

    let pushRadius = Math.min(220, Math.max(150, config.spacing * 6));
    let r2 = pushRadius * pushRadius;

    function refreshPhysics() {
      pushRadius = Math.min(220, Math.max(150, config.spacing * 6));
      r2 = pushRadius * pushRadius;
    }

    function update() {
      scheduled = false;
      if (!isTabVisible) return;

      frameSkip++;
      if (frameSkip % 2 === 0) {
        for (let k = 0; k < squares.length; k++) {
          const sq = squares[k];
          const dx = sq._bx - mx;
          const dy = sq._by - my;
          const dist2 = dx * dx + dy * dy;

          if (dist2 < r2) {
            const dist = Math.sqrt(dist2);
            const strength = ((pushRadius - dist) / pushRadius) * 50;
            const inv = dist > 0 ? 1 / dist : 0;
            const ox = dx * inv * strength;
            const oy = dy * inv * strength;

            if (sq._ox !== ox || sq._oy !== oy) {
              sq.style.transform = `translate(calc(-50% + ${ox}px), calc(-50% + ${oy}px))`;
              sq._ox = ox;
              sq._oy = oy;
            }
          } else {
            if (sq._ox !== 0 || sq._oy !== 0) {
              sq.style.transform = 'translate(-50%, -50%)';
              sq._ox = 0;
              sq._oy = 0;
            }
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
        squares = buildGrid();
        refreshPhysics();
      }, 150);
    }

    function handleVisibilityChange() {
      isTabVisible = !document.hidden;
    }

    window.addEventListener('mousemove', onMouseMove, { passive: true });
    window.addEventListener('touchmove', onTouchMove, { passive: true });
    window.addEventListener('resize', onResize, { passive: true });
    document.addEventListener('visibilitychange', handleVisibilityChange, { passive: true });

    window.__squareGridCleanup = function () {
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('touchmove', onTouchMove);
      window.removeEventListener('resize', onResize);
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }

  ready(init);
})();
