window.__dotsInit = function() {
    const container = document.getElementById('dots-container');
    if (!container) return;

    // Cleanup previous instance
    if (window.__dotsCleanup) window.__dotsCleanup();

    const isMobile = window.innerWidth <= 768 || 'ontouchstart' in window || navigator.maxTouchPoints > 0;
    const dpr = Math.min(window.devicePixelRatio || 1, 2);

    // Setup canvas
    container.innerHTML = '';
    const canvas = document.createElement('canvas');
    canvas.style.cssText = 'width:100%;height:100%;display:block;';
    container.appendChild(canvas);
    const ctx = canvas.getContext('2d');

    function resize() {
        const rect = container.getBoundingClientRect();
        canvas.width = Math.round(rect.width * dpr);
        canvas.height = Math.round(rect.height * dpr);
        ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
        return rect;
    }
    const rect = resize();
    let W = rect.width;
    let H = rect.height;
    let offsetX = rect.left;
    let offsetY = rect.top;

    // Grid
    const spacing = 30;
    let cols, rows, count, baseX, baseY, phase;

    function buildGrid() {
        cols = Math.ceil(W / spacing) + 1;
        rows = Math.ceil(H / spacing) + 2;
        count = cols * rows;
        baseX = new Float32Array(count);
        baseY = new Float32Array(count);
        phase = new Float32Array(count);
        let i = 0;
        for (let x = 0; x < cols; x++) {
            for (let y = 0; y < rows; y++) {
                baseX[i] = x * spacing;
                baseY[i] = y * spacing;
                phase[i] = Math.random() * Math.PI * 2;
                i++;
            }
        }
    }
    buildGrid();

    // Mouse state
    let mx = -9999, my = -9999;
    const pushRadius = 180;
    const r2 = pushRadius * pushRadius;

    // Animation state
    let running = true;
    let opacity = 0; // for fade-in
    let startTime = 0;

    function draw(now) {
        if (!running) return;
        requestAnimationFrame(draw);

        if (!startTime) startTime = now;
        const t = (now - startTime) / 1000; // seconds

        ctx.clearRect(0, 0, W, H);
        ctx.fillStyle = `rgba(255,255,255,${0.4 * opacity})`;
        ctx.beginPath();

        for (let k = 0; k < count; k++) {
            // Breeze: gentle sine wave offset
            const p = phase[k];
            const breezeX = Math.sin(t * 0.785 + p) * 2;        // ~8s period
            const breezeY = Math.cos(t * 0.785 + p + 1.5) * 1.8;

            let px = baseX[k] + breezeX;
            let py = baseY[k] + breezeY;

            // Cursor push (desktop only — mx stays at -9999 on mobile)
            const dx = baseX[k] - mx;
            const dy = baseY[k] - my;
            const dist2 = dx * dx + dy * dy;
            if (dist2 < r2) {
                const dist = Math.sqrt(dist2);
                const strength = ((pushRadius - dist) / pushRadius) * 50;
                const inv = dist > 0 ? 1 / dist : 0;
                px = baseX[k] + dx * inv * strength;
                py = baseY[k] + dy * inv * strength;
            }

            ctx.moveTo(px + 1.5, py);
            ctx.arc(px, py, 1.5, 0, 6.2832);
        }

        ctx.fill();
    }

    requestAnimationFrame(draw);

    // Mouse listener (desktop only)
    function onMouseMove(e) { mx = e.clientX - offsetX; my = e.clientY - offsetY; }
    function resetMouse() { mx = -9999; my = -9999; }
    function onVisibilityChange() { if (document.hidden) resetMouse(); }
    function onDocumentMouseOut(e) { if (!e.relatedTarget && !e.toElement) resetMouse(); }
    if (!isMobile) {
        document.addEventListener('mousemove', onMouseMove, { passive: true });
        container.addEventListener('mouseleave', resetMouse, { passive: true });
        document.addEventListener('mouseout', onDocumentMouseOut, { passive: true });
        window.addEventListener('mouseleave', resetMouse, { passive: true });
        window.addEventListener('blur', resetMouse, { passive: true });
        document.addEventListener('visibilitychange', onVisibilityChange, { passive: true });
    }

    // Resize handler
    let resizeTimer = 0;
    let resizeFrame = 0;
    function onResize() {
        if (resizeFrame) cancelAnimationFrame(resizeFrame);
        resizeFrame = requestAnimationFrame(() => {
            const r = resize();
            W = r.width;
            H = r.height;
            offsetX = r.left;
            offsetY = r.top;
        });
        clearTimeout(resizeTimer);
        resizeTimer = setTimeout(() => {
            buildGrid();
        }, 150);
    }
    window.addEventListener('resize', onResize, { passive: true });

    // Cleanup
    window.__dotsCleanup = function() {
        running = false;
        document.removeEventListener('mousemove', onMouseMove);
        container.removeEventListener('mouseleave', resetMouse);
        document.removeEventListener('mouseout', onDocumentMouseOut);
        window.removeEventListener('mouseleave', resetMouse);
        window.removeEventListener('blur', resetMouse);
        document.removeEventListener('visibilitychange', onVisibilityChange);
        window.removeEventListener('resize', onResize);
    };

    // Expose fade-in control for sequence
    window.__dotsFadeIn = function(duration) {
        const start = performance.now();
        function tick(now) {
            const p = Math.min(1, (now - start) / duration);
            opacity = p;
            if (p < 1) requestAnimationFrame(tick);
        }
        requestAnimationFrame(tick);
    };

    // Start the sequence
    startHomeSequence();
};

function startHomeSequence() {
    const titleLines = document.querySelectorAll('.home-title-line');
    const description = document.querySelector('.home-description');
    const uploadButton = document.querySelector('.upload-button');
    const nav = document.querySelector('.home-navbar');

    // Hide everything initially but reserve space
    titleLines.forEach((line, index) => {
        line.dataset.fullText = line.textContent;
        if (index === 0) {
            line.textContent = '';
        } else {
            line.innerHTML = '&nbsp;';
            line.style.visibility = 'hidden';
        }
    });
    if (description) description.style.opacity = '0';
    if (uploadButton) uploadButton.style.opacity = '0';

    function typewrite(element, text, callback, showCursorFirst) {
        let i = 0;
        element.innerHTML = '<span class="typewriter-cursor">_</span>';
        function type() {
            if (i < text.length) {
                element.innerHTML = text.substring(0, i + 1) + '<span class="typewriter-cursor">_</span>';
                i++;
                setTimeout(type, 60);
            } else {
                element.textContent = text;
                if (callback) setTimeout(callback, 200);
            }
        }
        if (showCursorFirst) {
            setTimeout(type, 1000);
        } else {
            type();
        }
    }

    function fadeIn(element, duration) {
        if (!element) return;
        element.style.transition = `opacity ${duration}ms ease`;
        element.style.opacity = '1';
    }

    const homeContent = document.querySelector('.home-content');
    if (homeContent) homeContent.style.opacity = '1';

    if (titleLines[0]) {
        typewrite(titleLines[0], titleLines[0].dataset.fullText, () => {
            if (titleLines[1]) {
                titleLines[1].style.visibility = 'visible';
                typewrite(titleLines[1], titleLines[1].dataset.fullText, () => {
                    setTimeout(() => {
                        fadeIn(description, 1500);
                        fadeIn(uploadButton, 1500);
                        if (nav) fadeIn(nav, 1500);
                        // Fade in dots via canvas opacity
                        if (window.__dotsFadeIn) window.__dotsFadeIn(1500);
                    }, 400);
                });
            }
        }, true);
    }
};
