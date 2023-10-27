<script context="module" lang="ts">
  let ws: WebSocket | undefined;
  let infoChats: Set<Chat> = new Set();

  export type MensagemApi = {
    id: string;
    message: string;
    date_created: string;
    user_id: number;
    user?: Usuario;
  };
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import type { Mensagem } from "./CardMensagem.svelte";
  import CardMensagem from "./CardMensagem.svelte";
  import { adquirirProtocoloWS, getJson, postJson } from "../utils/requests";
  import { PUBLIC_URL_BACKEND } from "$env/static/public";
  import {
    cachedUsers,
    getUser,
    selectChat,
    sidebarAtivada,
    type Usuario,
  } from "./+page.svelte";
  import { tick } from "svelte";
  import {
    modificarChat,
    type Chat,
    removerChat,
  } from "./ContainerChatSelector.svelte";
  import { parseDataDB } from "../utils/date";
  import HeightTransition from "$lib/utils/components/HeightTransition.svelte";
  import { writable } from "svelte/store";

  async function addMensagem(mensagem: MensagemSocket, atualizarChat: boolean) {
    if (!chat) return;

    const usuario = await getUser(mensagem.id);
    const novaMensagem = {
      data: parseDataDB(mensagem.date),
      idUsuario: mensagem.id,
      mensagem: mensagem.message,
      usuario: usuario,
      id: "",
    };
    mensagens = [...mensagens, novaMensagem];
    chat.last_message = {
      date_created: mensagem.date,
      id: novaMensagem.id,
      message: novaMensagem.mensagem,
      user_id: novaMensagem.idUsuario,
    };
    modificarChat(chat);

    await tick();
    await tick();
    scrollToBottomMsgs();
  }

  function scrollToBottomMsgs() {
    chatHolder.scrollTop = chatHolder.scrollHeight;
  }

  async function enviarMensagem() {
    if (!ws) await setupWebSocket();
    if (!ws) return;

    ws.send(mensagemEnviar);
    const dateNow = new Date().toISOString();
    addMensagem(
      {
        id: meuId,
        message: mensagemEnviar,
        message_type: "TEXT",
        date: dateNow.substring(0, dateNow.length - 1),
      },
      true
    );
    mensagemEnviar = "";
  }

  async function desconectar() {
    if (!ws) return;
    ws.close();
    ws = undefined;
  }

  async function setupWebSocket() {
    if (!chat) return;
    if (ws) ws.close();

    ws = new WebSocket(
      `${adquirirProtocoloWS()}//${PUBLIC_URL_BACKEND}/chat/connect/${
        chat.chat_id
      }?t=GROUP`
    );
    if (desconectado) {
      desconectado = false;
      mostrarAlerta = false;
    }
    ws.addEventListener("message", (msg) => {
      const mensagem: MensagemSocket = JSON.parse(msg.data);
      console.log(mensagem);
      switch (mensagem.message_type) {
        case "TEXT":
          addMensagem(mensagem, true);
          break;
        case "JOIN":
          contagemOnline++;
          break;
        case "LEAVE":
          contagemOnline--;
          break;
        case "INIT":
          contagemOnline = Number.parseInt(mensagem.message);
          break;
        case "DISCONNECTED":
          mostrarAlerta = true;
          desconectado = true;
          break;
        case "CHAT_DELETED":
          if (!chat) break;
          removerChat(chat);
          selectChat(undefined);
          break;
      }
    });
    ws.addEventListener("close", () => {
      alerta = "Você foi desconectado";
      mostrarAlerta = true;
      desconectado = true;
    });
  }

  async function getMessages(offset: number) {
    if (!chat) return;

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
    return resMessages;
  }

  onMount(async () => {
    const messages = await getMessages(0);
    if (chat && messages) {
      chat.last_message = messages.at(0);
      modificarChat(chat);
    }
    await setupWebSocket();
    mostrarAlerta = false;
    setTimeout(() => {
      // loading = false;
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

  async function apagarChat() {
    if (!chat) return;
    const res = await postJson(
      window.location.protocol + "//" + PUBLIC_URL_BACKEND + "/chat/remove",
      {
        chat_id: chat.chat_id,
      }
    );
    if (res.status === 200) {
      removerChat(chat);
      selectChat(undefined);
    }
  }

  async function atualizarChat() {
    if (!tituloModificado && !descricaoModificada && !chat) return;
    console.log(tituloModificado, descricaoModificada);
    const chatModificado = {
      ...chat,
      chat_name: tituloModificado,
      chat_desc: descricaoModificada,
    } as Chat;

    const res = await postJson(
      window.location.protocol + "//" + PUBLIC_URL_BACKEND + "/chat/update",
      chatModificado
    );
    if (res.status !== 200) return;

    chat = chatModificado;
  }

  export let meuId: number;
  export let chat: Chat | undefined;
  let loading = false;
  let desconectado = false;
  let alerta = "Carregando mensagens...";
  let mostrarAlerta = true;
  let chatHolder: HTMLDivElement;

  let mensagens: Mensagem[] = [];
  let contagemOnline = 0;
  let mensagemEnviar = "";
  let mostrarPerfil = false;

  type MensagemSocket = {
    message_type: string;
    message: string;
    id: number;
    date: string;
  };

  let tituloModificado = chat?.chat_name;
  let descricaoModificada = chat?.chat_desc;
  let eDonoChat = chat?.creator_id === meuId;
</script>

<section id="curr-chat" class={`${loading ? "notransition" : ""}`}>
  {#if chat}
    <div id="perfil-chat-container" class={`${mostrarPerfil ? "" : "hidden"}`}>
      <HeightTransition enabled={mostrarPerfil} timeMS={150}>
        <div id="perfil-chat">
          <!-- {JSON.stringify(chat)} -->
          <header id="perfil-chat-header">
            <img id="img-curr-chat" />
            {#if !eDonoChat}
              <div id="perfil-titulo-holder">
                <p id="perfil-chat-titulo">{chat.chat_name}</p>
              </div>
            {:else}
              <input
                type="text"
                placeholder="Sem titulo"
                bind:value={tituloModificado}
              />
            {/if}
            <button
              on:click={() => {
                mostrarPerfil = false;
              }}>Fechar</button
            >
          </header>
          {#if !eDonoChat}
            <p id="perfil-chat-desc">
              {chat.chat_desc ? chat.chat_desc : "Sem descricao"}
            </p>
          {:else}
            <input
              type="text"
              placeholder="Descrição"
              bind:value={descricaoModificada}
            />
          {/if}

          <p id="perfil-data-criada">
            Criado em {chat.date_created} <br /> por
            <button
              class="text-button"
              style="font-size: 12px;"
              on:click={() => {
                if (!chat?.creator) return;
                selectChat({
                  chat_desc: "",
                  chat_id: chat.creator.user_id,
                  chat_name: chat.creator.user_name
                    ? chat.creator.user_name
                    : chat.creator.user_nick,
                  chat_type: "USER",
                  last_message: undefined,
                });
              }}><b>{chat.creator?.user_nick}</b></button
            >
          </p>
          {#if eDonoChat}
            <div id="perfil-botoes-adm">
              <button
                on:click={async () => {
                  apagarChat();
                }}>Apagar</button
              >
              <button
                on:click={async () => {
                  atualizarChat();
                }}>Atualizar</button
              >
            </div>
          {/if}
          <section id="perfil-aba-participantes">
            <p>Participantes</p>
            <p>(ainda nao implementado)</p>
          </section>
        </div>
      </HeightTransition>
    </div>
  {/if}
  <header id="curr-chat-header" class="section-header">
    {#if !$sidebarAtivada}
      <button
        class="toggle-sidebar"
        on:click={() => {
          sidebarAtivada.update((ultimoValor) => !ultimoValor);
        }}
        >Menu
      </button>
    {/if}

    <button
      id="curr-chat-desc"
      on:click={() => {
        mostrarPerfil = true;
      }}
    >
      <img id="img-curr-chat" />
      <div id="curr-chat-info">
        <p>{chat ? chat.chat_name : "Nenhum chat selecionado"}</p>
        <p class="chat-status" />
      </div>
    </button>

    <!-- <button
      on:click={() => {
        desconectar();
      }}>Desconectar</button
    > -->

    <p id="curr-chat-online-holder">
      {#if contagemOnline > 0}
        <span id="curr-chat-online-count">{contagemOnline}</span> Online
      {/if}
    </p>
  </header>
  <div id="curr-chat-messages-holder" bind:this={chatHolder}>
    {#if chat}
      <div id="aviso-container" class={` ${mostrarAlerta ? "" : "hidden"}`}>
        <div id="aviso-chat">
          {alerta}
          {#if desconectado}
            <div id="aviso-botoes">
              <button
                on:click={() => {
                  setupWebSocket();
                }}>Reconectar</button
              >
            </div>
          {/if}
        </div>
      </div>
    {/if}
    {#each mensagens as mensagem}
      <CardMensagem {mensagem} {meuId} />
    {/each}
  </div>
  <footer id="curr-chat-footer" class="section-footer">
    {#if chat}
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
    {/if}
  </footer>
</section>
