<script context="module" lang="ts">
  import { writable, type Writable } from "svelte/store";
  export type Chat = {
    chat_id: string | number;
    creator_id?: number;
    creator?: Usuario;
    chat_name: string;
    chat_desc: string;
    chat_type: "USER" | "GROUP";
    last_message?: MensagemApi;
    date_created?: string;
  };
  let chats: Writable<Chat[] | undefined> = writable(undefined);

  export async function modificarChat(novoChat: Chat) {
    if (novoChat.last_message)
      novoChat.last_message.user = await requestUser(
        novoChat.last_message.user_id
      );
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
    let reqChats = JSON.parse(await res.text()) as Chat[];
    reqChats = await Promise.all(
      reqChats.map(async (chat) => {
        if (chat.creator_id) chat.creator = await requestUser(chat.creator_id);
        if (chat.last_message)
          chat.last_message.user = await requestUser(chat.last_message.user_id);
        return chat;
      })
    );
    chats.set(reqChats);
    console.log(chats);
  }
</script>

<script lang="ts">
  import { PUBLIC_URL_BACKEND } from "$env/static/public";
  import { getJson, postJson } from "../utils/requests";
  import {
    requestUser,
    selectChat,
    selectedChat,
    selectLastChat,
    type Usuario,
  } from "./+page.svelte";
  import ChatCard from "./ChatCard.svelte";
  import type { MensagemApi } from "./ContainerChat.svelte";
  import { onMount } from "svelte";

  let chatName = "";

  async function createChat() {
    // console.log(chatName);
    await postJson(`${location.protocol}//${PUBLIC_URL_BACKEND}/chat/create`, {
      nome: chatName,
    });
    chatName = "";
  }

  onMount(async () => {
    await getCards();
    console.log($chats);
    if (!$chats) return;
    selectLastChat($chats);
  });
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
