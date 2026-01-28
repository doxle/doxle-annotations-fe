(function () {
  function ready(fn) {
    if (document.readyState !== 'loading') fn();
    else document.addEventListener('DOMContentLoaded', fn, { once: true });
  }

  function init() {
    const existing = document.getElementById('geometric-grid-container');
    if (existing) {
      initWithContainer(existing);
      return;
    }
    const obs = new MutationObserver((_, observer) => {
      const el = document.getElementById('geometric-grid-container');
      if (el) {
        observer.disconnect();
        initWithContainer(el);
      }
    });
    obs.observe(document.documentElement, { childList: true, subtree: true });
  }

  function initWithContainer(container) {
    const gridSize = 6;
    let cellSize = 80;
    let isTabVisible = true;

    // Inject styles once
    if (!document.getElementById('geometric-grid-styles')) {
      const style = document.createElement('style');
      style.id = 'geometric-grid-styles';
      style.textContent = `
        .geo-grid {
          position: relative;
          width: 100%;
          height: 100%;
          transform: translateZ(0);
          will-change: transform;
        }
        .geo-line {
          position: absolute;
          background: rgba(255,255,255,0.15);
          transform-origin: 0 0;
          will-change: transform;
        }
        .geo-dot-static {
          position: absolute;
          width: 6px;
          height: 6px;
          background: #fff;
          border-radius: 50%;
          transform: translate(-50%, -50%) translateZ(0);
          will-change: transform, opacity;
          animation: geoPulse 3s ease-in-out infinite;
        }
        .geo-dot-moving {
          position: absolute;
          width: 4px;
          height: 4px;
          background: rgba(255,255,255,0.7);
          border-radius: 50%;
          transform: translateZ(0);
          will-change: transform, opacity;
        }
        .geo-dot-h {
          animation: geoMoveH var(--duration) linear infinite;
          animation-delay: var(--delay);
        }
        .geo-dot-v {
          animation: geoMoveV var(--duration) linear infinite;
          animation-delay: var(--delay);
        }
        .geo-dot-d1 {
          animation: geoMoveD1 var(--duration) linear infinite;
          animation-delay: var(--delay);
        }
        .geo-dot-d2 {
          animation: geoMoveD2 var(--duration) linear infinite;
          animation-delay: var(--delay);
        }
        @keyframes geoPulse {
          0%, 100% { transform: translate(-50%, -50%) scale(1) translateZ(0); opacity: 0.5; }
          50% { transform: translate(-50%, -50%) scale(1.4) translateZ(0); opacity: 1; }
        }
        @keyframes geoMoveH {
          0% { transform: translate3d(0, -50%, 0); opacity: 0; }
          5% { opacity: 0.7; }
          95% { opacity: 0.7; }
          100% { transform: translate3d(var(--grid-width), -50%, 0); opacity: 0; }
        }
        @keyframes geoMoveV {
          0% { transform: translate3d(-50%, 0, 0); opacity: 0; }
          5% { opacity: 0.7; }
          95% { opacity: 0.7; }
          100% { transform: translate3d(-50%, var(--grid-height), 0); opacity: 0; }
        }
        @keyframes geoMoveD1 {
          0% { transform: translate3d(0, 0, 0); opacity: 0; }
          5% { opacity: 0.7; }
          95% { opacity: 0.7; }
          100% { transform: translate3d(var(--grid-width), var(--grid-height), 0); opacity: 0; }
        }
        @keyframes geoMoveD2 {
          0% { transform: translate3d(var(--grid-width), 0, 0); opacity: 0; }
          5% { opacity: 0.7; }
          95% { opacity: 0.7; }
          100% { transform: translate3d(0, var(--grid-height), 0); opacity: 0; }
        }
      `;
      document.head.appendChild(style);
    }

    function calculateSize() {
      const rect = container.getBoundingClientRect();
      const minDim = Math.min(rect.width, rect.height);
      cellSize = Math.floor(minDim / (gridSize + 1));
      return { width: cellSize * gridSize, height: cellSize * gridSize };
    }

    function buildGrid() {
      container.innerHTML = '';
      const { width, height } = calculateSize();

      const grid = document.createElement('div');
      grid.className = 'geo-grid';
      grid.style.width = width + 'px';
      grid.style.height = height + 'px';
      grid.style.setProperty('--grid-width', width + 'px');
      grid.style.setProperty('--grid-height', height + 'px');

      const frag = document.createDocumentFragment();

      // Horizontal lines
      for (let i = 0; i <= gridSize; i++) {
        const line = document.createElement('div');
        line.className = 'geo-line';
        line.style.width = width + 'px';
        line.style.height = '1px';
        line.style.top = (i * cellSize) + 'px';
        line.style.left = '0';
        frag.appendChild(line);
      }

      // Vertical lines
      for (let i = 0; i <= gridSize; i++) {
        const line = document.createElement('div');
        line.className = 'geo-line';
        line.style.width = '1px';
        line.style.height = height + 'px';
        line.style.left = (i * cellSize) + 'px';
        line.style.top = '0';
        frag.appendChild(line);
      }

      // Diagonal lines within each cell
      for (let x = 0; x < gridSize; x++) {
        for (let y = 0; y < gridSize; y++) {
          // Top-left to bottom-right
          const d1 = document.createElement('div');
          d1.className = 'geo-line';
          d1.style.left = (x * cellSize) + 'px';
          d1.style.top = (y * cellSize) + 'px';
          d1.style.width = (Math.sqrt(2) * cellSize) + 'px';
          d1.style.height = '1px';
          d1.style.transform = 'rotate(45deg)';
          frag.appendChild(d1);

          // Top-right to bottom-left
          const d2 = document.createElement('div');
          d2.className = 'geo-line';
          d2.style.left = ((x + 1) * cellSize) + 'px';
          d2.style.top = (y * cellSize) + 'px';
          d2.style.width = (Math.sqrt(2) * cellSize) + 'px';
          d2.style.height = '1px';
          d2.style.transform = 'rotate(135deg)';
          frag.appendChild(d2);
        }
      }

      // Static dots at specific intersections (sparse pattern)
      const dotPositions = [
        [0,0], [0,1], [0,3], [0,5], [0,6],
        [1,0], [1,2], [1,3], [1,4], [1,6],
        [2,1], [2,5],
        [3,0], [3,2], [3,3], [3,4], [3,6],
        [4,1], [4,5],
        [5,0], [5,2], [5,3], [5,4], [5,6],
        [6,0], [6,1], [6,3], [6,5], [6,6]
      ];

      dotPositions.forEach(([x, y], idx) => {
        const dot = document.createElement('div');
        dot.className = 'geo-dot-static';
        dot.style.left = (x * cellSize) + 'px';
        dot.style.top = (y * cellSize) + 'px';
        dot.style.animationDelay = ((idx * 0.15) % 3) + 's';
        frag.appendChild(dot);
      });

      // Moving dots along lines
      for (let i = 0; i <= gridSize; i++) {
        // Horizontal
        const dh = document.createElement('div');
        dh.className = 'geo-dot-moving geo-dot-h';
        dh.style.left = '0';
        dh.style.top = (i * cellSize) + 'px';
        dh.style.setProperty('--duration', '4s');
        dh.style.setProperty('--delay', (i * 0.6) + 's');
        frag.appendChild(dh);

        // Vertical
        const dv = document.createElement('div');
        dv.className = 'geo-dot-moving geo-dot-v';
        dv.style.left = (i * cellSize) + 'px';
        dv.style.top = '0';
        dv.style.setProperty('--duration', '4s');
        dv.style.setProperty('--delay', (i * 0.4 + 2) + 's');
        frag.appendChild(dv);
      }

      // Diagonal moving dots
      for (let i = 0; i < 3; i++) {
        const dd1 = document.createElement('div');
        dd1.className = 'geo-dot-moving geo-dot-d1';
        dd1.style.left = '0';
        dd1.style.top = '0';
        dd1.style.setProperty('--duration', '5s');
        dd1.style.setProperty('--delay', (i * 1.5) + 's');
        frag.appendChild(dd1);

        const dd2 = document.createElement('div');
        dd2.className = 'geo-dot-moving geo-dot-d2';
        dd2.style.left = '0';
        dd2.style.top = '0';
        dd2.style.setProperty('--duration', '5s');
        dd2.style.setProperty('--delay', (i * 1.5 + 0.8) + 's');
        frag.appendChild(dd2);
      }

      grid.appendChild(frag);
      container.appendChild(grid);
    }

    buildGrid();

    // Handle visibility
    function handleVisibility() {
      isTabVisible = !document.hidden;
    }

    let resizeTimer = 0;
    function onResize() {
      clearTimeout(resizeTimer);
      resizeTimer = setTimeout(buildGrid, 150);
    }

    window.addEventListener('resize', onResize, { passive: true });
    document.addEventListener('visibilitychange', handleVisibility, { passive: true });

    window.__geometricGridCleanup = function () {
      window.removeEventListener('resize', onResize);
      document.removeEventListener('visibilitychange', handleVisibility);
    };
  }

  ready(init);
})();
