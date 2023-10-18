<script context="module" lang="ts">
  export type Mensagem = {
    mensagem: string;
    data: Date;
    idUsuario: number;
    id: string;
    usuario: Usuario | undefined;
  };
</script>

<script lang="ts">
  import { getUser, type Usuario } from "./+page.svelte";

  export let mensagem: Mensagem;
  export let meuId: Mensagem["idUsuario"];
  let user: Usuario | undefined = undefined;

  async function adquirirUsuario() {
    user = await getUser(mensagem.idUsuario);
  }
  adquirirUsuario();
</script>

{#if user}
  <div class={`msg-card ` + (meuId === mensagem.idUsuario ? " owned" : "")}>
    <header class="msg-card-header">
      <p class="msg-card-name">{user?.user_nick}</p>
    </header>
    <footer class="msg-card-footer">
      <p class="msg-card-time">
        {mensagem.data.getHours()}:{mensagem.data.getMinutes()}
      </p>
      <p class="msg-card-status">
        <span class="msg-card-msg">{mensagem.mensagem}</span>
      </p>
    </footer>
  </div>
{/if}
