use dioxus::prelude::*;

#[component]
pub fn MatrixPage() -> Element {
    rsx! {
        style { {
            r#"
            .matrix-grid {
                position: fixed;
                inset: 0;
                background: #000;
                display: grid;
                grid-template-columns: repeat(17, 1fr);
                grid-template-rows: repeat(17, 1fr);
                grid-auto-flow: row;
                width: 100vw;
                height: 100vh;
                contain: strict;
                --size: min(calc(100vw / 17), calc(100vh / 17));
                --cw: var(--size);
                --ch: var(--size);
            }
            .matrix-grid::before {
                content: '';
                position: absolute;
                inset: 0;
                background-image:
                  repeating-linear-gradient(to right, rgba(255,255,255,0.015) 0, rgba(255,255,255,0.015) 1px, transparent 1px var(--cw)),
                  repeating-linear-gradient(to bottom, rgba(255,255,255,0.015) 0, rgba(255,255,255,0.015) 1px, transparent 1px var(--ch));
                pointer-events: none;
            }
            .cell {
                width: var(--size);
                height: var(--size);
                background: #000;
                will-change: background-color;
            }
            .cell.on {
                background: #ff8800;
                box-shadow: 0 0 8px rgba(255,136,0,0.6), 0 0 16px rgba(255,136,0,0.4);
            }
            "#
        } }
        div { id: "grid", class: "matrix-grid" }
        script { {
            r#"
(function(){
  const root = document.getElementById('grid');
  if (!root || root.__init) return; root.__init = true;

  // Canvas-based Tetris effect (bypass CSS grid ordering)
  try {
    // Clear any existing children/cells
    root.innerHTML = '';

    // Create canvas overlay
    const canvas = document.createElement('canvas');
    canvas.id = 'tetris-canvas';
    canvas.style.position = 'absolute';
    canvas.style.inset = '0';
    canvas.style.width = '100%';
    canvas.style.height = '100%';
    canvas.style.zIndex = '1';
    root.appendChild(canvas);

    const ctx = canvas.getContext('2d');

    const COLS = 17, ROWS = 17;
    let cell = 0;
    let originX = 0, originY = 0;
    const SCALE = 1.8; // increase block size by 80% (relative to base), adjust if needed
    const GAP = 6;     // gap reduced by ~50%
    const MAX_CELL = 90; // max pixel size for a cell to cap growth on large screens
    // Palette and persistent colors per column (no white)
    const PALETTE = ['#DAC8FB', '#FFA5D2', '#049D56', '#FBFB52', '#A4E6F0'];
    const pickColor = () => PALETTE[(Math.random() * PALETTE.length) | 0];
    const bottomColors = new Map(); // col -> color for bottom row
    const secondColors = new Map(); // col -> color for second row
    const thirdColors = new Map();  // col -> color for third row corners

    // Color grouping: 30% chance to reuse neighbor's color for Tetris-like clustering
    function pickColorWithGrouping(colorMap, col){
      if (Math.random() < 0.3){
        // Try to reuse a neighbor's color (left or right)
        const neighbors = [col - 1, col + 1].filter(c => colorMap.has(c));
        if (neighbors.length > 0){
          const neighbor = neighbors[(Math.random() * neighbors.length) | 0];
          return colorMap.get(neighbor);
        }
      }
      return pickColor();
    }

    function resize(){
      const dpr = window.devicePixelRatio || 1;
      const rect = root.getBoundingClientRect();
      canvas.width = Math.floor(rect.width * dpr);
      canvas.height = Math.floor(rect.height * dpr);
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      // Increase block size by 20%
      const baseCell = Math.floor(Math.min(rect.width / COLS, rect.height / ROWS));
      cell = Math.max(1, Math.min(MAX_CELL, Math.floor(baseCell * SCALE)));
      const gridW = COLS * cell;
      const gridH = ROWS * cell;
      // Center horizontally
      originX = Math.floor((rect.width - gridW) / 2);
      // Position so bottom rows are visible (may cut off top rows)
      if (gridH > rect.height) {
        // Grid too tall: prioritize showing bottom
        originY = rect.height - gridH - 10;
      } else {
        // Grid fits: center it
        originY = Math.floor((rect.height - gridH) / 2);
      }
      startFall();
    }
    window.addEventListener('resize', resize);
    resize();

    const index = (c, r) => r * COLS + c;
    let board = new Set();
    let piece = null;

    const SHAPES = {
      O: [[0,0],[1,0],[0,1],[1,1]],
      I: [[0,0],[1,0],[2,0],[3,0]],
      T: [[1,0],[0,1],[1,1],[2,1]],
      L: [[0,0],[0,1],[0,2],[1,2]],
      J: [[1,0],[1,1],[1,2],[0,2]],
      S: [[1,0],[2,0],[0,1],[1,1]],
      Z: [[0,0],[1,0],[1,1],[2,1]],
    };

    function rotateCW(blocks){
      const r = blocks.map(([x,y]) => [y, -x]);
      const minX = Math.min(...r.map(p => p[0]));
      const minY = Math.min(...r.map(p => p[1]));
      return r.map(([x,y]) => [x - minX, y - minY]);
    }

    function randomBlocks(){
      const vals = Object.values(SHAPES);
      let b = vals[(Math.random() * vals.length) | 0].map(p => [p[0], p[1]]);
      const rot = (Math.random() * 4) | 0;
      for (let i=0;i<rot;i++) b = rotateCW(b);
      return b;
    }

    piece = { x: (COLS >> 1) - 2, y: 0, blocks: randomBlocks() };
    render();

    function canPlace(x, y, blocks){
      for (const [bx, by] of blocks){
        const c = x + bx, r = y + by;
        if (c < 0 || c >= COLS) return false;
        if (r >= ROWS) return false;
        if (r >= 0 && board.has(index(c, r))) return false;
      }
      return true;
    }

    function lock(){
      for (const [bx, by] of piece.blocks){
        const c = piece.x + bx, r = piece.y + by;
        if (r >= 0) board.add(index(c, r));
      }
      // Clear full lines
      for (let r = ROWS - 1; r >= 0; r--){
        let full = true;
        for (let c = 0; c < COLS; c++){
          if (!board.has(index(c, r))) { full = false; break; }
        }
        if (full){
          const nb = new Set();
          for (const idx of board){
            const rr = Math.floor(idx / COLS), cc = idx % COLS;
            if (rr < r) nb.add(index(cc, rr + 1));
            else if (rr > r) nb.add(idx);
          }
          board = nb;
          r++;
        }
      }
      piece = { x: (COLS >> 1) - 2, y: -1, blocks: randomBlocks() };
    }

    function drawCell(c, r){
      const gap = Math.min(GAP, Math.max(1, Math.floor(cell - 1))); // ensure tile remains visible
      const x = originX + c * cell;
      const y = originY + r * cell;
      const size = Math.max(1, cell - gap);
      const off = gap / 2;
      ctx.fillRect(Math.round(x + off), Math.round(y + off), Math.max(1, Math.round(size)), Math.max(1, Math.round(size)));
    }

    function drawTwo(){
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = '#ff8800';
      const baseC = Math.floor(COLS / 2) - 1;
      const baseR = Math.floor(ROWS / 2) - 1;
      // original two
      drawCell(baseC, baseR);
      drawCell(baseC + 1, baseR);
      // add top and bottom on the left column (forms a T rotated 90°)
      drawCell(baseC, baseR - 1);
      drawCell(baseC, baseR + 1);
    }

    function drawFloor(){
      // Bottom row spanning full viewport width with 1px gaps, using palette colors (persistent per column)
      const rect = root.getBoundingClientRect();
      let leftC = Math.floor((-originX) / cell);
      let rightC = Math.ceil((rect.width - originX) / cell) - 1;
      if (!Number.isFinite(leftC) || !Number.isFinite(rightC) || leftC > rightC){
        leftC = 0; rightC = COLS - 1;
      }
      const rBottom = ROWS - 1;
      const centerC = Math.floor(COLS / 2);
      for (let c = leftC; c <= rightC; c++){
        // Leave the center column empty so the T stem can fit
        if (c === centerC) continue;
        if (!bottomColors.has(c)) bottomColors.set(c, pickColorWithGrouping(bottomColors, c));
        ctx.fillStyle = bottomColors.get(c);
        drawCell(c, rBottom);
      }
    }

    function drawSecondRowWithHoles(){
      // Full second row from bottom with 3 center holes under the T bar, using palette colors
      const rect = root.getBoundingClientRect();
      let leftC = Math.floor((-originX) / cell);
      let rightC = Math.ceil((rect.width - originX) / cell) - 1;
      if (!Number.isFinite(leftC) || !Number.isFinite(rightC) || leftC > rightC){
        leftC = 0; rightC = COLS - 1;
      }
      const rAbove = ROWS - 2;
      const centerC = Math.floor(COLS / 2);
      for (let c = leftC; c <= rightC; c++){
        if (c === centerC - 1 || c === centerC || c === centerC + 1) continue;
        if (!secondColors.has(c)) secondColors.set(c, pickColorWithGrouping(secondColors, c));
        ctx.fillStyle = secondColors.get(c);
        drawCell(c, rAbove);
      }
    }

    function drawThirdRowCorners(){
      // Draw a few colored cells on the third row from bottom (persistent colors)
      const rect = root.getBoundingClientRect();
      let leftC = Math.floor((-originX) / cell);
      let rightC = Math.ceil((rect.width - originX) / cell) - 1;
      if (!Number.isFinite(leftC) || !Number.isFinite(rightC) || leftC > rightC){
        leftC = 0; rightC = COLS - 1;
      }
      const rThird = ROWS - 3;
      const centerC = Math.floor(COLS / 2);
      const skip = new Set([centerC - 1, centerC, centerC + 1]);
      const candidates = [leftC, rightC, leftC + 2, rightC - 2, leftC + 4, rightC - 4];
      for (const c of candidates){
        if (Number.isFinite(c) && c >= leftC && c <= rightC && !skip.has(c)){
          if (!thirdColors.has(c)) thirdColors.set(c, pickColorWithGrouping(thirdColors, c));
          ctx.fillStyle = thirdColors.get(c);
          drawCell(c, rThird);
        }
      }
    }


    function startFall(){
      // Upside-down T: (-1,0),(0,0),(1,0),(0,1)
      const baseC = Math.floor(COLS / 2); // center horizontally
      const baseR = 0; // top row
      let dy = 0;
      const shapeBottomOffset = 1; // lowest block is 1 row below base
      const maxDy = Math.max(0, (ROWS - 1) - (baseR + shapeBottomOffset));

      if (root.__fallTimer) { clearInterval(root.__fallTimer); root.__fallTimer = null; }

      const drawFrame = () => {
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        drawFloor();
        drawSecondRowWithHoles();
        drawThirdRowCorners();
        ctx.fillStyle = '#ff8800';
        drawCell(baseC - 1, baseR + dy);
        drawCell(baseC,     baseR + dy);
        drawCell(baseC + 1, baseR + dy);
        drawCell(baseC,     baseR + dy + 1);
      };

      drawFrame();
      root.__fallTimer = setInterval(() => {
        drawFrame();
        if (dy >= maxDy){
          dy = 0; // loop
        } else {
          dy++;
        }
      }, 500);
    }
    function drawOne(){
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = '#ff8800';
      const baseC = Math.floor(COLS / 2) - 1;
      const baseR = Math.floor(ROWS / 2) - 1;
      const shape = [[0,0],[1,0],[0,1],[1,1]]; // O piece
      for (const [dx, dy] of shape){
        const c = baseC + dx, r = baseR + dy;
        if (c >= 0 && c < COLS && r >= 0 && r < ROWS) drawCell(c, r);
      }
    }

    function render(){
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = '#ff8800';
      for (const idx of board){
        const r = Math.floor(idx / COLS), c = idx % COLS;
        drawCell(c, r);
      }
      if (piece){
        for (const [bx, by] of piece.blocks){
          const c = piece.x + bx, r = piece.y + by;
          if (r >= 0 && r < ROWS && c >= 0 && c < COLS) drawCell(c, r);
        }
      }
    }

    function startSimpleFall(){
      let y = 0;
      const baseX = Math.floor(COLS / 2) - 1;
      const shape = [[1,0],[0,1],[1,1],[2,1]]; // T shape
      setInterval(() => {
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        ctx.fillStyle = '#ff8800';
        for (const [dx, dy] of shape){
          const c = baseX + dx;
          const r = y + dy;
          if (r >= 0 && r < ROWS && c >= 0 && c < COLS) drawCell(c, r);
        }
        y = (y + 1) % (ROWS - 2);
      }, 500);
    }

    function shuffle(arr){
      for (let i=arr.length-1; i>0; i--){ const j=(Math.random()*(i+1))|0; [arr[i],arr[j]]=[arr[j],arr[i]]; }
      return arr;
    }

    startFall();
  } catch (e) {
    console.error('Canvas Tetris init failed', e);
  }
  return;

  const COLS = 17, ROWS = 17, TOTAL = COLS * ROWS;

  // Build cells (DOM order may vary by browser writing-mode; derive row/col by measuring positions)
  let indexMap = Array.from({ length: ROWS }, () => Array(COLS).fill(-1));
  if (root.children.length !== TOTAL){
    root.innerHTML = '';
    const frag = document.createDocumentFragment();
    for (let i = 0; i < TOTAL; i++){
      const d = document.createElement('div');
      d.className = 'cell';
      frag.appendChild(d);
    }
    root.appendChild(frag);
  }
  const cells = root.children;

  function rebuildIndexMap(){
    const items = [];
    for (let i = 0; i < cells.length; i++){
      const rect = cells[i].getBoundingClientRect();
      items.push({ i, top: Math.round(rect.top), left: Math.round(rect.left) });
    }
    const ys = Array.from(new Set(items.map(p => p.top))).sort((a,b)=>a-b).slice(0, ROWS);
    const xs = Array.from(new Set(items.map(p => p.left))).sort((a,b)=>a-b).slice(0, COLS);
    const map = Array.from({ length: ROWS }, () => Array(COLS).fill(-1));
    for (const p of items){
      const r = ys.indexOf(p.top);
      const c = xs.indexOf(p.left);
      if (r >= 0 && r < ROWS && c >= 0 && c < COLS){
        map[r][c] = p.i;
      }
    }
    // Fallback: if any -1 remains, fall back to row-major guess
    let hasHole = false;
    outer: for (let r=0;r<ROWS;r++) for (let c=0;c<COLS;c++) if (map[r][c] === -1){ hasHole = true; break outer; }
    if (hasHole){
      let k = 0;
      for (let r=0;r<ROWS;r++) for (let c=0;c<COLS;c++) map[r][c] = k++;
    }
    indexMap = map;
  }

  // Initial mapping and on resize
  rebuildIndexMap();
  window.addEventListener('resize', () => {
    // debounce
    clearTimeout(window.__grid_map_timer);
    window.__grid_map_timer = setTimeout(rebuildIndexMap, 100);
  });

  // Helpers
  function idxAt(c, r){
    if (r < 0 || r >= ROWS || c < 0 || c >= COLS) return -1;
    return indexMap[r][c];
  }

  let prevOn = new Set();
  function renderSet(set){
    for (const i of prevOn){ if (!set.has(i)) cells[i].classList.remove('on'); }
    for (const i of set){ if (!prevOn.has(i)) cells[i].classList.add('on'); }
    prevOn = set;
  }

  // Base shapes
  const BASE = {
    O: [[0,0],[1,0],[0,1],[1,1]],
    I: [[0,0],[1,0],[2,0],[3,0]],
    T: [[0,0],[1,0],[2,0],[1,1]],
    L: [[0,0],[0,1],[0,2],[1,2]],
    J: [[1,0],[1,1],[1,2],[0,2]],
    S: [[1,0],[2,0],[0,1],[1,1]],
    Z: [[0,0],[1,0],[1,1],[2,1]]
  };

  function rotateCW(blocks){
    // (x, y) -> (y, -x), then normalize to start at (0,0)
    const r = blocks.map(([x,y]) => [y, -x]);
    const minX = Math.min(...r.map(p => p[0]));
    const minY = Math.min(...r.map(p => p[1]));
    return r.map(([x,y]) => [x - minX, y - minY]);
  }

  function orientations(base){
    const outs = [base];
    for (let i=0; i<3; i++) outs.push(rotateCW(outs[outs.length - 1]));
    // Deduplicate identical rotations
    const key = b => b.map(([x,y]) => `${x},${y}`).sort().join('|');
    const seen = new Set();
    const uniq = [];
    for (const b of outs){
      const k = key(b);
      if (!seen.has(k)){ seen.add(k); uniq.push(b); }
    }
    return uniq;
  }

  const SHAPES = Object.values(BASE).map(b => orientations(b));

  function bounds(blocks){
    const maxX = Math.max(...blocks.map(([x]) => x));
    const maxY = Math.max(...blocks.map(([,y]) => y));
    return { w: maxX + 1, h: maxY + 1 };
  }

  const settled = new Set();
  let cur = null; // { blocks, x, y }

  function canPlace(blocks, x, y){
    for (const [bx, by] of blocks){
      const c = x + bx, r = y + by;
      if (c < 0 || c >= COLS) return false;
      if (r < 0) continue; // allow spawning slightly above
      if (r >= ROWS) return false;
      if (settled.has(idxAt(c, r))) return false;
    }
    return true;
  }

  function lock(){
    for (const [bx, by] of cur.blocks){
      const c = cur.x + bx, r = cur.y + by;
      if (r >= 0 && r < ROWS && c >= 0 && c < COLS){
        settled.add(idxAt(c, r));
      }
    }
    cur = null;
    // Optional: clear full rows
    for (let r = ROWS - 1; r >= 0; r--){
      let full = true;
      for (let c = 0; c < COLS; c++){
        if (!settled.has(idxAt(c, r))) { full = false; break; }
      }
      if (full){
        // drop everything above down by 1
        const newSet = new Set();
        for (const i of settled){
          const row = Math.floor(i / COLS);
          const col = i % COLS;
          if (row < r) newSet.add(idxAt(col, row + 1));
          else if (row > r) newSet.add(i);
          // skip row r
        }
        settled.clear();
        for (const i of newSet) settled.add(i);
        r++; // re-check this row after collapse
      }
    }
  }

  function spawn(){
    const shape = SHAPES[(Math.random() * SHAPES.length) | 0];
    const blocks = shape[(Math.random() * shape.length) | 0];
    const { w } = bounds(blocks);
    const x = Math.max(0, Math.min(COLS - w, (COLS - w) >> 1));
    const y = -1; // start slightly above
    cur = { blocks, x, y };
    if (!canPlace(cur.blocks, cur.x, cur.y + 1)){
      // board overflow -> reset
      settled.clear();
    }
  }

  function tick(){
    if (!cur) spawn();
    if (canPlace(cur.blocks, cur.x, cur.y + 1)){
      cur.y++;
    } else {
      lock();
    }
    const display = new Set(settled);
    if (cur){
      for (const [bx, by] of cur.blocks){
        const c = cur.x + bx, r = cur.y + by;
        if (r >= 0 && r < ROWS && c >= 0 && c < COLS) display.add(idxAt(c, r));
      }
    }
    renderSet(display);
  }

  const TICK = 500;
  // Ensure layout is ready before first tick (so indexMap is accurate)
  setTimeout(tick, 0);
  setInterval(tick, TICK);
})();
            "#
        } }
    }
}
