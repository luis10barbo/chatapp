<script lang="ts">
  import { onMount } from "svelte";
  import type { Mensagem } from "./CardMensagem.svelte";
  import CardMensagem from "./CardMensagem.svelte";

  export let meuId: number;
  export let idChat: string;
  let mensagens: Mensagem[];
  $: mensagens = [];

  let mensagemEnviar = "";

  type MensagemSocket = {
    message_type: string;
    message: string;
    id: number;
  };
  function addMensagem(mensagem: MensagemSocket) {
    mensagens = [
      ...mensagens,
      { horario: "00:00", id: 0, mensagem: mensagem.message, nome: "Teste" },
    ];
  }

  function enviarMensagem() {
    ws.send(mensagemEnviar);
    addMensagem({ id: meuId, message: mensagemEnviar, message_type: "TEXT" });
    mensagemEnviar = "";
  }
  let ws: WebSocket;
  onMount(async () => {
    console.log(WebSocket);
    ws = new WebSocket(
      "ws://127.0.0.1:8080/chats/d9b49810-a1cb-440a-9e66-c293aa61d4d9"
    );
    ws.addEventListener("open", (msg) => {
      console.log("Connected to Chat");
    });
    ws.addEventListener("message", (msg) => {
      mensagens = [...mensagens];
      const mensagem: MensagemSocket = JSON.parse(msg.data);
      console.log(mensagem);
      if (mensagem.message_type === "TEXT") addMensagem(mensagem);
    });
  });
</script>

<section id="curr-chat">
  <header id="curr-chat-header" class="section-header">
    <img id="img-curr-chat" />
    <div id="curr-chat-info">
      <p>Grupo Atual</p>
      <p class="chat-status" />
    </div>
    <p id="curr-chat-online-holder">
      <span id="curr-chat-online-count">0</span> Online
    </p>
  </header>
  <div id="curr-chat-messages-holder">
    {#each mensagens as mensagem}
      <CardMensagem {mensagem} {meuId} />
    {/each}
  </div>
  <footer id="curr-chat-footer" class="section-footer">
    <input
      bind:value={mensagemEnviar}
      id="send-message"
      placeholder="Escreva uma mensagem..."
      on:keydown={(event) => {
        if (event.key === "Enter") {
          enviarMensagem();
        }
      }}
    />
    <button
      id="send-message-button"
      on:click={() => {
        enviarMensagem();
      }}>Enviar</button
    >
  </footer>
</section>
