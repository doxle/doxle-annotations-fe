(function() {
  function ready(fn) {
    if (document.readyState !== 'loading') fn();
    else document.addEventListener('DOMContentLoaded', fn, { once: true });
  }

  function initTypewriter() {
    // Find the home page titles
    const titles = document.querySelectorAll('.home-title, .home-subtitle');
    if (!titles.length) {
      // Try again if elements not found (SPA navigation)
      setTimeout(initTypewriter, 100);
      return;
    }

    // Store original text and clear elements
    const elements = [];
    titles.forEach((el, index) => {
      const text = el.textContent;
      const hasSpan = el.querySelector('span');
      
      if (hasSpan) {
        // Handle "SO YOU BUILD" which has BUILD in a span
        const beforeSpan = el.childNodes[0].textContent;
        const spanText = el.querySelector('span').textContent;
        elements.push({
          element: el,
          fullText: beforeSpan + spanText,
          beforeSpan: beforeSpan,
          spanText: spanText,
          hasSpan: true
        });
      } else {
        elements.push({
          element: el,
          fullText: text,
          hasSpan: false
        });
      }
      
      // Clear all elements
      el.innerHTML = '';
      el.classList.add('typewriter-text');
    });

    // Add single cursor to first element only
    if (elements.length > 0) {
      elements[0].element.innerHTML = '<span class="typewriter-cursor">_</span>';
    }

    // Start typing after cursor blinks 3 times
    setTimeout(() => {
      typeNextElement(elements, 0);
    }, 1500); // Wait for cursor to blink 3 times
  }

  function typeNextElement(elements, index) {
    if (index >= elements.length) {
      // All done - just wait then restart
      setTimeout(() => {
        // Clear everything and start from beginning
        elements.forEach(item => {
          item.element.innerHTML = '';
        });
        if (elements.length > 0) {
          elements[0].element.innerHTML = '<span class="typewriter-cursor">_</span>';
        }
        setTimeout(() => {
          typeNextElement(elements, 0);
        }, 1500);
      }, 120000); // Wait 120 seconds then restart
      return;
    }
    
    const item = elements[index];
    const el = item.element;
    let charIndex = 0;
    
    // Clear element (remove cursor if it's there)
    el.innerHTML = '';
    
    function typeChar() {
      if (charIndex < item.fullText.length) {
        if (item.hasSpan && charIndex === item.beforeSpan.length) {
          // Start the span for BUILD
          el.innerHTML = item.beforeSpan + '<span>' + item.spanText.substring(0, charIndex - item.beforeSpan.length + 1) + '</span><span class="typewriter-cursor">_</span>';
        } else if (item.hasSpan && charIndex > item.beforeSpan.length) {
          // Continue typing in span
          el.innerHTML = item.beforeSpan + '<span>' + item.spanText.substring(0, charIndex - item.beforeSpan.length + 1) + '</span><span class="typewriter-cursor">_</span>';
        } else {
          // Normal typing
          el.innerHTML = item.fullText.substring(0, charIndex + 1) + '<span class="typewriter-cursor">_</span>';
        }
        charIndex++;
        setTimeout(typeChar, 80); // Slower typing speed
      } else {
        // Finished typing this element
        // Remove cursor and pause before next line
        setTimeout(() => {
          if (item.hasSpan) {
            el.innerHTML = item.beforeSpan + '<span>' + item.spanText + '</span>';
          } else {
            el.textContent = item.fullText;
          }
          // Add pause between lines
          setTimeout(() => {
            typeNextElement(elements, index + 1);
          }, 800); // Noticeable pause between lines
        }, 200);
      }
    }
    
    typeChar();
  }

  // Add CSS for cursor
  function injectCSS() {
    if (document.getElementById('typewriter-style')) return;
    const css = `
      .typewriter-cursor {
        display: inline-block;
        animation: blink 0.5s infinite;
        font-weight: 400;
        color: #ff0000;
        margin-left: 2px;
      }
      @keyframes blink {
        0%, 49% { opacity: 1; }
        50%, 100% { opacity: 0; }
      }
    `;
    const style = document.createElement('style');
    style.id = 'typewriter-style';
    style.textContent = css;
    document.head.appendChild(style);
  }

  ready(function() {
    injectCSS();
    initTypewriter();
    
    // Re-run on navigation for SPA
    let lastPath = window.location.pathname;
    const observer = new MutationObserver(() => {
      if (window.location.pathname !== lastPath) {
        lastPath = window.location.pathname;
        if (window.location.pathname === '/') {
          setTimeout(initTypewriter, 100);
        }
      }
    });
    observer.observe(document.body, { childList: true, subtree: true });
  });
})();