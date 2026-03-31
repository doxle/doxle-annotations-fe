window.__ourstoryInit = function() {
    const container = document.getElementById('ourstory-blocks-container');
    if (!container) return;
    
    const ASPECT_RATIO = 151 / 64;
    const PADDING = 2;
    const PALETTE = ['#DAC8FB', '#FFA5D2', '#049D56', '#FBFB52', '#A4E6F0'];
    const RED = '#FF3B30';
    
    function buildBuildings() {
        container.innerHTML = '';
        
        // Simple: use window dimensions
        const w = window.innerWidth;
        const h = window.innerHeight;
        
        // Calculate columns based on width
        let targetCols;
        if (w <= 480) {
            targetCols = 5;
        } else if (w <= 768) {
            targetCols = 7;
        } else if (w <= 1024) {
            targetCols = 9;
        } else {
            targetCols = Math.min(12, Math.floor(w / 150));
        }
        
        const blockWidth = Math.floor((w - (targetCols + 1) * PADDING) / targetCols);
        const blockHeight = Math.floor(blockWidth / ASPECT_RATIO);
        
        // Max building height (in blocks)
        const maxHeight = w <= 480 ? 3 : 4;
        
        // Create building configurations
        const buildings = [];
        for (let col = 0; col < targetCols; col++) {
            const rand = Math.random();
            let height;
            if (rand < 0.25) {
                height = 1;
            } else if (rand < 0.5) {
                height = 2;
            } else if (rand < 0.8) {
                height = 3;
            } else {
                height = maxHeight;
            }
            buildings.push({
                col: col,
                height: height,
                x: PADDING + col * (blockWidth + PADDING)
            });
        }
        
        // Create blocks - position from bottom using CSS bottom property
        const allBlocks = [];
        
        buildings.forEach(building => {
            for (let row = 0; row < building.height; row++) {
                // Calculate position from bottom
                const bottomPos = row * (blockHeight + PADDING);
                allBlocks.push({
                    x: building.x,
                    bottom: bottomPos,
                    col: building.col,
                    row: row,
                    building: building
                });
            }
        });
        
        // Sort by row (bottom first) for stacking animation
        allBlocks.sort((a, b) => {
            if (a.row !== b.row) return a.row - b.row;
            return a.col - b.col;
        });
        
        // Create DOM elements
        const blockElements = [];
        allBlocks.forEach(blockData => {
            const block = document.createElement('div');
            block.className = 'ourstory-block';
            block.style.position = 'absolute';
            block.style.left = blockData.x + 'px';
            block.style.bottom = blockData.bottom + 'px';
            block.style.width = blockWidth + 'px';
            block.style.height = blockHeight + 'px';
            block.style.backgroundColor = RED;
            block.style.opacity = '0';
            block.style.transform = 'translateY(30px)';
            block.style.transition = 'opacity 0.3s ease, transform 0.3s ease';
            container.appendChild(block);
            blockElements.push({ el: block, data: blockData });
        });
        
        // Animate blocks appearing
        let currentIndex = 0;
        const blocksPerFrame = Math.max(1, Math.floor(targetCols / 3));
        
        const animateInterval = setInterval(() => {
            for (let i = 0; i < blocksPerFrame && currentIndex < blockElements.length; i++) {
                const block = blockElements[currentIndex];
                block.el.style.opacity = '1';
                block.el.style.transform = 'translateY(0)';
                currentIndex++;
            }
            
            if (currentIndex >= blockElements.length) {
                clearInterval(animateInterval);
                
                // Transition to colorful
                setTimeout(() => {
                    blockElements.forEach((block, index) => {
                        setTimeout(() => {
                            const color = PALETTE[Math.floor(Math.random() * PALETTE.length)];
                            block.el.style.transition = 'background-color 0.4s ease';
                            block.el.style.backgroundColor = color;
                        }, index * 20);
                    });
                }, 800);
            }
        }, 40);
    }
    
    // Small delay to ensure DOM is ready
    setTimeout(buildBuildings, 100);
    
    // Handle resize
    let resizeTimer;
    const onResize = () => {
        clearTimeout(resizeTimer);
        resizeTimer = setTimeout(buildBuildings, 200);
    };
    
    if (window.__ourstoryResizeHandler) {
        window.removeEventListener('resize', window.__ourstoryResizeHandler);
    }
    window.__ourstoryResizeHandler = onResize;
    window.addEventListener('resize', onResize, { passive: true });
};
