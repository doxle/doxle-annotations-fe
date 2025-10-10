document.addEventListener('DOMContentLoaded', function() {
    const dotsContainer = document.getElementById('dots-container');
    if (!dotsContainer) return;
    
    const dotsX = 60;
    const dotsY = 35;
    const spacing = 30;
    
    // Create dots
    for (let x = 0; x < dotsX; x++) {
        for (let y = 0; y < dotsY; y++) {
            const dot = document.createElement('div');
            dot.className = 'dot';
            dot.style.left = (x * spacing) + 'px';
            dot.style.top = (y * spacing) + 'px';
            dot.dataset.baseX = x * spacing;
            dot.dataset.baseY = y * spacing;
            dotsContainer.appendChild(dot);
        }
    }
    
    // Handle mouse movement
    dotsContainer.addEventListener('mousemove', (e) => {
        const mx = e.clientX;
        const my = e.clientY;
        const pushRadius = 180;
        
        const dots = dotsContainer.querySelectorAll('.dot');
        dots.forEach(dot => {
            const baseX = parseFloat(dot.dataset.baseX);
            const baseY = parseFloat(dot.dataset.baseY);
            
            const dx = baseX - mx;
            const dy = baseY - my;
            const distance = Math.sqrt(dx * dx + dy * dy);
            
            if (distance < pushRadius) {
                const pushStrength = ((pushRadius - distance) / pushRadius) * 50;
                const offsetX = distance > 0 ? (dx / distance) * pushStrength : 0;
                const offsetY = distance > 0 ? (dy / distance) * pushStrength : 0;
                
                dot.style.transform = `translate(${offsetX}px, ${offsetY}px)`;
            } else {
                dot.style.transform = 'translate(0px, 0px)';
            }
        });
    });
});
