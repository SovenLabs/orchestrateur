<script lang="ts">
  import { harnessStore } from "$lib/stores/harness.svelte";

  let search = $state("");
  let token = $state("");
  let allowedIds = $state("");
  let enabled = $state(true);

  const filtered = $derived(
    harnessStore.channels.filter((c) =>
      c.display_name.toLowerCase().includes(search.toLowerCase()),
    ),
  );

  $effect(() => {
    const ch = harnessStore.selectedChannel;
    if (!ch) return;
    enabled = ch.enabled;
    token = "";
    allowedIds = "";
  });

  function badgeClass(b: string): string {
    if (b === "actif") return "harness-badge harness-badge--ok";
    if (b === "needs setup") return "harness-badge harness-badge--warn";
    return "harness-badge";
  }

  async function save() {
    const id = harnessStore.selectedChannelId;
    if (!id) return;
    await harnessStore.saveChannel(id, token, allowedIds, enabled);
  }
</script>

{#if harnessStore.messagingOpen}
  <div
    class="cosmic-drawer-scrim"
    role="presentation"
    onclick={() => harnessStore.closeMessaging()}
  ></div>
  <aside class="harness-messaging drawer-slide-right" aria-label="Messaging">
    <header class="harness-messaging__header">
      <div>
        <p class="cosmic-drawer__eyebrow">Settings · Messaging</p>
        <h2 class="cosmic-drawer__title">Canaux</h2>
        <p class="cosmic-drawer__subtitle">
          Connectez Orchestrateur à Telegram, Discord et Slack.
        </p>
      </div>
      <button
        type="button"
        class="cosmic-drawer__close"
        onclick={() => harnessStore.closeMessaging()}
        aria-label="Fermer"
      >
        ✕
      </button>
    </header>

    <div class="harness-messaging__body">
      <aside class="harness-messaging__list">
        <input
          type="search"
          class="harness-input harness-input--dark"
          placeholder="Rechercher…"
          bind:value={search}
        />
        <ul class="harness-channel-list">
          {#each filtered as ch (ch.id)}
            <li>
              <button
                type="button"
                class="harness-channel-item {harnessStore.selectedChannelId === ch.id
                  ? 'harness-channel-item--active'
                  : ''}"
                onclick={() => (harnessStore.selectedChannelId = ch.id)}
              >
                <span class="harness-channel-item__name">{ch.display_name}</span>
                <span class="harness-channel-item__badges">
                  {#each ch.badges as b (b)}
                    <span class={badgeClass(b)}>{b}</span>
                  {/each}
                </span>
              </button>
            </li>
          {/each}
        </ul>
        {#if harnessStore.services}
          <p class="harness-messaging__gateway">
            gateway: <span class={harnessStore.services.gateway === "alive" ? "text-[var(--status-green)]" : "text-[var(--status-orange)]"}>{harnessStore.services.gateway}</span>
          </p>
        {/if}
      </aside>

      {#if harnessStore.selectedChannel}
        {@const ch = harnessStore.selectedChannel}
        <section class="harness-messaging__detail scroll-thin">
          <div class="harness-detail__head">
            <h3>{ch.display_name}</h3>
            <div class="harness-detail__badges">
              {#each ch.badges as b (b)}
                <span class={badgeClass(b)}>{b}</span>
              {/each}
            </div>
          </div>
          <p class="harness-detail__hint">{ch.setup_hint}</p>

          <div class="harness-detail__section">
            <p class="harness-detail__label">OBTENIR VOS IDENTIFIANTS</p>
            <a
              href={ch.setup_url}
              target="_blank"
              rel="noopener noreferrer"
              class="harness-link"
            >
              Ouvrir le guide de configuration ↗
            </a>
          </div>

          <div class="harness-detail__section">
            <p class="harness-detail__label">REQUIS</p>
            <label class="harness-field harness-field--dark">
              <span>{ch.token_env}</span>
              <input
                type="password"
                class="harness-input harness-input--dark"
                placeholder={ch.token_set ? "•••••• (laisser vide pour conserver)" : "Coller le token"}
                bind:value={token}
              />
            </label>
          </div>

          {#if ch.id === "discord" || ch.id === "telegram"}
            <div class="harness-detail__section">
              <p class="harness-detail__label">RECOMMANDÉ</p>
              <label class="harness-field harness-field--dark">
                <span>IDs utilisateur autorisés (séparés par des virgules)</span>
                <input
                  type="text"
                  class="harness-input harness-input--dark"
                  placeholder="123456789, 987654321"
                  bind:value={allowedIds}
                />
              </label>
            </div>
          {/if}

          <label class="harness-check harness-check--dark">
            <input type="checkbox" bind:checked={enabled} />
            Activer {ch.display_name} dans orchestrator.toml
          </label>

          <div class="harness-detail__footer">
            <button type="button" class="harness-btn harness-btn--primary" onclick={save}>
              Enregistrer
            </button>
            {#if harnessStore.saveMessage}
              <span class="harness-save-msg">{harnessStore.saveMessage}</span>
            {/if}
          </div>
        </section>
      {/if}
    </div>
  </aside>
{/if}