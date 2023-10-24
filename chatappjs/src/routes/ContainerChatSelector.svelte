<script context="module" lang="ts">
  import { writable, type Writable } from "svelte/store";
  export type Chat = {
    chat_id: string | number;
    chat_name: string;
    chat_desc: string;
    chat_type: "USER" | "GROUP";
    last_message: MensagemApi | undefined;
  };
  let chats: Writable<Chat[] | undefined> = writable(undefined);

  export function modificarChat(novoChat: Chat) {
    chats.update((chats) => {
      return chats?.map((chat) => {
        if (chat.chat_id !== novoChat.chat_id) return chat;
        return novoChat;
      });
    });
  }
  async function getCards() {
    const res = await getJson(
      location.protocol + "//" + PUBLIC_URL_BACKEND + "/chat/"
    );
    if (res.status !== 200) {
      console.error("Erro adquirindo chats");
      return;
    }
    chats.set(JSON.parse(await res.text()) as Chat[]);
    console.log(chats);
  }
</script>

<script lang="ts">
  import { PUBLIC_URL_BACKEND } from "$env/static/public";
  import { getJson, postJson } from "../utils/requests";
  import { selectChat, selectedChat } from "./+page.svelte";
  import ChatCard from "./ChatCard.svelte";
  import type { MensagemApi } from "./ContainerChat.svelte";

  let chatName = "";

  getCards();
  async function createChat() {
    // console.log(chatName);
    await postJson(`${location.protocol}//${PUBLIC_URL_BACKEND}/chat/create`, {
      nome: chatName,
    });
    chatName = "";
  }
</script>

<section id="chats-holder">
  <div id="chat-search-holder">
    <input placeholder="Pesquisar uma conversa..." />
    <button>Pesquisar</button>
  </div>

  <div id="chat-create-holder">
    <input placeholder="Nome do chat" bind:value={chatName} />
    <button
      on:click={() => {
        createChat();
      }}>Criar</button
    >
  </div>
  {#key chats}
    {#if $chats}
      {#each $chats as chat}
        <ChatCard
          {chat}
          click={() => {
            selectChat(chat);
          }}
        />
      {/each}
    {/if}
  {/key}
</section>
