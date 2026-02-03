(function () {
  let currentResizeHandler = null;
  
  function ready(fn) {
    if (document.readyState !== 'loading') fn();
    else document.addEventListener('DOMContentLoaded', fn, { once: true });
  }

  function init() {
    const existing = document.getElementById('rects-container');
    if (existing && existing.children.length === 0) {
      initWithContainer(existing);
    }
  }
  
  // Watch for rects-container appearing (handles SPA navigation)
  // MutationObserver is efficient - only fires on actual DOM changes, not polling
  const domObserver = new MutationObserver((mutations) => {
    // Only check if mutations involve added nodes
    for (const mutation of mutations) {
      if (mutation.addedNodes.length > 0) {
        const el = document.getElementById('rects-container');
        if (el && el.children.length === 0) {
          initWithContainer(el);
          return;
        }
      }
    }
  });
  
  function startObserving() {
    // Observe body with subtree for SPA route changes
    domObserver.observe(document.body, { childList: true, subtree: true });
  }

  function initWithContainer(container) {
    const ASPECT_RATIO = 151 / 64; // Width to height ratio
    const PADDING = 2;
    const PALETTE = ['#DAC8FB', '#FFA5D2', '#049D56', '#FBFB52', '#A4E6F0'];
    const ANIMATION_DELAY = 30; // ms between each block appearing

    function buildGrid() {
      container.innerHTML = '';
      const rect = container.getBoundingClientRect();
      const w = rect.width || window.innerWidth;
      const h = rect.height || window.innerHeight;
      
      // Responsive configuration based on screen width
      let targetCols, blockWidth, targetBlockCount;
      
      if (w <= 480) {
        // Mobile: fewer larger blocks
        targetCols = 4;
        blockWidth = Math.floor((w - (targetCols + 1) * PADDING) / targetCols);
        targetBlockCount = 10;
      } else if (w <= 768) {
        // Tablet portrait: medium blocks
        targetCols = 5;
        blockWidth = Math.floor((w - (targetCols + 1) * PADDING) / targetCols);
        targetBlockCount = 12;
      } else if (w <= 1024) {
        // Tablet landscape / small desktop: slightly smaller blocks
        targetCols = 7;
        blockWidth = Math.floor((w - (targetCols + 1) * PADDING) / targetCols);
        targetBlockCount = 13;
      } else {
        // Desktop: more columns, standard blocks
        targetCols = Math.min(10, Math.floor(w / 180));
        blockWidth = Math.floor((w - (targetCols + 1) * PADDING) / targetCols);
        targetBlockCount = 21;
      }
      
      const RECT_WIDTH = blockWidth;
      const RECT_HEIGHT = Math.floor(RECT_WIDTH / ASPECT_RATIO);
      
      // Distribute blocks with varied stacking (20% have 1, 40% have 2, 40% have 3)
      const columnStacks = [];
      
      // Initialize columns with weighted random heights
      for (let i = 0; i < targetCols; i++) {
        const rand = Math.random();
        if (rand < 0.2) {
          columnStacks.push(1); // 20% chance of 1 block
        } else if (rand < 0.6) {
          columnStacks.push(2); // 40% chance of 2 blocks
        } else {
          columnStacks.push(3); // 40% chance of 3 blocks
        }
      }
      
      // Adjust to match target block count
      let currentTotal = columnStacks.reduce((a, b) => a + b, 0);
      
      // Add or remove blocks to reach target
      while (currentTotal !== targetBlockCount) {
        if (currentTotal < targetBlockCount) {
          // Need more blocks - find columns with less than 3
          const availableCols = columnStacks.map((v, i) => v < 3 ? i : -1).filter(i => i >= 0);
          if (availableCols.length === 0) break;
          const col = availableCols[Math.floor(Math.random() * availableCols.length)];
          columnStacks[col]++;
          currentTotal++;
        } else {
          // Too many blocks - find columns with more than 1
          const availableCols = columnStacks.map((v, i) => v > 1 ? i : -1).filter(i => i >= 0);
          if (availableCols.length === 0) break;
          const col = availableCols[Math.floor(Math.random() * availableCols.length)];
          columnStacks[col]--;
          currentTotal--;
        }
      }
      
      // Create all rectangles data
      const allRects = [];
      
      // For each column, create the stacked blocks
      for (let col = 0; col < targetCols; col++) {
        const stackHeight = columnStacks[col];
        const x = PADDING + col * (RECT_WIDTH + PADDING);
        
        // Stack rectangles from bottom up
        for (let i = 0; i < stackHeight; i++) {
          const y = h - (RECT_HEIGHT + PADDING) * (i + 1);
          allRects.push({ x, y });
        }
      }
      
      // Shuffle the rectangles for random appearance
      for (let i = allRects.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [allRects[i], allRects[j]] = [allRects[j], allRects[i]];
      }
      
      // Create all rectangles hidden initially
      const rectElements = [];
      for (const rectData of allRects) {
        const r = document.createElement('div');
        r.className = 'rect';
        r.style.left = rectData.x + 'px';
        r.style.top = rectData.y + 'px';
        r.style.width = RECT_WIDTH + 'px';
        r.style.height = RECT_HEIGHT + 'px';
        r.style.backgroundColor = '#2B7CFF'; // Start blue
        r.style.opacity = '0';
        r.style.transition = 'opacity 0.2s';
        container.appendChild(r);
        rectElements.push(r);
      }
      
      // Animate blocks appearing
      let currentBlock = 0;
      const appearInterval = setInterval(() => {
        if (currentBlock < rectElements.length) {
          rectElements[currentBlock].style.opacity = '1';
          currentBlock++;
        } else {
          clearInterval(appearInterval);
          // All blocks loaded, show content
          showContent();
          // Then change blocks to multi-color
          setTimeout(() => {
            changeToMultiColor(rectElements);
          }, 500);
        }
      }, ANIMATION_DELAY);
    }

    function showContent() {
      const contentEl = document.querySelector('.home-content');
      if (contentEl) {
        contentEl.style.opacity = '1';
      }
    }

    function changeToMultiColor(rectElements) {
      rectElements.forEach((rect, index) => {
        setTimeout(() => {
          const color = PALETTE[Math.floor(Math.random() * PALETTE.length)];
          rect.style.transition = 'background-color 0.3s';
          rect.style.backgroundColor = color;
        }, index * 15); // Fast sequential color change
      });
    }

    buildGrid();

    // Rebuild on resize
    let resizeTimer = 0;
    function onResize() {
      clearTimeout(resizeTimer);
      resizeTimer = setTimeout(() => {
        buildGrid();
      }, 150);
    }

    window.addEventListener('resize', onResize, { passive: true });

    // Expose cleanup if your router unmounts this page
    window.__rectsCleanup = function () {
      window.removeEventListener('resize', onResize);
    };
  }

  // Expose init function globally for re-initialization on navigation
  window.__rectsInit = init;
  
  ready(() => {
    init();
    startObserving();
  });
})();
