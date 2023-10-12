<script context="module" lang="ts">
  export type Usuario = {
    user_email?: string;
    user_id: number;
    user_name?: string;
    user_nick: string;
    user_status?: string;
  };
  export let cachedUsers = new Map<Usuario["user_id"], Usuario>();

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

  let usuario: Usuario | undefined = undefined;
  function redirecionarLogin() {
    window.location.replace("/login");
  }
  async function deslogar() {
    await postJson("http://" + PUBLIC_URL_BACKEND + "/user/sair", {});
    // window.location.reload();
    // redirecionarLogin();
  }
  async function getUsuario() {
    const res = await requestPerfil();
    if (res.status === 200) {
      usuario = JSON.parse(await res.text()) as Usuario;
      // if (!usuario) return redirecionarLogin();

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
      <section id="chats-holder">
        <div id="chat-search-holder">
          <input placeholder="Pesquisar uma conversa..." />
          <button>Pesquisar</button>
        </div>
        <button class="chat-card">
          <header class="chat-card-header">
            <p class="chat-card-name">Titulo</p>
          </header>
          <footer class="chat-card-footer">
            <p class="chat-card-time">10:45</p>
            <p class="chat-card-status">
              <span class="chat-card-status-name">Luis</span>
              <span class="chat-card-status-msg">teste teste</span>
            </p>
          </footer>
        </button>
      </section>
      <footer id="chats-footer" class="section-footer">
        <button>Contatos</button>
        <button>Perfil</button>
        <button on:click={deslogar}>Sair</button>
      </footer>
    </section>
    <ContainerChat
      idChat="d9b49810-a1cb-440a-9e66-c293aa61d4d9"
      meuId={usuario.user_id}
    />
  {/if}
</div>
