window.__dotsInit = function() {
    const dotsContainer = document.getElementById('dots-container');
    if (!dotsContainer) return;
    
    // Detect mobile/touch device
    const isMobile = window.innerWidth <= 768 || 'ontouchstart' in window || navigator.maxTouchPoints > 0;
    
    // Clear existing dots
    dotsContainer.innerHTML = '';
    
    const spacing = 30;
    const dotsX = Math.ceil(window.innerWidth / spacing) + 2;
    const dotsY = Math.ceil(window.innerHeight / spacing) + 2;
    
    // Create dots (but keep them hidden initially)
    for (let x = 0; x < dotsX; x++) {
        for (let y = 0; y < dotsY; y++) {
            const dot = document.createElement('div');
            dot.className = 'dot';
            dot.style.left = (x * spacing) + 'px';
            dot.style.top = (y * spacing) + 'px';
            dot.dataset.baseX = x * spacing;
            dot.dataset.baseY = y * spacing;
            dot.style.opacity = '0';
            
            // Add breeze animation class with random delay (both mobile and desktop)
            dot.classList.add('dot-breeze');
            dot.style.animationDelay = (Math.random() * 4) + 's';
            
            dotsContainer.appendChild(dot);
        }
    }
    
    // Only add mouse tracking on desktop
    if (!isMobile) {
        // Handle mouse movement
        const handleMouseMove = (e) => {
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
                    dot.style.animation = 'none';
                } else {
                    dot.style.transform = '';
                    dot.style.animation = '';
                }
            });
        };
        
        // Remove old listener if exists
        if (window.__dotsMouseHandler) {
            document.removeEventListener('mousemove', window.__dotsMouseHandler);
        }
        
        // Add new listener to document for smooth tracking
        window.__dotsMouseHandler = handleMouseMove;
        document.addEventListener('mousemove', handleMouseMove);
    } else {
        // Clean up any existing mouse handler on mobile
        if (window.__dotsMouseHandler) {
            document.removeEventListener('mousemove', window.__dotsMouseHandler);
            window.__dotsMouseHandler = null;
        }
    }
    
    // Start the sequence
    startHomeSequence();
};

function startHomeSequence() {
    const titleLines = document.querySelectorAll('.home-title-line');
    const description = document.querySelector('.home-description');
    const uploadButton = document.querySelector('.upload-button');
    const dots = document.querySelectorAll('.dot');
    const nav = document.querySelector('.home-navbar');
    
    // Hide everything initially but reserve space
    titleLines.forEach((line, index) => {
        line.dataset.fullText = line.textContent;
        if (index === 0) {
            line.textContent = '';
        } else {
            // Reserve space with invisible placeholder
            line.innerHTML = '&nbsp;';
            line.style.visibility = 'hidden';
        }
    });
    if (description) description.style.opacity = '0';
    if (uploadButton) uploadButton.style.opacity = '0';
    
    // Typewriter function with red cursor
    function typewrite(element, text, callback, showCursorFirst) {
        let i = 0;
        element.innerHTML = '<span class="typewriter-cursor">_</span>';
        
        function type() {
            if (i < text.length) {
                element.innerHTML = text.substring(0, i + 1) + '<span class="typewriter-cursor">_</span>';
                i++;
                setTimeout(type, 60);
            } else {
                // Remove cursor when done
                element.textContent = text;
                if (callback) setTimeout(callback, 200);
            }
        }
        
        if (showCursorFirst) {
            // Blink cursor twice before typing (2 blinks = 1000ms at 500ms per blink)
            setTimeout(type, 1000);
        } else {
            type();
        }
    }
    
    // Fade in function
    function fadeIn(element, duration) {
        if (!element) return;
        element.style.transition = `opacity ${duration}ms ease`;
        element.style.opacity = '1';
    }
    
    // Show title immediately (for typewriter visibility)
    const homeContent = document.querySelector('.home-content');
    if (homeContent) homeContent.style.opacity = '1';
    
    // Sequence: typewrite "Building", then "Intelligence.", then fade in rest
    if (titleLines[0]) {
        typewrite(titleLines[0], titleLines[0].dataset.fullText, () => {
            if (titleLines[1]) {
                titleLines[1].style.visibility = 'visible';
                typewrite(titleLines[1], titleLines[1].dataset.fullText, () => {
                        // After typewriter done, fade in everything else
                        setTimeout(() => {
                            fadeIn(description, 1500);
                            fadeIn(uploadButton, 1500);
                            if (nav) fadeIn(nav, 1500);
                            
                            // Fade in dots
                            dots.forEach(dot => {
                                dot.style.transition = 'opacity 1500ms ease';
                                dot.style.opacity = '1';
                            });
                    }, 400);
                });
            }
        }, true); // true = show cursor blinking first
    }
};
