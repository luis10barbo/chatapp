<script lang="ts">
  import { onMount } from "svelte";
  import type { Mensagem } from "./CardMensagem.svelte";
  import CardMensagem from "./CardMensagem.svelte";
  import { getJson } from "../utils/requests";
  import { PUBLIC_URL_BACKEND } from "$env/static/public";
  import { cachedUsers, getUser } from "./+page.svelte";

  export let meuId: number;
  export let idChat: string;
  let chatHolder: HTMLDivElement;

  let mensagens: Mensagem[] = [];
  let contagemOnline = 0;
  let mensagemEnviar = "";

  type MensagemSocket = {
    message_type: string;
    message: string;
    id: number;
    date: string;
  };
  async function addMensagem(mensagem: MensagemSocket) {
    const usuario = await getUser(mensagem.id);
    mensagens = [
      ...mensagens,
      {
        data: new Date(mensagem.date.replace(" ", "T") + "Z"),
        id: mensagem.id,
        mensagem: mensagem.message,
        usuario: usuario,
      },
    ];
    chatHolder.scrollTop = chatHolder.scrollHeight;
  }

  function enviarMensagem() {
    ws.send(mensagemEnviar);
    const dateNow = new Date().toISOString();
    addMensagem({
      id: meuId,
      message: mensagemEnviar,
      message_type: "TEXT",
      date: dateNow.substring(0, dateNow.length - 1),
    });
    mensagemEnviar = "";
  }

  let ws: WebSocket;
  async function setupWebSocket() {
    // const res = await getJson(`http://${PUBLIC_URL_BACKEND}/chat/auth`);
    // if (res.status !== 200) {
    //   console.error("Erro ao authenticar chat");
    // }
    // const auth = await res.text();
    ws = new WebSocket(
      `ws://${PUBLIC_URL_BACKEND}/chat/d9b49810-a1cb-440a-9e66-c293aa61d4d9`
    );
    ws.addEventListener("open", (msg) => {
      console.log("Connected to Chat");
    });
    ws.addEventListener("message", (msg) => {
      mensagens = [...mensagens];
      const mensagem: MensagemSocket = JSON.parse(msg.data);
      if (mensagem.message_type === "TEXT") addMensagem(mensagem);
      else if (mensagem.message_type === "JOIN") contagemOnline++;
      else if (mensagem.message_type === "LEAVE") contagemOnline--;
      else if (mensagem.message_type === "INIT")
        contagemOnline = Number.parseInt(mensagem.message);
      else if (mensagem.message_type === "DISCONNECTED") {
        // TODO: utilizar mensagem deslogada
      }
    });
  }

  onMount(async () => {
    setupWebSocket();
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
      {#if contagemOnline > 0}
        <span id="curr-chat-online-count">{contagemOnline}</span> Online
      {/if}
    </p>
  </header>
  <div id="curr-chat-messages-holder" bind:this={chatHolder}>
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
