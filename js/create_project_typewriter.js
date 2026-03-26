(function() {
  function init() {
    var el = document.getElementById('create-project-input');
    if (!el) return;
    if (el.dataset.twDone) return;
    el.dataset.twDone = '1';

    var parts = [
      { text: 'enter ', bold: false },
      { text: 'project', bold: true },
      { text: ' name', bold: false }
    ];
    var fullText = parts.map(function(p) { return p.text; }).join('');
    var charIndex = 0;
    var isPlaceholder = true;
    var isAnimatingPlaceholder = true;
    var IDLE_CURSOR_Y_OFFSET = 3;
    var INLINE_CURSOR = '<span class="cp-inline-cursor">_</span>';
    var idleCursor = null;
    var liveCursor = null;

    el.innerHTML = '';
    el.classList.add('cp-placeholder');
    el.focus();

    function buildHTML(count) {
      var html = '';
      var done = 0;
      for (var i = 0; i < parts.length; i++) {
        var p = parts[i];
        var remaining = count - done;
        if (remaining <= 0) break;
        var chars = p.text.substring(0, Math.min(remaining, p.text.length));
        html += p.bold ? '<b>' + chars + '</b>' : chars;
        done += p.text.length;
      }
      return html;
    }

    function syncHidden() {
      var text = isPlaceholder ? '' : (el.textContent || '');
      var hidden = document.getElementById('create-project-hidden');
      if (hidden) {
        var setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
        setter.call(hidden, text);
        hidden.dispatchEvent(new Event('input', { bubbles: true }));
      }
    }

    function placeCaretAtStart() {
      el.focus();
      var sel = window.getSelection();
      if (!sel) return;
      var range = document.createRange();
      range.selectNodeContents(el);
      range.collapse(true);
      sel.removeAllRanges();
      sel.addRange(range);
    }

    function insertTextAtCaret(text) {
      var sel = window.getSelection();
      if (!sel) return;
      if (sel.rangeCount === 0) {
        placeCaretAtStart();
        sel = window.getSelection();
        if (!sel || sel.rangeCount === 0) return;
      }
      var range = sel.getRangeAt(0);
      range.deleteContents();
      var node = document.createTextNode(text);
      range.insertNode(node);
      range.setStartAfter(node);
      range.collapse(true);
      sel.removeAllRanges();
      sel.addRange(range);
    }

    function ensureIdleCursor() {
      if (idleCursor || !el.parentNode) return;
      idleCursor = document.createElement('span');
      idleCursor.className = 'cp-idle-cursor';
      idleCursor.textContent = '_';
      el.parentNode.appendChild(idleCursor);
    }

    function hideIdleCursor() {
      if (idleCursor) idleCursor.style.display = 'none';
    }

    function showIdleCursor() {
      ensureIdleCursor();
      if (idleCursor) idleCursor.style.display = 'inline-block';
    }

    function positionIdleCursor() {
      if (!isPlaceholder || isAnimatingPlaceholder) {
        hideIdleCursor();
        return;
      }
      showIdleCursor();
      if (!idleCursor || !el.parentNode) return;

      var parentRect = el.parentNode.getBoundingClientRect();
      var inputRect = el.getBoundingClientRect();
      var left = inputRect.left - parentRect.left;
      var top = inputRect.top - parentRect.top;

      if (el.firstChild) {
        var range = document.createRange();
        range.setStart(el.firstChild, 0);
        range.collapse(true);
        var rects = range.getClientRects();
        if (rects.length > 0) {
          left = rects[0].left - parentRect.left;
          top = rects[0].top - parentRect.top;
        }
      }

      idleCursor.style.left = left + 'px';
      idleCursor.style.top = (top + IDLE_CURSOR_Y_OFFSET) + 'px';
    }

    function ensureLiveCursor() {
      if (liveCursor || !el.parentNode) return;
      liveCursor = document.createElement('span');
      liveCursor.className = 'cp-live-cursor';
      liveCursor.textContent = '_';
      el.parentNode.appendChild(liveCursor);
    }

    function hideLiveCursor() {
      if (liveCursor) liveCursor.style.display = 'none';
    }

    function showLiveCursor() {
      ensureLiveCursor();
      if (liveCursor) liveCursor.style.display = 'inline-block';
    }

    function positionLiveCursor() {
      if (isPlaceholder) {
        hideLiveCursor();
        positionIdleCursor();
        return;
      }
      hideIdleCursor();
      showLiveCursor();
      if (!liveCursor || !el.parentNode) return;

      var parentRect = el.parentNode.getBoundingClientRect();
      var inputRect = el.getBoundingClientRect();
      var sel = window.getSelection();
      var left = inputRect.left - parentRect.left;
      var top = inputRect.top - parentRect.top;

      if (sel && sel.rangeCount > 0 && el.contains(sel.anchorNode)) {
        var range = sel.getRangeAt(0).cloneRange();
        range.collapse(true);
        var rects = range.getClientRects();
        if (rects.length > 0) {
          left = rects[0].left - parentRect.left;
          top = rects[0].top - parentRect.top;
        }
      }

      var minLeft = inputRect.left - parentRect.left;
      var maxLeft = inputRect.right - parentRect.left - 2;
      if (left < minLeft) left = minLeft;
      if (left > maxLeft) left = maxLeft;

      liveCursor.style.left = left + 'px';
      liveCursor.style.top = top + 'px';
    }

    function renderIdlePlaceholder() {
      hideLiveCursor();
      isAnimatingPlaceholder = false;
      el.classList.add('cp-placeholder');
      el.innerHTML = buildHTML(fullText.length);
      el.scrollLeft = 0;
      requestAnimationFrame(positionIdleCursor);
    }

    function clearPlaceholder() {
      if (!isPlaceholder) return;
      isPlaceholder = false;
      isAnimatingPlaceholder = false;
      el.innerHTML = '';
      el.classList.remove('cp-placeholder');
      el.style.fontWeight = '300';
      el.scrollLeft = 0;
      hideIdleCursor();
      placeCaretAtStart();
      requestAnimationFrame(positionLiveCursor);
    }

    // Typewriter
    setTimeout(function typeChar() {
      if (!isPlaceholder) return;
      if (charIndex < fullText.length) {
        charIndex++;
        el.innerHTML = buildHTML(charIndex) + INLINE_CURSOR;
        setTimeout(typeChar, 50);
      } else {
        // Done — keep cursor at end of "enter project name"
        renderIdlePlaceholder();
      }
    }, 300);

    el.addEventListener('beforeinput', function() {
      clearPlaceholder();
    });

    el.addEventListener('keydown', function(e) {
      if (e.key === 'Enter') {
        e.preventDefault();
        var text = isPlaceholder ? '' : el.textContent.trim();
        if (text.length < 3) return;
        el.dispatchEvent(new CustomEvent('projectsubmit', { detail: text, bubbles: true }));
        return;
      }
      if (isPlaceholder && !e.altKey && !e.ctrlKey && !e.metaKey && e.key.length === 1) {
        e.preventDefault();
        clearPlaceholder();
        placeCaretAtStart();
        insertTextAtCaret(e.key);
        syncHidden();
        requestAnimationFrame(positionLiveCursor);
        return;
      }
      setTimeout(positionLiveCursor, 0);
    });

    el.addEventListener('input', function() {
      if (isPlaceholder) return;
      var text = el.textContent || '';
      if (text.length === 0) {
        isPlaceholder = true;
        charIndex = fullText.length;
        renderIdlePlaceholder();
      } else {
        el.classList.remove('cp-placeholder');
        requestAnimationFrame(positionLiveCursor);
      }
      syncHidden();
    });

    el.addEventListener('mousedown', function(e) {
      if (isPlaceholder) {
        e.preventDefault();
        clearPlaceholder();
      }
      else setTimeout(positionLiveCursor, 0);
    });

    el.addEventListener('focus', function() {
      if (isPlaceholder) requestAnimationFrame(positionIdleCursor);
      else requestAnimationFrame(positionLiveCursor);
    });

    el.addEventListener('blur', function() {
      hideLiveCursor();
      if (!isPlaceholder) hideIdleCursor();
    });

    el.addEventListener('keyup', function() {
      if (!isPlaceholder) requestAnimationFrame(positionLiveCursor);
    });

    el.addEventListener('mouseup', function() {
      if (!isPlaceholder) requestAnimationFrame(positionLiveCursor);
    });

    document.addEventListener('selectionchange', positionLiveCursor);
    window.addEventListener('resize', positionLiveCursor);
    window.addEventListener('focus', positionLiveCursor);
  }

  function tryInit() {
    if (document.getElementById('create-project-input')) init();
    else setTimeout(tryInit, 50);
  }

  if (document.readyState !== 'loading') tryInit();
  else document.addEventListener('DOMContentLoaded', tryInit, { once: true });
})();
