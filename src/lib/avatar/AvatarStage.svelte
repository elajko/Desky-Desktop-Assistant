<script lang="ts">
  import { chatStore } from "../stores/chat.svelte";
  import bodyImg from "./sprites/body.png";
  import headOpen from "./sprites/idle/head_open.png";
  import headBlink from "./sprites/idle/head_blink.png";
  import headClosed from "./sprites/talking/head_closed.png";
  import headMouthOpen from "./sprites/talking/head_mouthopen.png";

  const talking = $derived(chatStore.streaming && chatStore.streamingText.length > 0);

  let blinking = $state(false);
  let mouthOpen = $state(false);

  // Blink loop: runs continuously regardless of talking state, on a
  // randomized interval so it doesn't look mechanical.
  $effect(() => {
    let blinkTimeout: ReturnType<typeof setTimeout>;
    let openAgainTimeout: ReturnType<typeof setTimeout>;

    function scheduleNextBlink() {
      const delay = 2500 + Math.random() * 3000;
      blinkTimeout = setTimeout(() => {
        blinking = true;
        openAgainTimeout = setTimeout(() => {
          blinking = false;
          scheduleNextBlink();
        }, 140);
      }, delay);
    }
    scheduleNextBlink();

    return () => {
      clearTimeout(blinkTimeout);
      clearTimeout(openAgainTimeout);
    };
  });

  // Mouth-flap loop: only while actively talking (real text is streaming
  // in), reset to closed the moment talking stops.
  $effect(() => {
    if (!talking) {
      mouthOpen = false;
      return;
    }
    const interval = setInterval(() => {
      mouthOpen = !mouthOpen;
    }, 160);
    return () => clearInterval(interval);
  });

  const headSrc = $derived(
    talking ? (mouthOpen ? headMouthOpen : headClosed) : blinking ? headBlink : headOpen,
  );
</script>

<div class="avatar-stage">
  <img class="body" src={bodyImg} alt="" draggable="false" />
  <img class="head" src={headSrc} alt="" draggable="false" />
</div>

<style>
  .avatar-stage {
    position: relative;
    height: 100%;
    width: auto;
    /* Combined canvas: two 512x512 sprites overlapped by 227px (see below).
       Height comes from the parent; width is derived from it so nothing
       stretches. */
    aspect-ratio: 512 / 739;
  }
  .avatar-stage img {
    position: absolute;
    left: 0;
    width: 100%;
    height: auto;
    user-select: none;
    pointer-events: none;
  }
  /* Alignment computed from the sprites' actual content bounding boxes so
     the neck lines up: both are 512x512 canvases, head content bottoms out
     around y=381, body content starts around y=139 — offsetting body down
     by 227px (of the combined 739px canvas, ~30.72%) connects them. Both
     sprites are perfect squares, so width:100%/height:auto sizes each to
     exactly (512/739) of the container's height automatically. */
  .head {
    top: 0;
  }
  .body {
    top: 30.72%;
  }
</style>
