<script context="module" lang="ts">
  let ws: WebSocket;
  let infoChats: Set<Chat> = new Set();
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import type { Mensagem } from "./CardMensagem.svelte";
  import CardMensagem from "./CardMensagem.svelte";
  import { adquirirProtocoloWS, getJson } from "../utils/requests";
  import { PUBLIC_URL_BACKEND } from "$env/static/public";
  import { cachedUsers, getUser } from "./+page.svelte";
  import { tick } from "svelte";
  import type { Chat } from "./ContainerChatSelector.svelte";

  export let meuId: number;
  export let chat: Chat;
  let loading = true;
  let alerta = "Você fez login em outra localização. Desconectado!";
  let mostrarAlerta = false;
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

  type MensagemApi = {
    id: string;
    message: string;
    date_created: string;
    user_id: number;
  };

  async function addMensagem(mensagem: MensagemSocket) {
    const usuario = await getUser(mensagem.id);
    mensagens = [
      ...mensagens,
      {
        data: new Date(mensagem.date.replace(" ", "T") + "Z"),
        idUsuario: mensagem.id,
        mensagem: mensagem.message,
        usuario: usuario,
        id: "",
      },
    ];
    await tick();
    await tick();
    scrollToBottomMsgs();
  }

  function scrollToBottomMsgs() {
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

  async function setupWebSocket() {
    if (ws) ws.close();

    ws = new WebSocket(
      `${adquirirProtocoloWS()}//${PUBLIC_URL_BACKEND}/chat/connect/${
        chat.chat_id
      }?t=GROUP`
    );
    ws.addEventListener("message", (msg) => {
      const mensagem: MensagemSocket = JSON.parse(msg.data);
      console.log(mensagem);
      if (mensagem.message_type === "TEXT") addMensagem(mensagem);
      else if (mensagem.message_type === "JOIN") contagemOnline++;
      else if (mensagem.message_type === "LEAVE") contagemOnline--;
      else if (mensagem.message_type === "INIT")
        contagemOnline = Number.parseInt(mensagem.message);
      else if (mensagem.message_type === "DISCONNECTED") {
        // TODO: utilizar mensagem deslogada
        mostrarAlerta = true;
      }
    });
    ws.addEventListener("close", () => {
      console.log("desconectado");
      mostrarAlerta = true;
    });
  }

  async function getMessages(offset: number) {
    const res = await getJson(
      `${window.location.protocol}//${PUBLIC_URL_BACKEND}/chat/messages/${chat.chat_id}?offset=${offset}`
    );
    if (res.status !== 200) {
      return;
    }
    const resMessages: MensagemApi[] = JSON.parse(await res.text());
    const messages_parsed = await parse_msgs(resMessages);
    mensagens = [...messages_parsed, ...mensagens];
    await tick();
    await tick();
    scrollToBottomMsgs();
  }

  onMount(async () => {
    getMessages(0);
    setupWebSocket();
    setTimeout(() => {
      loading = false;
    }, 100);
  });

  async function parse_msgs(messages: MensagemApi[]) {
    const messages_parsed = await Promise.all(
      messages.map(async (message) => {
        return {
          data: new Date(message.date_created.replace(" ", "T") + "Z"),
          idUsuario: message.user_id,
          mensagem: message.message,
          usuario: await getUser(message.user_id),
          id: message.id,
        } as Mensagem;
      })
    );
    return messages_parsed;
  }
</script>

<section id="curr-chat" class={`${loading ? "notransition" : ""}`}>
  <div id="aviso-container" class={` ${mostrarAlerta ? "" : "hidden"}`}>
    <div id="aviso">{alerta}</div>
  </div>
  <header id="curr-chat-header" class="section-header">
    <img id="img-curr-chat" />
    <div id="curr-chat-info">
      <p>{chat.chat_name}</p>
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
