<script lang="ts">
  export let timeMS: number;
  export let enabled: boolean;
</script>

<div
  style={`--timeMS: ${timeMS}ms`}
  class={`height-transition ${enabled ? "enabled" : "disabled"}`}
>
  <div>
    <slot />
  </div>
</div>

<style>
  .height-transition {
    display: grid;
    grid-template-rows: 0fr;
    transition: grid-template-rows var(--timeMS);
  }

  .height-transition > div {
    overflow: hidden;
  }

  .height-transition.enabled {
    grid-template-rows: 1fr;
  }
</style>
