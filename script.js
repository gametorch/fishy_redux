/* Fishy Redux front-end script */

/**
 * Simple video carousel for small screens.
 * Cycles through all <video> elements in #videoCarousel every few seconds.
 */
function initCarousel() {
  const carousel = document.getElementById("videoCarousel");
  if (!carousel) return;
  const videos = Array.from(carousel.querySelectorAll("video"));
  if (videos.length === 0) return;

  let current = 0;
  videos[current].classList.add("active");
  // Auto-play the active video.
  videos[current].play().catch(() => {/* autoplay might be blocked */});

  setInterval(() => {
    // Pause & hide current
    videos[current].pause();
    videos[current].classList.remove("active");

    current = (current + 1) % videos.length;

    // Show & play next
    videos[current].classList.add("active");
    videos[current].currentTime = 0;
    videos[current].play().catch(() => {/* ignore */});
  }, 8000); // 8-second interval
}

/**
 * Dynamically import the WASM/JS glue when the game canvas is visible.
 * Assumes the build pipeline outputs fishy_redux.js next to this script.
 */
async function initGameIfNeeded() {
  const canvas = document.getElementById("gameCanvas");
  if (!canvas) return;

  // Only initialise when the canvas is displayed (i.e., not small screen).
  const isVisible = canvas.offsetParent !== null;
  if (!isVisible) return;

  // Record current scroll position so we can restore it later.
  const originalScrollY = window.scrollY;

  try {
    // The generated Bevy WASM glue typically has the same name as the crate.
    // Adjust the path as needed if the build output differs.
    const wasm = await import("./wasm_out/fishy_redux.js");
    // Most wasm-bindgen glue scripts expose `default` as the init.
    await wasm.default();

    /* ------------------------------------------------------
       Post-init: the engine (Bevy / macroquad / etc.) will
       usually append its own <canvas> element to <body>.

       Strategy:
       1. Find the newly created canvas that is NOT the
          placeholder (#gameCanvas).
       2. Replace the placeholder with the real canvas so
          it inherits the existing CSS + border/frame.
    ------------------------------------------------------ */
    const placeholder = document.getElementById("gameCanvas");
    if (!placeholder) return;

    // Any canvas that isn't the placeholder is assumed to be the real one.
    const realCanvas = Array.from(document.querySelectorAll("canvas"))
      .find(c => c !== placeholder);

    if (realCanvas) {
      // Move it into the interactive area and adopt the styling ID.
      placeholder.replaceWith(realCanvas);
      realCanvas.id = "gameCanvas";
    }
  } catch (err) {
    console.error("Failed to load WebAssembly game:", err);
  } finally {
    // Restore the scroll position (use a timeout to allow layout to settle).
    setTimeout(() => {
      window.scrollTo({ top: originalScrollY, behavior: "instant" });
    }, 0);
  }
}

// Initialise features after DOM is ready
window.addEventListener("DOMContentLoaded", () => {
  initCarousel();
  initGameIfNeeded();
}); 