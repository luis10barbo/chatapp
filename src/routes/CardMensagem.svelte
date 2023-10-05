<script context="module" lang="ts">
  export type Mensagem = {
    nome: string;
    mensagem: string;
    horario: string;
    id: number;
  };
</script>

<script lang="ts">
  export let mensagem: Mensagem;
  export let meuId: Mensagem["id"];
</script>

<div class={`msg-card ` + (meuId === mensagem.id ? " owned" : "")}>
  <header class="msg-card-header">
    <p class="msg-card-name">{mensagem.nome}</p>
  </header>
  <footer class="msg-card-footer">
    <p class="msg-card-time">{mensagem.horario}</p>
    <p class="msg-card-status">
      <span class="msg-card-msg">{mensagem.mensagem}</span>
    </p>
  </footer>
</div>
