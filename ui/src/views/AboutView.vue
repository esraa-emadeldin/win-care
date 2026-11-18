<template>
  <div class="page">
    <section class="card panel">
      <img src="/icon.svg" alt="" width="64" height="64" />
      <h1>win-care</h1>
      <p class="muted">Version 0.1.0 · MIT licence · open source</p>
      <p>No telemetry. Nothing leaves this PC.</p>
      <p class="muted">
        Nothing is deleted before you review it, and deletes go to the Recycle Bin.
      </p>
    </section>
  </div>
</template>

<style scoped>
.panel {
  max-width: 480px;
  padding: calc(var(--space) * 4);
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space);
}
p {
  margin: 0;
}
</style>
