<script lang="ts">
  import { harnessStore } from "$lib/stores/harness.svelte";

  let profile = $state("ai_assisted");
  let llm = $state("ollama");
  let installDaemon = $state(false);
  let step = $state<"welcome" | "options" | "bootstrap">("welcome");
  let saving = $state(false);

  const profiles = [
    { id: "local_only", label: "Local only", hint: "zéro egress cloud" },
    { id: "ai_assisted", label: "AI assisted", hint: "LLM cloud selon profil" },
    { id: "strict", label: "Strict", hint: "restrictions renforcées" },
  ];

  async function startSetup() {
    step = "options";
  }

  async function apply() {
    saving = true;
    step = "bootstrap";
    try {
      if (profile === "local_only") llm = "ollama";
      await harnessStore.completeOnboard({
        profile,
        llm,
        install_daemon: installDaemon,
      });
    } finally {
      saving = false;
    }
  }
</script>

{#if harnessStore.setupOpen}
  <div class="harness-scrim" role="presentation"></div>
  <div class="harness-setup" role="dialog" aria-labelledby="setup-title">
    {#if step === "welcome"}
      <div class="harness-card harness-card--light panel-enter">
        <h1 id="setup-title" class="harness-card__title">
          Configurons votre harness Orchestrateur
        </h1>
        <p class="harness-card__subtitle">
          Connectez un provider LLM pour commencer. La plupart des options prennent un clic.
        </p>
        <div class="harness-card__actions">
          <button type="button" class="harness-btn harness-btn--primary" onclick={startSetup}>
            Commencer
          </button>
          <button
            type="button"
            class="harness-btn harness-btn--ghost"
            onclick={() => harnessStore.dismissSetup()}
          >
            Plus tard
          </button>
        </div>
      </div>
    {:else if step === "options"}
      <div class="harness-card harness-card--light panel-enter">
        <h2 class="harness-card__title">Profil & provider</h2>
        <label class="harness-field">
          <span>Profil sécurité</span>
          <select bind:value={profile} class="harness-input">
            {#each profiles as p (p.id)}
              <option value={p.id}>{p.label} — {p.hint}</option>
            {/each}
          </select>
        </label>
        {#if profile !== "local_only"}
          <label class="harness-field">
            <span>Provider LLM</span>
            <select bind:value={llm} class="harness-input">
              <option value="ollama">ollama (local)</option>
              <option value="xai">xAI (XAI_API_KEY)</option>
            </select>
          </label>
        {/if}
        <label class="harness-check">
          <input type="checkbox" bind:checked={installDaemon} />
          Installer le daemon au démarrage Windows
        </label>
        <div class="harness-card__actions">
          <button
            type="button"
            class="harness-btn harness-btn--primary"
            disabled={saving}
            onclick={apply}
          >
            Installer & démarrer
          </button>
        </div>
      </div>
    {:else}
      <div class="harness-card harness-card--light panel-enter">
        <h2 class="harness-card__title">Démarrage Orchestrateur</h2>
        <p class="harness-card__status">{harnessStore.bootstrapStatus}</p>
        <div class="harness-progress" aria-valuenow={harnessStore.bootstrapPercent} role="progressbar">
          <div
            class="harness-progress__fill"
            style="width: {harnessStore.bootstrapPercent}%"
          ></div>
        </div>
        <p class="harness-card__percent">{harnessStore.bootstrapPercent}%</p>
        {#if harnessStore.services}
          <p class="harness-card__meta">
            daemon: {harnessStore.services.daemon} · gateway: {harnessStore.services.gateway}
          </p>
        {/if}
        {#if !harnessStore.bootstrapping}
          <div class="harness-card__actions">
            <button
              type="button"
              class="harness-btn harness-btn--primary"
              onclick={() => {
                harnessStore.dismissSetup();
                harnessStore.openMessaging();
              }}
            >
              Configurer les canaux
            </button>
            <button
              type="button"
              class="harness-btn harness-btn--ghost"
              onclick={() => harnessStore.dismissSetup()}
            >
              Fermer
            </button>
          </div>
        {/if}
      </div>
    {/if}
  </div>
{/if}