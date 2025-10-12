// Filter out noisy Dioxus warnings
(function() {
    const originalWarn = console.warn;
    console.warn = function(...args) {
        const message = args.join(' ');
        // Filter out Dioxus style change warnings
        if (message.includes('Changing the props of `Style {}` is not supported')) {
            return;
        }
        originalWarn.apply(console, args);
    };
})();
