<script lang="ts">
  import { formatBytes } from "../format";
  import type { SystemInfoData } from "../ipc";

  let { data }: { data: SystemInfoData } = $props();
</script>

<div class="panel">
  <div class="row"><span class="label">OS</span><span>{data.os_name} {data.os_version}</span></div>
  <div class="row"><span class="label">Kernel</span><span>{data.kernel_version}</span></div>
  <div class="row"><span class="label">Host</span><span>{data.host_name}</span></div>
  <div class="row"><span class="label">CPU</span><span>{data.cpu_brand} ({data.cpu_count} cores)</span></div>
  <div class="row">
    <span class="label">Memory</span>
    <span>{formatBytes(data.used_memory_bytes)} / {formatBytes(data.total_memory_bytes)}</span>
  </div>
  {#each data.disks as disk}
    <div class="row">
      <span class="label">Disk ({disk.mount_point})</span>
      <span>{formatBytes(disk.available_bytes)} free / {formatBytes(disk.total_bytes)}</span>
    </div>
  {/each}
</div>

<style>
  .panel {
    align-self: flex-start;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 0.7rem 0.9rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.85rem;
    min-width: 240px;
  }
  .row {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
  }
  .label {
    color: var(--text-muted);
  }
</style>
