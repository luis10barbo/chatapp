<script context="module" lang="ts">
  export type Usuario = {
    user_email?: string;
    user_id: number;
    user_name?: string;
    user_nick: string;
    user_status?: string;
  };
  export let cachedUsers = new Map<Usuario["user_id"], Usuario>();
  export let selectedChat: Writable<undefined | Chat> = writable(undefined);
  const TAG_ULTIMO_CHAT_ID = "ULTIMO_CHAT_ID";
  export async function selectChat(chat: Chat) {
    localStorage.setItem(TAG_ULTIMO_CHAT_ID, chat.chat_id.toString());
    selectedChat.set(chat);
  }

  export async function selectLastChat(chats: Chat[]) {
    const ultimoChat = localStorage.getItem(TAG_ULTIMO_CHAT_ID);
    if (!ultimoChat) return;

    let idChat: Chat["chat_id"];
    if (!Number.isNaN(ultimoChat)) {
      idChat = ultimoChat;
    } else {
      idChat = Number.parseInt(ultimoChat);
    }

    for (let i = 0; i < chats.length; i++) {
      const chat = chats.at(i);
      if (!chat || chat.chat_id !== idChat) return;

      selectChat(chat);
    }
  }

  export async function requestUser(user_id: Usuario["user_id"]) {
    const res = await getJson(
      `${location.protocol}//${PUBLIC_URL_BACKEND}/user/info?id=${user_id}`
    );
    if (res.status !== 200) {
      console.error("Error fetching user " + user_id);
      return;
    }

    const user = JSON.parse(await res.text()) as Usuario;
    cachedUsers.set(user_id, user);
    return user;
  }

  export async function getUser(userId: Usuario["user_id"]) {
    if (cachedUsers.has(userId)) return cachedUsers.get(userId);

    return await requestUser(userId);
  }
</script>

<script lang="ts">
  import "./style.css";
  import ContainerChat from "./ContainerChat.svelte";
  import { getJson, postJson, requestPerfil } from "../utils/requests";
  import { onMount } from "svelte";
  import { PUBLIC_URL_BACKEND } from "$env/static/public";
  import ContainerChatSelector, {
    type Chat,
  } from "./ContainerChatSelector.svelte";
  import { writable, type Writable } from "svelte/store";

  let usuario: Usuario | undefined = undefined;
  function redirecionarLogin() {
    window.location.replace("/login");
  }
  async function deslogar() {
    await postJson(
      window.location.protocol + "//" + PUBLIC_URL_BACKEND + "/user/sair",
      {}
    );
    window.location.reload();
    // redirecionarLogin();
  }
  async function getUsuario() {
    const res = await requestPerfil();
    if (res.status === 200) {
      usuario = JSON.parse(await res.text()) as Usuario;
      if (!usuario) return window.location.reload();

      cachedUsers.set(usuario.user_id, usuario);
      return;
    }
    redirecionarLogin();
  }
  onMount(async () => {
    await getUsuario();
  });
</script>

<div id="page">
  {#if !usuario}
    <div
      style="width: 100%; font-size:30px; display:flex; flex-direction:column; justify-content:center; align-items:center;"
    >
      <p>ZipZop 2</p>
      <p style="font-size: 20px;">Carregando...</p>
    </div>
  {:else}
    <section id="chats">
      <header id="chats-header" class="section-header">
        <p>Conversas</p>
      </header>
      <ContainerChatSelector />
      <footer id="chats-footer" class="section-footer">
        <button>Contatos</button>
        <button>Perfil</button>
        <button on:click={deslogar}>Sair</button>
      </footer>
    </section>
    {#key $selectedChat}
      <ContainerChat chat={$selectedChat} meuId={usuario.user_id} />
    {/key}
  {/if}
</div>
