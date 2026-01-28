(function () {
  function ready(fn) {
    if (document.readyState !== 'loading') fn();
    else document.addEventListener('DOMContentLoaded', fn, { once: true });
  }

  function init() {
    const existing = document.getElementById('constellation-container');
    if (existing) {
      initWithContainer(existing);
      return;
    }
    const obs = new MutationObserver((_, observer) => {
      const el = document.getElementById('constellation-container');
      if (el) {
        observer.disconnect();
        initWithContainer(el);
      }
    });
    obs.observe(document.documentElement, { childList: true, subtree: true });
  }

  function initWithContainer(container) {
    if (!document.getElementById('constellation-styles')) {
      const style = document.createElement('style');
      style.id = 'constellation-styles';
      style.textContent = `
        .constellation-node {
          position: absolute;
          width: 5px;
          height: 5px;
          background: rgba(180, 60, 70, 0.5);
          border-radius: 50%;
          transform: translate(-50%, -50%) scale(0);
          opacity: 0;
          will-change: transform, opacity;
        }
        .constellation-line {
          position: absolute;
          height: 1px;
          background: rgba(255, 255, 255, 0.12);
          transform-origin: 0 50%;
          transform: scaleX(0);
          opacity: 0;
          will-change: transform, opacity;
        }
        .node-visible {
          transform: translate(-50%, -50%) scale(1);
          opacity: 1;
          transition: transform 0.6s ease-out, opacity 0.6s ease-out;
        }
        .line-visible {
          transform: scaleX(1);
          opacity: 1;
          transition: transform 0.8s ease-out, opacity 0.6s ease-out;
        }
        .node-pulse {
          animation: nodePulse 4s ease-in-out infinite;
        }
        @keyframes nodePulse {
          0%, 100% { transform: translate(-50%, -50%) scale(1); opacity: 0.85; }
          50% { transform: translate(-50%, -50%) scale(1.2); opacity: 1; }
        }
      `;
      document.head.appendChild(style);
    }

    function shuffle(arr) {
      for (let i = arr.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [arr[i], arr[j]] = [arr[j], arr[i]];
      }
      return arr;
    }

    function buildConstellation() {
      container.innerHTML = '';
      const rect = container.getBoundingClientRect();
      const w = rect.width;
      const h = rect.height;

      // Scale factor to reduce horizontal stretch
      const sx = 0.56; // compress width more
      const ox = 0.22; // offset to center

      // Define triangles (only some dots will be connected)
      const triangles = [
        // Top
        { nodes: [{x:0.35,y:0.10}, {x:0.32,y:0.18}, {x:0.38,y:0.18}] },
        { nodes: [{x:0.32,y:0.18}, {x:0.26,y:0.26}, {x:0.38,y:0.18}] },
        // Upper left - sparse
        { nodes: [{x:0.14,y:0.30}, {x:0.22,y:0.30}, {x:0.18,y:0.38}] },
        { nodes: [{x:0.18,y:0.38}, {x:0.26,y:0.38}, {x:0.22,y:0.46}] },
        // Middle section - sparse
        { nodes: [{x:0.30,y:0.38}, {x:0.38,y:0.38}, {x:0.34,y:0.46}] },
        { nodes: [{x:0.34,y:0.46}, {x:0.42,y:0.46}, {x:0.38,y:0.54}] },
        // Lower center
        { nodes: [{x:0.32,y:0.54}, {x:0.38,y:0.54}, {x:0.35,y:0.62}] },
        // Bottom
        { nodes: [{x:0.35,y:0.70}, {x:0.32,y:0.78}, {x:0.38,y:0.78}] },
        // Right cluster - sparse
        { nodes: [{x:0.56,y:0.40}, {x:0.64,y:0.40}, {x:0.60,y:0.48}] },
        // Far right
        { nodes: [{x:0.72,y:0.38}, {x:0.80,y:0.42}, {x:0.76,y:0.50}] },
        // Scattered
        { nodes: [{x:0.10,y:0.44}, {x:0.14,y:0.52}, {x:0.06,y:0.50}] },
      ];

      // Extra unconnected dots (no lines)
      const extraDots = [
        {x:0.44,y:0.26}, {x:0.26,y:0.26}, {x:0.22,y:0.30}, {x:0.30,y:0.30},
        {x:0.46,y:0.38}, {x:0.42,y:0.46}, {x:0.28,y:0.46},
        {x:0.44,y:0.54}, {x:0.41,y:0.62}, {x:0.38,y:0.70}, {x:0.35,y:0.86},
        {x:0.52,y:0.40}, {x:0.68,y:0.40}, {x:0.64,y:0.48}, {x:0.60,y:0.56},
        {x:0.88,y:0.46}, {x:0.84,y:0.54}, {x:0.52,y:0.52}, {x:0.48,y:0.60},
        {x:0.72,y:0.50}, {x:0.68,y:0.58}, {x:0.18,y:0.50}, {x:0.22,y:0.58}, {x:0.14,y:0.56},
      ];

      const allNodes = [];
      const allLines = [];
      const nodeMap = new Map();

      // Collect unique nodes and lines from triangles
      triangles.forEach(tri => {
        const triNodeIndices = [];
        tri.nodes.forEach(pos => {
          const key = `${pos.x},${pos.y}`;
          if (!nodeMap.has(key)) {
            const idx = allNodes.length;
            const scaledX = (pos.x * sx + ox) * w;
            allNodes.push({ x: scaledX, y: pos.y * h, key });
            nodeMap.set(key, idx);
          }
          triNodeIndices.push(nodeMap.get(key));
        });
        // Create 3 lines for the triangle
        allLines.push({ from: triNodeIndices[0], to: triNodeIndices[1], tri });
        allLines.push({ from: triNodeIndices[1], to: triNodeIndices[2], tri });
        allLines.push({ from: triNodeIndices[2], to: triNodeIndices[0], tri });
      });

      // Add extra unconnected dots
      extraDots.forEach(pos => {
        const key = `${pos.x},${pos.y}`;
        if (!nodeMap.has(key)) {
          const scaledX = (pos.x * sx + ox) * w;
          allNodes.push({ x: scaledX, y: pos.y * h, key });
          nodeMap.set(key, allNodes.length - 1);
        }
      });

      const frag = document.createDocumentFragment();
      const nodeEls = [];
      const lineEls = [];

      // Create node elements
      allNodes.forEach((node, idx) => {
        const el = document.createElement('div');
        el.className = 'constellation-node';
        el.style.left = node.x + 'px';
        el.style.top = node.y + 'px';
        nodeEls.push(el);
        frag.appendChild(el);
      });

      // Create line elements
      allLines.forEach((line, idx) => {
        const n1 = allNodes[line.from];
        const n2 = allNodes[line.to];
        const dx = n2.x - n1.x;
        const dy = n2.y - n1.y;
        const length = Math.sqrt(dx * dx + dy * dy);
        const angle = Math.atan2(dy, dx) * (180 / Math.PI);

        const el = document.createElement('div');
        el.className = 'constellation-line';
        el.style.left = n1.x + 'px';
        el.style.top = n1.y + 'px';
        el.style.width = length + 'px';
        el.style.transform = `rotate(${angle}deg) scaleX(0)`;
        lineEls.push({ el, from: line.from, to: line.to });
        frag.appendChild(el);
      });

      container.appendChild(frag);

      // Phase 1: Draw all nodes one by one in random order
      const nodeIndices = shuffle([...Array(allNodes.length).keys()]);
      const nodeDelay = 120; // ms between each node
      const totalNodeTime = nodeIndices.length * nodeDelay;

      nodeIndices.forEach((nIdx, order) => {
        setTimeout(() => {
          nodeEls[nIdx].classList.add('node-visible');
          setTimeout(() => nodeEls[nIdx].classList.add('node-pulse'), 400);
        }, order * nodeDelay);
      });

      // Phase 2: After all nodes drawn, draw lines triangle by triangle
      const triIndices = shuffle([...Array(triangles.length).keys()]);

      triIndices.forEach((triIdx, order) => {
        const lineDelay = totalNodeTime + 500 + order * 280; // start after nodes + gap
        const triLines = allLines
          .map((l, i) => ({ ...l, idx: i }))
          .filter(l => l.tri === triangles[triIdx]);

        triLines.forEach((l, i) => {
          setTimeout(() => {
            lineEls[l.idx].el.style.transform = `rotate(${lineEls[l.idx].el.style.transform.match(/rotate\(([^)]+)\)/)[1]}) scaleX(1)`;
            lineEls[l.idx].el.classList.add('line-visible');
          }, lineDelay + i * 100);
        });
      });
    }

    buildConstellation();

    let resizeTimer = 0;
    function onResize() {
      clearTimeout(resizeTimer);
      resizeTimer = setTimeout(buildConstellation, 150);
    }

    window.addEventListener('resize', onResize, { passive: true });

    window.__constellationCleanup = function () {
      window.removeEventListener('resize', onResize);
    };
  }

  ready(init);
})();
